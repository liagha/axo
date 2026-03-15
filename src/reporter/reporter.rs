use {
    crate::{
        data::Str,
        format::Display,
        format::Show,
        internal::{
            timer::Duration,
            platform::PathBuf,
        },
        parser::Element,
        reporter::Error,
        scanner::Token,
    },
    broccli::{xprintln, Color},
};
use crate::analyzer::Analysis;

pub struct Reporter {
    pub verbosity: u8,
}

impl Reporter {
    pub fn new(verbosity: u8) -> Self {
        Self {
            verbosity,
        }
    }
    
    pub fn is_verbose(&self) -> bool {
        self.verbosity > 0
    }

    pub fn start(&self, stage: &str) {
        if self.is_verbose() {
            xprintln!(
                "Started {}." => Color::Blue,
                format!("`{}`", stage) => Color::White,
            );
            xprintln!();
        }
    }

    pub fn generate(&self, kind: &str, target: &PathBuf) {
        if self.is_verbose() {
            xprintln!(
                "Generated {} {}." => Color::Green,
                format!("({})", kind) => Color::White,
                format!("`{}`", target.to_string_lossy()) => Color::White
            );

            xprintln!();
        }
    }

    pub fn run(&self, target: String) {
        if self.is_verbose() {
            xprintln!(
                "Running {}." => Color::Blue,
                format!("`{}`", target) => Color::White
            );

            xprintln!();
        }
    }

    pub fn finish(&self, stage: &str, duration: Duration) {
        if self.is_verbose() {
            xprintln!(
                "Finished {} {}s." => Color::Green,
                format!("`{}` in", stage) => Color::White,
                duration.as_secs_f64(),
            );
            
            xprintln!();
        }
    }

    pub fn tokens(&self, tokens: &[Token]) {
        if self.is_verbose() {
            let tree = tokens
                .iter()
                .map(|token| Str::from(format!("{}", token.format(self.verbosity))))
                .collect::<Vec<Str>>()
                .join(", ");

            if !tree.is_empty() {
                xprintln!(
                    "{}{}\n{}" => Color::White,
                    "Tokens" => Color::Cyan,
                    ":" => Color::White,
                    tree.indent(self.verbosity) => Color::White
                );

                xprintln!();
            }
        }
    }

    pub fn elements(&self, elements: &[Element]) {
        if self.is_verbose() {
            let tree = elements
                .iter()
                .map(|element| Str::from(format!("{}", element.format(self.verbosity))))
                .collect::<Vec<Str>>()
                .join("\n");

            if !tree.is_empty() {
                xprintln!(
                    "{}{}\n{}" => Color::White,
                    "Elements" => Color::Cyan,
                    ":" => Color::White,
                    tree.indent(self.verbosity) => Color::White
                );
                xprintln!();
            }
        }
    }

    pub fn symbols<'reporter>(
        &self,
        symbols: &[
            crate::parser::Symbol<'reporter>
        ],
    ) {
        if self.is_verbose() {
            let mut tree = String::new();
            for symbol in symbols {
                tree.push_str(&format!("{}", symbol.format(self.verbosity)));
                tree.push('\n');
            }

            if !tree.is_empty() {
                xprintln!(
                    "{}{}\n{}" => Color::White,
                    "Symbols" => Color::Blue,
                    ":" => Color::White,
                    Str::from(tree).indent(self.verbosity) => Color::White,
                );
                xprintln!();
            }
        }
    }

    pub fn analysis<'reporter>(
        &self,
        analysis: &[
            Analysis<'reporter>
        ],
    ) {
        if self.is_verbose() {
            let mut tree = String::new();

            for analysis in analysis {
                tree.push_str(&format!("{}", analysis.format(self.verbosity)));
                tree.push('\n');
            }

            if !tree.is_empty() {
                xprintln!(
                    "{}{}\n{}" => Color::White,
                    "Analysis" => Color::Blue,
                    ":" => Color::White,
                    Str::from(tree).indent(self.verbosity) => Color::White,
                );
                xprintln!();
            }
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
            self.error(&error);
        }
    }
}
