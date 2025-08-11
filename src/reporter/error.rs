use {
    super::{
        Hint,
    },
    
    crate::{
        reporter,
        format::{Display, Debug, Formatter, Result},
        tracker::{Span},
        data::{
            Number,
            Scale,
            string::Str,
        },
    },

    broccli::{Color, TextStyle}
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Error<'error, K, N = Str<'error>, H = Str<'error>>
where K: Display, N: Display, H: Display
{
    pub kind: K,
    pub span: Span<'error>,
    pub note: Option<N>,
    pub hints: Vec<Hint<H>>,
}

impl<'error, K, N, H> reporter::Failure for Error<'error, K, N, H>
where K: Display, N: Display, H: Display
{}

impl<'error, K: Display, N: Display, H: Display > Debug for Error<'error, K, N, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (msg, details) = self.format();

        write!(f, "{} \n {}", msg, details)
    }
}

impl<'error, K: Display, N: Display, H: Display > Display for Error<'error, K, N, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (msg, details) = self.format();

        write!(f, "{} \n {}", msg, details)
    }
}

impl<'error, K: Display, N: Display, H: Display> Error<'error, K, N, H> {
    pub fn new(kind: K, span: Span<'error>) -> Self {
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

    pub fn format(&self) -> (Str<'error>, Str<'error>) {
        let mut messages = String::new();
        let mut details = String::new();

        messages.push_str(&format!("{} {}", "error:".colorize(Color::Crimson).bold(), self.kind));

        let source = self.span.start.location.get_value();
        let lines: Vec<Str> = source.lines();

        let start = self.span.start;
        let end = self.span.end;
        let surround = 3;

        let beginning = start.line.saturating_sub(surround);
        let finish = end.line.saturating_add(surround);

        let max = (lines.len().digit_count() + 2) as usize;

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

        (Str::from(messages), Str::from(details))
    }
}