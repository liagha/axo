use {
    crate::{
        data::memory::PhantomData,
        data::{Number, Str},
        format::Display,
        internal::Record,
        reporter::{Failure},
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

impl<'error, K> Failure for Error<'error, K>
where
    K: Clone + Display,
{
}

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
        let Some(rows) = record.rows.as_ref() else {
            return (message, Str::from(""));
        };

        let mut details = String::new();
        let start = self.span.start.min(content.len() as u32) as usize;
        let end = self.span.end.min(content.len() as u32) as usize;
        let (start_line, start_column, _, _) = locate(content, rows, start);
        let (end_line, end_column, _, _) = locate(content, rows, end);
        let surround = 3usize;
        let first = start_line.saturating_sub(surround).max(1);
        let last = (end_line + surround).min(rows.len());
        let max = (rows.len().digit_count() + 2) as usize;

        details.push_str(
            &format!(
                " --> {}:{}:{}\n",
                record.location, start_line, start_column
            )
            .colorize(Color::Blue),
        );

        for number in first..=last {
            let line = line_text(content, rows, number);
            let label = format!("{: ^max$}", number).colorize(Color::Blue);
            details.push_str(&format!("{}|  {}\n", label, line));

            let mark = highlight(line, number, start_line, start_column, end_line, end_column);
            if !mark.is_empty() {
                details.push_str(&format!("{}|  {}\n", " ".repeat(max), mark.colorize(Color::Red)));
            }
        }

        (message, Str::from(details))
    }
}

fn locate(content: &str, rows: &[u32], offset: usize) -> (usize, usize, usize, usize) {
    let line = rows.partition_point(|value| *value as usize <= offset).saturating_sub(1);
    let start = rows[line] as usize;
    let end = if line + 1 < rows.len() {
        rows[line + 1] as usize - 1
    } else {
        content.len()
    };
    let slice = &content[start..offset.min(end)];
    (line + 1, slice.chars().count() + 1, start, end)
}

fn line_text<'a>(content: &'a str, rows: &[u32], number: usize) -> &'a str {
    let start = rows[number - 1] as usize;
    let end = if number < rows.len() {
        rows[number] as usize - 1
    } else {
        content.len()
    };
    &content[start..end]
}

fn highlight(
    line: &str,
    number: usize,
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
) -> String {
    if number < start_line || number > end_line {
        return String::new();
    }

    let width = line.chars().count().max(1);

    if start_line == end_line {
        let count = (end_column.saturating_sub(start_column)).max(1);
        return format!("{}{}", " ".repeat(start_column.saturating_sub(1)), "^".repeat(count));
    }

    if number == start_line {
        return format!(
            "{}{}",
            " ".repeat(start_column.saturating_sub(1)),
            "^".repeat(width.saturating_sub(start_column.saturating_sub(1)).max(1))
        );
    }

    if number == end_line {
        return "^".repeat(end_column.saturating_sub(1).max(1));
    }

    "^".repeat(width)
}
