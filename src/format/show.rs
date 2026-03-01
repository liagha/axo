use crate::{data::Str, format::Display, internal::hash::Set};

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

impl<'show, Item: Show<'show, Verbosity = u8>> Show<'show> for [Item] {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        Str::from(
            self.iter()
                .map(|form| Str::from(form.format(verbosity)))
                .collect::<Vec<Str>>()
                .join(", "),
        )
    }
}

impl<'show, Item: Display> Show<'show> for Set<Item> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        Str::from(
            self.iter()
                .map(|form| Str::from(form.to_string()))
                .collect::<Vec<Str>>()
                .join(", "),
        )
    }
}

impl<'show> Show<'show> for String {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        Str::from(self.clone())
    }
}

impl<'show> Show<'show> for &'show str {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        Str::from(*self)
    }
}

impl<'show> Show<'show> for Str<'show> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        *self
    }
}
