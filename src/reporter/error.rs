use {
    crate::{
        data::{Number, Str},
        format::Display,
        reporter::{Failure, Hint},
        tracker::Span,
    },
    broccli::{Color, TextStyle},
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Error<'error, K, H = Str<'error>>
where
    K: Clone + Display,
    H: Clone + Display,
{
    pub kind: K,
    pub span: Span<'error>,
    pub hints: Vec<Hint<H>>,
}

impl<'error, K, H> Failure for Error<'error, K, H>
where
    K: Clone + Display,
    H: Clone + Display,
{
}

impl<'error, K, H> Error<'error, K, H>
where
    K: Clone + Display,
    H: Clone + Display,
{
    pub fn new(kind: K, span: Span<'error>) -> Self {
        Self {
            kind,
            span,
            hints: Vec::new(),
        }
    }

    pub fn handle(&self) -> (Str<'error>, Str<'error>) {
        let mut messages = String::new();
        let mut details = String::new();

        messages.push_str(&self.kind.to_string());

        match self.span.location.get_value() {
            Ok(content) => {
                let lines: Vec<Str> = content.lines();

                let start_line = self.span.start_line;
                let end_line = self.span.end_line;
                let start_column = self.span.start_column;
                let end_column = self.span.end_column;
                let surround = 3;

                let beginning = start_line.saturating_sub(surround);
                let finish = end_line.saturating_add(surround);

                let max = (lines.len().digit_count() + 2) as usize;

                details.push_str(&format!(" --> {}\n", self.span).colorize(Color::Blue));

                for index in beginning..=finish {
                    if let Some(line) = lines.get(index) {
                        let index = index + 1;
                        let identifier = format!("{: ^max$}", index).colorize(Color::Blue);

                        details.push_str(&format!("{}|  {}\n", identifier, line));

                        let highlighter = "^".colorize(Color::Red);

                        if start_line == end_line {
                            if index == start_line {
                                if start_column == end_column {
                                    let highlight =
                                        format!("{}{}", " ".repeat(start_column - 1), highlighter);
                                    details.push_str(&format!(
                                        "{}|  {}\n",
                                        " ".repeat(max),
                                        highlight
                                    ));
                                } else {
                                    let highlight = format!(
                                        "{}{}",
                                        " ".repeat(start_column - 1),
                                        highlighter.repeat(end_column - start_column)
                                    );
                                    details.push_str(&format!(
                                        "{}|  {}\n",
                                        " ".repeat(max),
                                        highlight
                                    ));
                                }
                            }
                        } else {
                            let terminus = line.len();

                            let highlight = if index == start_line {
                                format!(
                                    "{}{}",
                                    " ".repeat(start_column - 1),
                                    highlighter.repeat(terminus.saturating_sub(start_column) + 1)
                                )
                            } else if start_line < index && index < end_line {
                                format!("{}", highlighter.repeat(terminus))
                            } else if index == end_line {
                                format!("{}", highlighter.repeat(end_column - 1))
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
                    details.push_str(
                        format!("{}: {}", "hint".colorize(Color::Blue), hint.message).as_str(),
                    );
                }

                (Str::from(messages), Str::from(details))
            }

            Err(_error) => (Str::from(messages), Str::from("")),
        }
    }
}
