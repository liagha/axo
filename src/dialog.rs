use axo::{
    data::{
        Str, from_utf8,
    },
    internal::{
        platform::{
            set_current_dir,
            read_dir, stdin, stdout,
            Read, Write,
            Command,
        },
        RecordKind,
        Record
    },
    interpreter,
    interpreter::Interpreter,
    parser::Symbol,
    tracker::Location
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
        let _ = Command::new("stty").args(["raw", "-echo"]).status();

        let mut input = stdin();
        let mut buffer: Vec<char> = Vec::new();
        let mut cursor = 0;
        let mut index = self.history.len();

        let render = |characters: &[char], position: usize| {
            let string: String = characters.iter().collect();
            print!("\r\x1b[2K{}{}", prompt, string);
            if characters.len() > position {
                print!("\x1b[{}D", characters.len() - position);
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

pub fn start(bare: bool, directives: Vec<Symbol>, flag_content: Str) {
    let mut session = crate::create(bare, directives, Vec::new(), flag_content);
    let mut core = Interpreter::new(1024);

    let mut keys: Vec<_> = session
        .records
        .iter()
        .filter_map(|(&key, record)| (record.kind == RecordKind::Source).then_some(key))
        .collect();
    keys.sort();

    crate::run(&mut session, &mut core, &keys);

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

        let location = Location::from("dialog");
        let mut record = Record::new(RecordKind::Source, location);
        record.set_content(input);

        let identity = session.records.len() | 0x40000000;
        session.records.insert(identity, record);

        crate::run(&mut session, &mut core, &[identity]);

        if session.errors.is_empty() {
            if let Some(result) = core.extract() {
                if !matches!(result, interpreter::Value::Empty) {
                    println!("{:?}", result);
                }
            }
        }
    }
}