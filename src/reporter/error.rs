use {
    crate::{
        data::memory::PhantomData,
        data::{Number, Str},
        format::Display,
        internal::Record,
        reporter::Failure,
        tracker::Span,
    },
    broccli::{Color, TextStyle},
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Error<'error, K>
where
    K: Clone + Display,
{
    pub kind: K,
    pub span: Span,
    pub phantom: PhantomData<&'error ()>,
}

impl<'error, K> Failure for Error<'error, K> where K: Clone + Display {}

impl<'error, K> Error<'error, K>
where
    K: Clone + Display,
{
    pub fn new(kind: K, span: Span) -> Self {
        Self {
            kind,
            span,
            phantom: PhantomData,
        }
    }

    pub fn handle(&self) -> (Str<'error>, Str<'error>) {
        let mut messages = String::new();
        messages.push_str(&self.kind.to_string());
        (Str::from(messages), Str::from(""))
    }

    pub fn handle_record(&self, record: Option<&Record<'error>>) -> (Str<'error>, Str<'error>) {
        let (message, _) = self.handle();
        let Some(record) = record else {
            return (message, Str::from(""));
        };
        let Some(content) = record.content.as_ref() else {
            return (message, Str::from(""));
        };

        let mut details = String::new();
        let start_offset = self.span.start.min(content.len() as u32) as usize;
        let end_offset = self.span.end.min(content.len() as u32) as usize;

        let start_lc = record.offset_to_line_column(start_offset as u32);
        let end_lc = record.offset_to_line_column(end_offset as u32);

        let (start_line, start_column) = start_lc.unwrap_or((0, 0));
        let (end_line, end_column) = end_lc.unwrap_or((0, 0));

        details.push_str(
            &format!(" --> {}:{}:{}\n", record.location, start_line + 1, start_column + 1)
                .colorize(Color::Blue),
        );

        let surround = 3;
        let first = start_line.saturating_sub(surround);
        let total_lines = content.bytes().filter(|b| **b == b'\n').count();
        let last = (end_line + surround).min(total_lines);

        let max = ((total_lines + 1).digit_count() + 2) as usize;

        for line_num in first..=last {
            let line_text = get_line(content, line_num);
            let label = format!("{: >max$}", line_num + 1).colorize(Color::Blue);
            details.push_str(&format!("{}|  {}\n", label, line_text));

            let mark = highlight(
                line_text,
                line_num,
                start_line,
                start_column,
                end_line,
                end_column,
            );
            if !mark.is_empty() {
                details.push_str(&format!(
                    "{}|  {}\n",
                    " ".repeat(max),
                    mark.colorize(Color::Red)
                ));
            }
        }

        (message, Str::from(details))
    }
}

fn get_line(content: &str, line_num: usize) -> &str {
    let mut current_line = 0;
    let mut start = 0;
    for (i, byte) in content.bytes().enumerate() {
        if current_line == line_num {
            start = i;
            break;
        }
        if byte == b'\n' {
            current_line += 1;
            if current_line == line_num {
                start = i + 1;
                break;
            }
        }
    }
    let end = content[start..]
        .find('\n')
        .map(|pos| start + pos)
        .unwrap_or(content.len());
    &content[start..end]
}

fn highlight(
    line: &str,
    line_num: usize,
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
) -> String {
    if line_num < start_line || line_num > end_line {
        return String::new();
    }

    let width = line.chars().count().max(1);

    if start_line == end_line {
        let count = (end_column.saturating_sub(start_column)).max(1);
        return format!(
            "{}{}",
            " ".repeat(start_column.saturating_sub(0)),
            "^".repeat(count)
        );
    }

    if line_num == start_line {
        return format!(
            "{}{}",
            " ".repeat(start_column.saturating_sub(0)),
            "^".repeat(width.saturating_sub(start_column.saturating_sub(0)).max(1))
        );
    }

    if line_num == end_line {
        return "^".repeat(end_column.saturating_sub(0).max(1));
    }

    "^".repeat(width)
}