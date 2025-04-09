use std::fs::read_to_string;
use broccli::{Color, TextStyle};
use crate::axo_lexer::Span;
use crate::axo_parser::Context;

#[derive(Debug, Clone)]
pub struct Error<T> where T: core::fmt::Display {
    pub kind: T,
    pub span: Span,
    pub context: Option<Context>,
    pub help: Option<String>,
}

impl<T: core::fmt::Display> core::fmt::Display for Error<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (msg, details) = self.format();

        write!(f, "{} \n {}", msg, details)
    }
}

impl<T: core::fmt::Display> Error<T> {
    pub fn new(kind: T, span: Span) -> Self {
        Self {
            kind,
            span,
            context: None,
            help: None,
        }
    }

    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn format(&self) -> (String, String) {
        let source_code = read_to_string(self.span.file.clone()).unwrap();
        let lines: Vec<&str> = source_code.lines().collect();
        let mut messages = String::new();
        let mut details = String::new();

        messages.push_str(&format!("error: {}", self.kind.to_string().colorize(Color::Red).bold()));

        let (line_start, column_start) = self.span.start;
        let (line_end, column_end) = self.span.end;

        details.push_str(&format!(" --> {}:{}:{}\n",
                                  self.span.file.display(),
                                  line_start,
                                  column_start
        ).colorize(Color::Blue));

        if let Some(ctx) = &self.context {
            messages.push_str(&format!(" note: {}\n", ctx.describe_chain().colorize(Color::Blue)));
        }

        // Calculate maximum line number width safely
        let max_line_number = line_end.max(line_start).max(1); // Ensure at least 1
        let line_number_width = max_line_number.to_string().len();

        // Calculate bounds safely
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
                    end_col.saturating_sub(start_col) + 1
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

        if let Some(help) = &self.help {
            messages.push_str(&format!("help: {}\n", help.colorize(Color::Green)));
        }

        (messages, details)
    }
}
