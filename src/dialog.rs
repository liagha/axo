use crate::{
    analyzer::Analyzer,
    data::{Identity, Str},
    generator::{CraneliftEngine, CraneliftValue},
    internal::{
        platform::{read_dir, set_current_dir, stdin, stdout, IsTerminal, Write},
        session::{ANALYZE_STAGE, PARSE_STAGE, RESOLVE_STAGE, SCAN_STAGE},
        time::Instant,
        Record, RecordKind, Session,
    },
    parser::{Parser, Symbol},
    resolver::Resolver,
    scanner::Scanner,
    tracker::Location,
};
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};

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

            return Some(input);
        }

        let _ = enable_raw_mode();

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

        loop {
            if let Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = read()
            {
                match code {
                    KeyCode::Enter => break,
                    KeyCode::Char('c') | KeyCode::Char('d')
                        if modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        let _ = disable_raw_mode();
                        println!();
                        return None;
                    }
                    KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
                        print!("\x1B[2J\x1B[1;1H");
                        render(&buffer, cursor);
                    }
                    KeyCode::Char('w') if modifiers.contains(KeyModifiers::CONTROL) => {
                        while cursor > 0 && buffer[cursor - 1].is_whitespace() {
                            cursor -= 1;
                            buffer.remove(cursor);
                        }
                        while cursor > 0 && !buffer[cursor - 1].is_whitespace() {
                            cursor -= 1;
                            buffer.remove(cursor);
                        }
                        render(&buffer, cursor);
                    }
                    KeyCode::Home => {
                        cursor = 0;
                        render(&buffer, cursor);
                    }
                    KeyCode::End => {
                        cursor = buffer.len();
                        render(&buffer, cursor);
                    }
                    KeyCode::Delete => {
                        if cursor < buffer.len() {
                            buffer.remove(cursor);
                            render(&buffer, cursor);
                        }
                    }
                    KeyCode::Backspace => {
                        if cursor > 0 {
                            cursor -= 1;
                            buffer.remove(cursor);
                            render(&buffer, cursor);
                        }
                    }
                    KeyCode::Left => {
                        if cursor > 0 {
                            cursor -= 1;
                            render(&buffer, cursor);
                        }
                    }
                    KeyCode::Right => {
                        if cursor < buffer.len() {
                            cursor += 1;
                            render(&buffer, cursor);
                        }
                    }
                    KeyCode::Up => {
                        if index > 0 {
                            index -= 1;
                            buffer = self.history[index].chars().collect();
                            cursor = buffer.len();
                            render(&buffer, cursor);
                        }
                    }
                    KeyCode::Down => {
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
                    KeyCode::Char(character) => {
                        if !character.is_control() {
                            buffer.insert(cursor, character);
                            cursor += 1;
                            render(&buffer, cursor);
                        }
                    }
                    _ => {}
                }
            }
        }

        let _ = disable_raw_mode();
        println!();

        Some(buffer.into_iter().collect())
    }

    pub fn refresh<'a>(
        session: &mut Session<'a>,
        core: Option<&mut CraneliftEngine<'a>>,
        keys: &[Identity],
    ) {
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
                    let before = session
                        .records
                        .get(&key)
                        .map(|record| record.artifact_version(1))
                        .unwrap_or(0);
                    Scanner::execute(session, &[key]);
                    let after = session
                        .records
                        .get(&key)
                        .map(|record| record.artifact_version(1))
                        .unwrap_or(0);
                    session.set_stage(SCAN_STAGE, key, signature);
                    changed |= before != after;
                }
            }

            for key in session.source_keys(keys) {
                let signature = session.parse_signature(key);
                if session.stage_value(PARSE_STAGE, key) != signature
                    || session.records.get(&key).unwrap().fetch(2).is_none()
                {
                    let before = session
                        .records
                        .get(&key)
                        .map(|record| record.artifact_version(2))
                        .unwrap_or(0);
                    Parser::execute(session, &[key]);
                    let after = session
                        .records
                        .get(&key)
                        .map(|record| record.artifact_version(2))
                        .unwrap_or(0);
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
                || targets
                    .iter()
                    .any(|key| session.records.get(key).unwrap().fetch(3).is_none())
            {
                let before = targets
                    .iter()
                    .map(|key| {
                        session
                            .records
                            .get(key)
                            .map(|record| record.artifact_version(3))
                            .unwrap_or(0)
                    })
                    .sum::<usize>();
                Analyzer::execute(session, &targets);
                let after = targets
                    .iter()
                    .map(|key| {
                        session
                            .records
                            .get(key)
                            .map(|record| record.artifact_version(3))
                            .unwrap_or(0)
                    })
                    .sum::<usize>();
                session.set_stage(ANALYZE_STAGE, 0, analyze);
                changed |= before != after;
            }

            let _ = core;

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
        let mut core = CraneliftEngine::new();

        let mut keys: Vec<_> = session
            .records
            .iter()
            .filter_map(|(&key, record)| (record.kind == RecordKind::Source).then_some(key))
            .collect();
        keys.sort();

        Self::refresh(&mut session, None, &keys);

        let mut terminal = Self::new();
        let mut timing = false;

        let is_closed = |text: &str| -> bool {
            let mut braces = 0;
            let mut parens = 0;
            let mut string = false;
            let mut escape = false;

            for item in text.chars() {
                if escape {
                    escape = false;
                    continue;
                }
                match item {
                    '\\' => escape = true,
                    '"' => string = !string,
                    '{' if !string => braces += 1,
                    '}' if !string => braces -= 1,
                    '(' if !string => parens += 1,
                    ')' if !string => parens -= 1,
                    _ => {}
                }
            }

            braces <= 0 && parens <= 0 && !string
        };

        loop {
            let mut content = String::new();
            let mut prompt = "> ";

            loop {
                let Some(input) = terminal.read(prompt) else {
                    return;
                };

                if content.is_empty() {
                    content = input;
                } else {
                    content.push('\n');
                    content.push_str(&input);
                }

                if is_closed(&content) {
                    break;
                }
                prompt = ". ";
            }

            let trimmed = content.trim();
            if trimmed.is_empty() {
                continue;
            }

            terminal.history.push(trimmed.to_string());

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
                    ":clear" => {
                        print!("\x1B[2J\x1B[1;1H");
                        let _ = stdout().flush();
                    }
                    ":time" => {
                        timing = !timing;
                        println!("{}", timing);
                    }
                    ":help" => {
                        println!(":history  :cd  :ls  :clear  :time  :help  :exit  :q");
                    }
                    ":exit" | ":q" => break,
                    _ => {}
                }
                continue;
            }

            let identity = session.records.len() | 0x40000000;
            let location = Location::from("dialog");
            let mut record = Record::new(RecordKind::Source, location);
            record.set_content(Str::from(content));
            session.records.insert(identity, record);

            Self::refresh(&mut session, None, &[identity]);

            if session.errors.is_empty() {
                let start = Instant::now();
                let outcome = core.execute_line(&session, identity);
                let elapsed = start.elapsed();

                if let Ok(Some(result)) = outcome {
                    if !matches!(result, CraneliftValue::Empty) {
                        println!("{:?}", result);
                    }
                }

                if timing {
                    println!("{:?}", elapsed);
                }
            } else {
                session.records.remove(&identity);
            }
        }
    }
}
