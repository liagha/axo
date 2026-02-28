use std::path::PathBuf;
use {
    crate::{
        data::Str,
        format::Display,
        format::Show,
        internal::timer::Duration,
        parser::Element,
        reporter::Error,
        resolver::checker::{Type, TypeKind},
        resolver::{Inference, Resolution},
        scanner::Token,
    },
    broccli::{xprintln, Color},
};

pub struct Reporter {
    pub verbosity: bool,
    current_target: Option<String>,
    current_index: usize,
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

    pub fn new(verbosity: bool) -> Self {
        Self {
            verbosity,
            current_target: None,
            current_index: 0,
        }
    }

    pub fn start(&self, stage: &str) {
        if self.verbosity {
            if let Some(ref target) = self.current_target {
                xprintln!(
                    "Started {} {} {}." => Color::Blue,
                    format!("`{}`", stage) => Color::White,
                    target,
                    self.current_index,
                );
            } else {
                xprintln!(
                    "Started {}." => Color::Blue,
                    format!("`{}`", stage) => Color::White,
                );
            }
            xprintln!();
        }
    }

    pub fn generate(&self, kind: &str, target: &PathBuf) {
        if self.verbosity {
            xprintln!(
                "Generated {} {}." => Color::Green,
                format!("({})", kind) => Color::White,
                format!("`{}`", target.to_string_lossy()) => Color::White
            );

            xprintln!();
        }
    }

    pub fn run(&self, target: &PathBuf) {
        if self.verbosity {
            xprintln!(
                "Running {}." => Color::Blue,
                format!("`{}`", target.to_string_lossy()) => Color::White
            );

            xprintln!();
        }
    }

    pub fn finish(&self, stage: &str, duration: Duration, error_count: usize) {
        if self.verbosity {
            let target_info = if let Some(ref target) = self.current_target {
                format!(" {} {}", target, self.current_index)
            } else {
                String::new()
            };

            if error_count > 0 {
                xprintln!(
                    "Finished {}{} {}s with {} {}." => Color::Green,
                    format!("`{}` in", stage) => Color::White,
                    target_info,
                    duration.as_secs_f64(),
                    error_count => Color::Red,
                    "errors" => Color::Red,
                );
            } else {
                xprintln!(
                    "Finished {}{} {}s." => Color::Green,
                    format!("`{}` in", stage) => Color::White,
                    target_info,
                    duration.as_secs_f64(),
                );
            }
            xprintln!();
        }
    }

    pub fn tokens(&self, tokens: &[Token]) {
        if self.verbosity {
            let body = tokens
                .iter()
                .map(|token| Str::from(format!("{:?}", token)))
                .collect::<Vec<Str>>()
                .join("\n");
            xprintln!(
                "{}{}\n{}" => Color::White,
                "Tokens" => Color::Blue,
                ":" => Color::White,
                body.indent() => Color::White
            );
            xprintln!();
        }
    }

    pub fn elements(&self, elements: &[Element]) {
        if self.verbosity {
            let tree = elements
                .iter()
                .map(|element| Str::from(format!("{:#?}", element)))
                .collect::<Vec<Str>>()
                .join("\n");

            if !tree.is_empty() {
                xprintln!(
                    "{}{}\n{}" => Color::White,
                    "Elements" => Color::Cyan,
                    ":" => Color::White,
                    tree.indent() => Color::White
                );
                xprintln!();
            }
        }
    }

    pub fn symbols<'reporter>(
        &self,
        symbols: &[(
            crate::parser::Symbol<'reporter>,
            Option<Inference<'reporter>>,
        )],
    ) {
        if self.verbosity {
            let mut tree = String::new();
            for (symbol, maybe_inference) in symbols {
                tree.push_str(&format!("{:#?}", symbol));
                if let Some(inference) = maybe_inference {
                    let declared = inference
                        .declared
                        .as_ref()
                        .map(Self::describe_type)
                        .unwrap_or_else(|| "None".to_string());
                    let inferred = inference
                        .inferred
                        .as_ref()
                        .map(Self::describe_type)
                        .unwrap_or_else(|| "None".to_string());
                    tree.push_str(&format!(
                        "\n  Inference(declared: {}, inferred: {}).",
                        declared, inferred
                    ));
                }
                tree.push('\n');
            }

            if !tree.is_empty() {
                xprintln!(
                    "{}{}\n{}" => Color::White,
                    "Symbols" => Color::Blue,
                    ":" => Color::White,
                    Str::from(tree).indent() => Color::White,
                );
                xprintln!();
            }
        }
    }

    pub fn resolutions(&self, resolutions: &[Resolution]) {
        if self.verbosity {
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
                    tree.indent() => Color::White,
                );
                xprintln!();
            } else {
                xprintln!("no Analyses" => Color::Cyan);
                xprintln!();
            }
        }
    }

    pub fn errors<K, H>(&self, errors: &[Error<K, H>])
    where
        K: Clone + Display,
        H: Clone + Display,
    {
        if !errors.is_empty() {
            for error in errors {
                let (message, details) = error.format();
                xprintln!(
                    "{}\n{}" => Color::Red,
                    message => Color::White,
                    details => Color::White
                );
                xprintln!();
            }
        }
    }

    pub fn set_current(&mut self, target: String) {
        self.current_index += 1;
        self.current_target = Some(target);
    }

    pub fn clear_current(&mut self) {
        self.current_target = None;
    }
}
