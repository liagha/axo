use crate::{
    analyzer::Analyzer,
    data::{from_utf8, Identity, Str},
    internal::{
        platform::{
            read_dir, set_current_dir, stdin, stdout, Command, Read, Write,
        },
        session::{
            ANALYZE_STAGE, INTERPRET_STAGE, PARSE_STAGE, RESOLVE_STAGE, SCAN_STAGE,
        },
        Record, RecordKind, Session,
    },
    interpreter,
    interpreter::Interpreter,
    parser::{Parser, Symbol},
    resolver::Resolver,
    scanner::Scanner,
    tracker::Location,
};
#[cfg(unix)]
use std::io::IsTerminal;

pub struct Dialog {
    pub history: Vec<String>,
}

impl Dialog {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    #[cfg(unix)]
    pub fn read(&mut self, prompt: &str) -> Option<String> {
        if !stdin().is_terminal() {
            print!("{}", prompt);
            let _ = stdout().flush();

            let mut input = String::new();
            if stdin().read_line(&mut input).ok()? == 0 {
                return None;
            }

            while input.ends_with('\n') || input.ends_with('\r') {
                input.pop();
            }

            if !input.trim().is_empty() {
                self.history.push(input.clone());
            }

            return Some(input);
        }

        let _ = Command::new("stty").args(["raw", "-echo"]).status();

        let mut input = stdin();
        let mut buffer: Vec<char> = Vec::new();
        let mut cursor = 0;
        let mut index = self.history.len();

        let render = |chars: &[char], pos: usize| {
            let string: String = chars.iter().collect();
            print!("\r\x1b[2K{}{}", prompt, string);
            if chars.len() > pos {
                print!("\x1b[{}D", chars.len() - pos);
            }
            let _ = stdout().flush();
        };

        render(&buffer, cursor);

        let mut octet = [0u8; 1];
        loop {
            if input.read_exact(&mut octet).is_err() {
                break;
            }

            match octet[0] {
                3 | 4 => {
                    let _ = Command::new("stty").arg("sane").status();
                    println!();
                    return None;
                }
                13 | 10 => {
                    break;
                }
                127 | 8 => {
                    if cursor > 0 {
                        cursor -= 1;
                        buffer.remove(cursor);
                        render(&buffer, cursor);
                    }
                }
                27 => {
                    let mut sequence = [0u8; 2];
                    if input.read_exact(&mut sequence).is_ok() && sequence[0] == b'[' {
                        match sequence[1] {
                            b'A' => {
                                if index > 0 {
                                    index -= 1;
                                    buffer = self.history[index].chars().collect();
                                    cursor = buffer.len();
                                    render(&buffer, cursor);
                                }
                            }
                            b'B' => {
                                if index + 1 < self.history.len() {
                                    index += 1;
                                    buffer = self.history[index].chars().collect();
                                    cursor = buffer.len();
                                } else {
                                    index = self.history.len();
                                    buffer.clear();
                                    cursor = 0;
                                }
                                render(&buffer, cursor);
                            }
                            b'C' => {
                                if cursor < buffer.len() {
                                    cursor += 1;
                                    render(&buffer, cursor);
                                }
                            }
                            b'D' => {
                                if cursor > 0 {
                                    cursor -= 1;
                                    render(&buffer, cursor);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                byte => {
                    let mut sequence = vec![byte];
                    let length = match byte {
                        x if x & 0b1000_0000 == 0 => 1,
                        x if x & 0b1110_0000 == 0b1100_0000 => 2,
                        x if x & 0b1111_0000 == 0b1110_0000 => 3,
                        x if x & 0b1111_1000 == 0b1111_0000 => 4,
                        _ => 1,
                    };

                    for _ in 1..length {
                        let mut next = [0u8; 1];
                        if input.read_exact(&mut next).is_ok() {
                            sequence.push(next[0]);
                        }
                    }

                    if let Ok(string) = from_utf8(&sequence) {
                        for character in string.chars() {
                            if !character.is_control() {
                                buffer.insert(cursor, character);
                                cursor += 1;
                            }
                        }
                        render(&buffer, cursor);
                    }
                }
            }
        }

        let _ = Command::new("stty").arg("sane").status();
        println!();

        let result: String = buffer.into_iter().collect();
        if !result.trim().is_empty() {
            self.history.push(result.clone());
        }

        Some(result)
    }
}

pub fn refresh<'a>(session: &mut Session<'a>, mut core: Option<&mut Interpreter<'a>>, keys: &[Identity]) {
    session.errors.clear();
    session.bootstrap();
    if !session.prepare() {
        session.report_all();
        return;
    }

    loop {
        let mut changed = false;

        for key in session.source_keys(keys) {
            let signature = session.scan_signature(key);
            if session.stage_value(SCAN_STAGE, key) != signature
                || session.records.get(&key).unwrap().fetch(1).is_none()
            {
                let before = session.records.get(&key).map(|record| record.artifact_version(1)).unwrap_or(0);
                Scanner::execute(session, &[key]);
                let after = session.records.get(&key).map(|record| record.artifact_version(1)).unwrap_or(0);
                session.set_stage(SCAN_STAGE, key, signature);
                changed |= before != after;
            }
        }

        for key in session.source_keys(keys) {
            let signature = session.parse_signature(key);
            if session.stage_value(PARSE_STAGE, key) != signature
                || session.records.get(&key).unwrap().fetch(2).is_none()
            {
                let before = session.records.get(&key).map(|record| record.artifact_version(2)).unwrap_or(0);
                Parser::execute(session, &[key]);
                let after = session.records.get(&key).map(|record| record.artifact_version(2)).unwrap_or(0);
                session.set_stage(PARSE_STAGE, key, signature);
                changed |= before != after;
            }
        }

        let targets = session.all_source_keys();
        let resolve = session.resolve_signature(&targets);
        if session.stage_value(RESOLVE_STAGE, 0) != resolve {
            let before = session.resolver.registry.len();
            Resolver::execute(session, &targets);
            let after = session.resolver.registry.len();
            session.set_stage(RESOLVE_STAGE, 0, resolve);
            changed |= before != after;
        }

        let analyze = session.analyze_signature(&targets);
        if session.stage_value(ANALYZE_STAGE, 0) != analyze
            || targets.iter().any(|key| session.records.get(key).unwrap().fetch(3).is_none())
        {
            let before = targets
                .iter()
                .map(|key| session.records.get(key).map(|record| record.artifact_version(3)).unwrap_or(0))
                .sum::<usize>();
            Analyzer::execute(session, &targets);
            let after = targets
                .iter()
                .map(|key| session.records.get(key).map(|record| record.artifact_version(3)).unwrap_or(0))
                .sum::<usize>();
            session.set_stage(ANALYZE_STAGE, 0, analyze);
            changed |= before != after;
        }

        let interpret = session.interpret_signature();
        if let Some(core) = core.as_deref_mut() {
            if session.stage_value(INTERPRET_STAGE, 0) != interpret {
                core.reset();
                Interpreter::process(session, core, &targets);
                session.set_stage(INTERPRET_STAGE, 0, interpret);
                changed = true;
            }
        }

        if !changed || !session.errors.is_empty() {
            break;
        }
    }

    session.report_tokens(keys);
    session.report_elements(keys);
    session.report_analyses(keys);
    session.report_all();
}

pub fn start(directives: Vec<Symbol>, flag: Str) {
    let mut session = Session::create(directives, Vec::new(), flag);
    let mut core = Interpreter::new(1024);

    let mut keys: Vec<_> = session
        .records
        .iter()
        .filter_map(|(&key, record)| (record.kind == RecordKind::Source).then_some(key))
        .collect();
    keys.sort();

    refresh(&mut session, Some(&mut core), &keys);

    let mut terminal = Dialog::new();

    loop {
        let Some(input) = terminal.read("> ").map(Str::from) else {
            break;
        };

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with(':') {
            let mut parts = trimmed.split_whitespace();
            match parts.next().unwrap() {
                ":history" => {
                    for (index, command) in terminal.history.iter().enumerate() {
                        println!("{:3}  {}", index + 1, command);
                    }
                }
                ":cd" => {
                    if let Some(path) = parts.next() {
                        if let Err(error) = set_current_dir(path) {
                            println!("{}", error);
                        }
                    }
                }
                ":ls" => {
                    if let Ok(entries) = read_dir(".") {
                        for entry in entries.flatten() {
                            println!("{}", entry.file_name().to_string_lossy());
                        }
                    }
                }
                ":exit" | ":q" => break,
                _ => {}
            }
            continue;
        }

        let identity = session.records.len() | 0x40000000;
        let location = Location::from("dialog");
        let mut record = Record::new(RecordKind::Source, location);
        record.set_content(input);
        session.records.insert(identity, record);

        refresh(&mut session, None, &[identity]);

        if session.errors.is_empty() {
            if let Ok(Some(result)) = Interpreter::execute_line(&session, &mut core, identity) {
                if !matches!(result, interpreter::Value::Empty) {
                    println!("{:?}", result);
                }
            }
        } else {
            session.records.remove(&identity);
        }
    }
}
