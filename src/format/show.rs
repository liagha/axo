use crate::{data::Str, internal::hash::Set};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Verbosity {
    Off,
    #[default]
    Minimal,  // Formerly 0
    Detailed, // Formerly 1
    Debug,    // Formerly 2 and above
}

impl From<u8> for Verbosity {
    fn from(level: u8) -> Self {
        match level {
            0 => Self::Minimal,
            1 => Self::Detailed,
            _ => Self::Debug,
        }
    }
}

impl Verbosity {
    pub fn fallback(self) -> Self {
        match self {
            Self::Off => Self::Off,
            Self::Debug => Self::Detailed,
            Self::Detailed => Self::Minimal,
            Self::Minimal => Self::Minimal,
        }
    }
}

pub trait Show<'show> {
    fn format(&self, verbosity: Verbosity) -> Str<'show>;
    fn indent(&self, verbosity: Verbosity) -> Str<'show> {
        Str::from(
            self.format(verbosity)
                .lines()
                .into_iter()
                .map(|line| format!("    {}", line))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

impl<'show, T: Show<'show>> Show<'show> for &T {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        (*self).format(verbosity)
    }
}

impl<'show, T: Show<'show>> Show<'show> for Box<T> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        (**self).format(verbosity)
    }
}

impl<'show, T: Show<'show>> Show<'show> for Option<T> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => match self {
                Some(value) => value.format(verbosity),
                None => "".into(),
            },
            Verbosity::Detailed => match self {
                Some(value) => format!("Some({})", value.format(verbosity)).into(),
                None => "None".into(),
            },
            Verbosity::Debug => match self {
                Some(value) => format!(
                    "Some(\n{}\n)",
                    value.format(verbosity).indent(verbosity)
                ).into(),
                None => "None".into(),
            },
        }
    }
}

impl<'show, Item: Show<'show>> Show<'show> for [Item] {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => Str::from(
                self.iter()
                    .map(|form| form.format(verbosity).to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
            ),
            Verbosity::Detailed => Str::from(
                format!(
                    "[{}]",
                    self.iter()
                        .map(|form| form.format(verbosity).to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            ),
            Verbosity::Debug => {
                if self.is_empty() {
                    return "[]".into();
                }
                Str::from(format!(
                    "[\n{}\n]",
                    self.iter()
                        .map(|form| form.format(verbosity).indent(verbosity).to_string())
                        .collect::<Vec<String>>()
                        .join(",\n")
                ))
            }
        }
    }
}

impl<'show, Item: Show<'show>> Show<'show> for Vec<Item> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        (&self.as_slice()).format(verbosity)
    }
}

impl<'show, Item: Show<'show>> Show<'show> for Set<Item> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        self.iter().collect::<Vec<&Item>>().format(verbosity)
    }
}

impl<'show> Show<'show> for String {
    fn format(&self, _verbosity: Verbosity) -> Str<'show> {
        Str::from(self.clone())
    }
}

impl<'show> Show<'show> for &'show str {
    fn format(&self, _verbosity: Verbosity) -> Str<'show> {
        Str::from(*self)
    }
}

impl<'show> Show<'show> for Str<'show> {
    fn format(&self, _verbosity: Verbosity) -> Str<'show> {
        *self
    }
}
