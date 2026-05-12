// src/dialog.rs

use crate::{
    analyzer::Analyzer,
    data::{Identity, Str},
    internal::{
        platform::{read_dir, set_current_dir, stdin, stdout, IsTerminal, Write},
        time::Instant,
        Record, RecordKind, Session,
    },
    parser::Parser,
    resolver::Resolver,
    scanner::Scanner,
    tracker::Location,
};

#[cfg(feature = "interpreter")]
use crate::emitter::{Engine, Value};

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

    pub fn refresh(session: &mut Session, keys: &[Identity]) {
        session.errors.clear();
        if !session.prepare() {
            session.report_all();
            return;
        }

        Scanner::execute(session, keys);
        if !session.errors.is_empty() {
            session.report_all();
            return;
        }

        Parser::execute(session, keys);
        if !session.errors.is_empty() {
            session.report_all();
            return;
        }

        Resolver::execute(session, keys);
        if !session.errors.is_empty() {
            session.report_all();
            return;
        }

        Analyzer::execute(session, keys);
        session.report_tokens(keys);
        session.report_elements(keys);
        session.report_analyses(keys);
        session.report_all();
    }

    pub fn start(mut session: Session) {
        let mut keys: Vec<_> = session
            .records
            .iter()
            .filter_map(|(&key, record)| (record.kind == RecordKind::Source).then_some(key))
            .collect();
        keys.sort();

        Self::refresh(&mut session, &keys);

        #[cfg(feature = "interpreter")]
        let mut engine = Engine::new();

        #[cfg(feature = "interpreter")]
        {
            let base_keys = session.all_source_keys();
            for key in &base_keys {
                if let Some(analyses) = session.records.get(key).and_then(|r| {
                    if let Some(crate::internal::Artifact::Analyses(a)) = r.fetch(3) {
                        Some(a.clone())
                    } else {
                        None
                    }
                }) {
                    let _ = engine.execute(analyses);
                }
            }
        }

        let mut terminal = Self::new();
        let mut timing = false;

        let is_closed = |text: &str| -> bool {
            let mut braces = 0i32;
            let mut parens = 0i32;
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

            Self::refresh(&mut session, &[identity]);

            if session.errors.is_empty() {
                let start = Instant::now();

                #[cfg(feature = "interpreter")]
                match session.execute_line(&mut engine, identity) {
                    Ok(Some(result)) if !matches!(result, Value::Void) => {
                        println!("{:?}", result);
                    }
                    Ok(_) => {}
                    Err(error) => {
                        session.report_error(&error);
                    }
                }

                let elapsed = start.elapsed();

                if timing {
                    println!("{:?}", elapsed);
                }
            } else {
                session.records.remove(&identity);
            }
        }
    }
}
