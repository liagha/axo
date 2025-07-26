use {
    super::{
        Hint,
    },
    
    crate::{
        format::{Display, Debug, Formatter, Result},
        file::read_to_string,
        axo_cursor::{Span, Location},
    },

    broccli::{Color, TextStyle}
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Error<K, N = String, H = String> 
where K: Display, N: Display, H: Display 
{
    pub kind: K,
    pub span: Span,
    pub note: Option<N>,
    pub hints: Vec<Hint<H>>,
}

impl<K, N, H> crate::error::Error for Error<K, N, H>
where K: Display, N: Display, H: Display 
{}

impl<K: Display, N: Display, H: Display > Debug for Error<K, N, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (msg, details) = self.format();

        write!(f, "{} \n {}", msg, details)
    }
}

impl<K: Display, N: Display, H: Display > Display for Error<K, N, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (msg, details) = self.format();

        write!(f, "{} \n {}", msg, details)
    }
}

impl<K: Display, N: Display, H: Display> Error<K, N, H> {
    pub fn new(kind: K, span: Span) -> Self {
        Self {
            kind,
            span,
            note: None,
            hints: vec![],
        }
    }

    pub fn with_help(mut self, note: impl Into<N>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn format(&self) -> (String, String) {
        fn count_digits(mut num: usize) -> usize {
            if num == 0 {
                return 1;
            }
            let mut count = 0;
            while num != 0 {
                num /= 10;
                count += 1;
            }
            count
        }

        let mut messages = String::new();
        let mut details = String::new();

        messages.push_str(&format!("{} {}", "error:".colorize(Color::Crimson).bold(), self.kind));

        if let Location::File(path) = self.span.start.location {
            let source = read_to_string(path).unwrap_or_default();
            let lines: Vec<&str> = source.lines().collect();

            let start = self.span.start;
            let end = self.span.end;
            let surround = 3;

            let beginning = start.line.saturating_sub(surround);
            let finish = end.line.saturating_add(surround);

            let max = count_digits(lines.len()) + 2;

            details.push_str(&format!(" --> {}\n", self.span).colorize(Color::Blue));

            for index in beginning..=finish {
                if let Some(line) = lines.get(index) {
                    let index = index + 1;
                    let identifier = format!("{: ^max$}", index).colorize(Color::Blue);

                    details.push_str(&format!("{}|  {}\n", identifier, line));

                    let highlighter = "^".colorize(Color::Red);

                    if start.line == end.line {
                        if index == start.line {
                            if start.column == end.column {
                                let highlight = format!("{}{}", " ".repeat(start.column - 1), highlighter);
                                details.push_str(&format!("{}|  {}\n", " ".repeat(max), highlight));
                            } else {
                                let highlight = format!("{}{}", " ".repeat(start.column - 1), highlighter.repeat(end.column - start.column));
                                details.push_str(&format!("{}|  {}\n", " ".repeat(max), highlight));
                            }
                        }
                    } else {
                        let terminus = line.len();

                        let highlight = if index == start.line {
                            format!("{}{}", " ".repeat(start.column - 1), highlighter.repeat(terminus.saturating_sub(start.column) + 1))
                        } else if start.line < index && index < end.line {
                            format!("{}", highlighter.repeat(terminus))
                        } else if index == end.line {
                            format!("{}", highlighter.repeat(end.column - 1))
                        } else {
                            "".to_string()
                        };

                        if !highlight.is_empty() {
                            details.push_str(&format!("{}|  {}\n", " ".repeat(max), highlight));
                        }
                    }
                }
            }

            for hint in &self.hints {
                details.push_str(format!("{}: {}", "hint".colorize(Color::Blue), hint.message).as_str());
            }
        } else {
            details.push_str("invalid location!")
        }

        (messages, details)
    }
}