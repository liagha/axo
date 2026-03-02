use {
    crate::{
        data::Str,
        format::Display,
        format::Show,
        checker::{Type, TypeKind},
        internal::{
            timer::Duration,
            platform::PathBuf,
        },
        parser::Element,
        reporter::Error,
        resolver::{Resolution},
        scanner::Token,
    },
    broccli::{xprintln, Color},
};

pub struct Reporter {
    pub verbosity: u8,
}

impl Reporter {
    fn describe_type(typ: &Type) -> String {
        match &typ.kind {
            TypeKind::Integer { bits, signed } => format!("Int(bits={}, signed={})", bits, signed),
            TypeKind::Float { bits } => format!("Float(bits={})", bits),
            TypeKind::Boolean => "Bool".to_string(),
            TypeKind::String => "String".to_string(),
            TypeKind::Char => "Char".to_string(),
            TypeKind::Pointer { to } => {
                format!("Pointer(to={})", Self::describe_type(to))
            }
            TypeKind::Array { member, size } => {
                format!(
                    "Array(member={}, size={})",
                    Self::describe_type(member),
                    size
                )
            }
            TypeKind::Tuple { members } => {
                let members = members
                    .iter()
                    .map(Self::describe_type)
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("Tuple({})", members)
            }
            TypeKind::Infer => "Infer".to_string(),
            TypeKind::Type(item) => format!("Type({})", Self::describe_type(item)),
            TypeKind::Structure(structure) => format!(
                "Structure({})",
                structure.target.as_str().unwrap_or("UnknownStructure"),
            ),
            TypeKind::Enumeration(enumeration) => format!(
                "Enumeration({})",
                enumeration.target.as_str().unwrap_or("UnknownEnumeration"),
            ),
            TypeKind::Method(method) => format!(
                "Method({}, output={})",
                method.target.as_str().unwrap_or("UnknownMethod"),
                Self::describe_type(&method.output),
            ),
        }
    }

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

    pub fn run(&self, target: &PathBuf) {
        if self.is_verbose() {
            xprintln!(
                "Running {}." => Color::Blue,
                format!("`{}`", target.to_string_lossy()) => Color::White
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
                .join("\n");

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

    pub fn resolutions(&self, resolutions: &[Resolution]) {
        if self.is_verbose() {
            let tree = resolutions
                .iter()
                .map(|resolution| Str::from(format!("{:#?}", resolution.analysis.instruction)))
                .collect::<Vec<Str>>()
                .join("\n");

            if !tree.is_empty() {
                xprintln!(
                    "{}{}\n{}" => Color::White,
                    "Analyses" => Color::Cyan,
                    ":" => Color::White,
                    tree.indent(self.verbosity) => Color::White,
                );
                xprintln!();
            } else {
                xprintln!("no Analyses" => Color::Cyan);
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
