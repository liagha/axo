use crate::{data::Str, internal::hash::Set};

pub trait Show<'show> {
    type Verbosity;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show>;
    fn indent(&self, verbosity: Self::Verbosity) -> Str<'show> {
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

impl<'show, T: Show<'show, Verbosity=u8>> Show<'show> for &T  {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        (*self).format(verbosity)
    }
}

impl<'show, T: Show<'show, Verbosity=u8>> Show<'show> for Box<T>  {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        (**self).format(verbosity)
    }
}

impl<'show, T: Show<'show, Verbosity=u8>> Show<'show> for Option<T> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match self {
            Some(value) => {
                format!("Some({})", value.format(verbosity))
            }

            None => "None".to_string(),
        }.into()
    }
}

impl<'show, Item: Show<'show, Verbosity=u8>> Show<'show> for [Item] {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => {
                Str::from(
                    self.iter()
                        .map(|form| Str::from(form.format(verbosity)))
                        .collect::<Vec<Str>>()
                        .join(", "),
                )
            }

            1 => {
                Str::from(
                    self.iter()
                        .map(|form| Str::from(form.format(verbosity)))
                        .collect::<Vec<Str>>()
                        .join(",\n"),
                )
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}

impl<'show, Item: Show<'show, Verbosity=u8>> Show<'show> for Vec<Item> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        (&self.as_slice()).format(verbosity)
    }
}

impl<'show, Item: Show<'show, Verbosity=u8>> Show<'show> for Set<Item> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        self.iter().collect::<Vec<&Item>>().format(verbosity)
    }
}

impl<'show> Show<'show> for String {
    type Verbosity = u8;

    fn format(&self, _verbosity: Self::Verbosity) -> Str<'show> {
        Str::from(self.clone())
    }
}

impl<'show> Show<'show> for &'show str {
    type Verbosity = u8;

    fn format(&self, _verbosity: Self::Verbosity) -> Str<'show> {
        Str::from(*self)
    }
}

impl<'show> Show<'show> for Str<'show> {
    type Verbosity = u8;

    fn format(&self, _verbosity: Self::Verbosity) -> Str<'show> {
        *self
    }
}
