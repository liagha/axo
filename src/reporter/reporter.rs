use {
    crate::{
        analyzer::Analysis,
        data::Str,
        format::{Display, Show, Verbosity},
        internal::{platform::PathBuf, timer::Duration},
        parser::{Element, Symbol},
        reporter::Error,
        scanner::Token,
    },
    broccli::{xprintln, Color},
};

pub struct Reporter {
    pub verbosity: Verbosity,
}

impl Reporter {
    pub fn new(verbosity: u8) -> Self {
        Self {
            verbosity: verbosity.into(),
        }
    }

    pub fn active(&self) -> bool {
        self.verbosity != Verbosity::Off
    }

    pub fn start(&self, stage: &str) {
        if self.active() {
            xprintln!(
                "Started {}." => Color::Blue,
                format!("`{}`", stage) => Color::White
            );
            xprintln!();
        }
    }

    pub fn finish(&self, stage: &str, duration: Duration, count: usize) {
        if self.active() {
            let suffix = if count > 0 {
                format!(" ({} errors)", count)
            } else {
                String::new()
            };

            xprintln!(
                "Finished {} {}s{}" => Color::Green,
                format!("`{}` in", stage) => Color::White,
                duration.as_secs_f64(),
                suffix => Color::Red
            );
            xprintln!();
        }
    }

    pub fn generate(&self, kind: &str, target: &PathBuf) {
        if self.active() {
            xprintln!(
                "Generated {} {}." => Color::Green,
                format!("({})", kind) => Color::White,
                format!("`{}`", target.to_string_lossy()) => Color::White
            );
            xprintln!();
        }
    }

    pub fn run(&self, target: String) {
        if self.active() {
            xprintln!(
                "Running {}." => Color::Blue,
                format!("`{}`", target) => Color::White
            );
            xprintln!();
        }
    }

    pub fn section(&self, head: &str, color: Color, body: String) {
        if self.active() && !body.is_empty() {
            xprintln!(
                "{}{}\n{}" => Color::White,
                head => color,
                ":" => Color::White,
                Str::from(body).indent(self.verbosity) => Color::White
            );
            xprintln!();
        }
    }

    pub fn tokens(&self, tokens: &[Token]) {
        let body = tokens
            .iter()
            .map(|token| format!("{}", token.format(self.verbosity)))
            .collect::<Vec<String>>()
            .join(", ");
        self.section("Tokens", Color::Cyan, body);
    }

    pub fn elements(&self, elements: &[Element]) {
        let body = elements
            .iter()
            .map(|element| format!("{}", element.format(self.verbosity)))
            .collect::<Vec<String>>()
            .join("\n");
        self.section("Elements", Color::Cyan, body);
    }

    pub fn symbols(&self, symbols: &[Symbol]) {
        let body = symbols
            .iter()
            .map(|symbol| format!("{}", symbol.format(self.verbosity)))
            .collect::<Vec<String>>()
            .join("\n");
        self.section("Symbols", Color::Blue, body);
    }

    pub fn analysis(&self, analysis: &[Analysis]) {
        let body = analysis
            .iter()
            .map(|item| format!("{}", item.format(self.verbosity)))
            .collect::<Vec<String>>()
            .join("\n");
        self.section("Analysis", Color::Blue, body);
    }

    pub fn order(&self, sequence: &[String]) {
        if self.active() && !sequence.is_empty() {
            xprintln!(
                "{}{} {}" => Color::White,
                "Order" => Color::Magenta,
                ":" => Color::White,
                sequence.join(" -> ") => Color::White
            );
            xprintln!();
        }
    }

    pub fn error<K, H>(&self, error: &Error<K, H>)
    where
        K: Clone + Display,
        H: Clone + Display,
    {
        let (message, details) = error.handle();
        xprintln!(
            "{}\n{}" => Color::Red,
            message => Color::White,
            details => Color::White
        );
        xprintln!();
    }

    pub fn errors<K, H>(&self, errors: &[Error<K, H>])
    where
        K: Clone + Display,
        H: Clone + Display,
    {
        for error in errors {
            self.error(error);
        }
    }
}
