use crate::file::read_to_string;
use broccli::{Color, TextStyle};
use crate::axo_errors::hint::Hint;
use crate::axo_span::Span;
use crate::format::{Display, Debug, Formatter, Result};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Error<K, N = String, H = String> where K: Display, N: Display, H: Display {
    pub kind: K,
    pub span: Span,
    pub note: Option<N>,
    pub hints: Vec<Hint<H>>,
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
        let source_code = read_to_string(self.span.start.file.clone()).unwrap_or_default();
        let lines: Vec<&str> = source_code.lines().collect();
        let mut messages = String::new();
        let mut details = String::new();

        messages.push_str(&format!("{} {}", "error:".colorize(Color::Crimson).bold(), self.kind));

        let line_start = self.span.start.line;
        let column_start = self.span.start.column;
        let line_end = self.span.end.line;
        let column_end = self.span.end.column;

        details.push_str(&format!(" --> {}:{}:{}\n",
                                  self.span.start.file.display(),
                                  line_start,
                                  column_start
        ).colorize(Color::Blue));

        let max_line_number = line_end.max(line_start).max(1);
        let line_number_width = max_line_number.to_string().len();

        let start_line = line_start.saturating_sub(3).max(1);
        let end_line = (line_end + 3).min(lines.len()).max(1);

        for line_idx in start_line..=end_line {
            let line_content_idx = line_idx.saturating_sub(1);
            if line_content_idx >= lines.len() { break; }

            let line_content = lines[line_content_idx];
            let line_num_str = if line_content.is_empty() {
                " ".repeat(line_number_width)
            } else {
                line_idx.to_string()
            };

            details.push_str(&format!("{:>width$} | {}\n",
                                      line_num_str.colorize(Color::Blue),
                                      line_content,
                                      width = line_number_width
            ));

            if line_idx >= line_start && line_idx <= line_end {
                let line_length = line_content.chars().count();
                let start_col = if line_idx == line_start {
                    column_start.saturating_sub(1).min(line_length)
                } else { 0 };
                let end_col = if line_idx == line_end {
                    column_end.saturating_sub(1).min(line_length)
                } else { line_length };

                let caret_count = if start_col <= end_col {
                    end_col.saturating_sub(start_col)
                } else {
                    1
                };

                let underline = format!("{:width$}{}",
                                        "",
                                        "^".repeat(caret_count),
                                        width = start_col
                );

                details.push_str(&format!("{:>width$} | {}\n",
                                          " ".repeat(line_number_width),
                                          underline.colorize(Color::Red).bold(),
                                          width = line_number_width
                ));
            }
        }

        if let Some(note) = &self.note {
            messages.push_str(&format!("note: {}\n", note.to_string().colorize(Color::Green)));
        }

        for hint in &self.hints {
            details.push_str(&format!("\n{}{}\n", "hint: ".colorize(Color::Blue), hint.message.to_string().bold()));

            use crate::axo_errors::hint::Action::*;

            for action in &hint.action {
                match action {
                    Add(text, span) => {
                        let line_idx = span.start.line;
                        let col_idx = span.start.column;
                        if let Some(line) = lines.get(line_idx.saturating_sub(1)) {
                            let mut rendered = String::new();
                            let (before, after) = line.split_at(col_idx.saturating_sub(1));
                            rendered.push_str(before);
                            rendered.push_str(&text.colorize(Color::Green).bold().to_string());
                            rendered.push_str(after);

                            details.push_str(&format!(
                                "{:>width$} | {}\n",
                                line_idx.to_string().colorize(Color::Blue),
                                rendered,
                                width = line_number_width
                            ));

                            details.push_str(&format!(
                                "{:>width$} | {:>col$}{} {}\n",
                                "",
                                "",
                                "^".colorize(Color::Green).bold(),
                                format!("insert `{}`", text).colorize(Color::Green),
                                width = line_number_width,
                                col = col_idx.saturating_sub(1),
                            ));
                        }
                    }

                    Remove(span) => {
                        let line_idx = span.start.line;
                        let col_start = span.start.column;
                        let col_end = span.end.column;
                        if let Some(line) = lines.get(line_idx.saturating_sub(1)) {
                            let before = &line[..col_start.saturating_sub(1)];
                            let target = &line[col_start.saturating_sub(1)..col_end.saturating_sub(1)];
                            let after = &line[col_end.saturating_sub(1)..];

                            let mut rendered = String::new();
                            rendered.push_str(before);
                            rendered.push_str(&target.colorize(Color::Red).bold().to_string());
                            rendered.push_str(after);

                            details.push_str(&format!(
                                "{:>width$} | {}\n",
                                line_idx.to_string().colorize(Color::Blue),
                                rendered,
                                width = line_number_width
                            ));

                            details.push_str(&format!(
                                "{:>width$} | {:>col$}{} {}\n",
                                "",
                                "",
                                "^".repeat(target.len()).colorize(Color::Red).bold(),
                                "remove this".colorize(Color::Red),
                                width = line_number_width,
                                col = col_start.saturating_sub(1),
                            ));
                        }
                    }

                    Replace(text, span) => {
                        let line_idx = span.start.line;
                        let col_start = span.start.column;
                        let col_end = span.end.column;
                        if let Some(line) = lines.get(line_idx.saturating_sub(1)) {
                            let before = &line[..col_start.saturating_sub(1)];
                            let after = &line[col_end.saturating_sub(1)..];

                            let mut rendered = String::new();
                            rendered.push_str(before);
                            rendered.push_str(&text.colorize(Color::Blue).bold().to_string());
                            rendered.push_str(after);

                            details.push_str(&format!(
                                "{:>width$} | {}\n",
                                line_idx.to_string().colorize(Color::Blue),
                                rendered,
                                width = line_number_width
                            ));

                            details.push_str(&format!(
                                "{:>width$} | {:>col$}{} {}\n",
                                "",
                                "",
                                "^".repeat(text.len().max(1)).colorize(Color::Blue).bold(),
                                format!("replace with `{}`", text).colorize(Color::Blue),
                                width = line_number_width,
                                col = col_start.saturating_sub(1),
                            ));
                        }
                    }

                    _ => {
                        let (line_num, content, prefix, color) = match action {
                            AddLine(text, line) => (*line, text, "+", Color::Green),
                            RemoveLine(line) => (*line, &"<line removed>".to_string(), "-", Color::Red),
                            ReplaceLine(text, line) => (*line, text, "~", Color::Blue),
                            _ => continue,
                        };

                        details.push_str(&format!(
                            "{:>width$} | {}\n",
                            line_num.to_string().colorize(Color::Blue),
                            content.colorize(color).bold(),
                            width = line_number_width
                        ));

                        details.push_str(&format!(
                            "{:>width$} | {} {}\n",
                            "",
                            prefix.colorize(color).bold(),
                            match prefix {
                                "+" => "added line",
                                "-" => "removed line",
                                "~" => "replaced line",
                                _ => "",
                            }.colorize(color),
                            width = line_number_width
                        ));
                    }
                }
            }
        }

        (messages, details)
    }
}