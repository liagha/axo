use {
    crate::{
        data::Str,
        internal::hash::Set,
        format::Display,
    }
};

pub trait Show<'show> {
    fn format(&self) -> Str<'show>;
    fn indent(&self) -> Str<'show> {
        Str::from(
            self.format().lines()
                .into_iter()
                .map(|line| format!("    {}", line))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl<'show, Item: Display> Show<'show> for [Item] {
    fn format(&self) -> Str<'show> {
        Str::from(
            self.iter()
                .map(|form| Str::from(form.to_string()))
                .collect::<Vec<Str>>()
                .join(", ")
        )
    }
}

impl<'show, Item: Display> Show<'show> for Set<Item> {
    fn format(&self) -> Str<'show> {
        Str::from(
            self.iter()
                .map(|form| Str::from(form.to_string()))
                .collect::<Vec<Str>>()
                .join(", ")
        )
    }
}

impl<'show> Show<'show> for String {
    fn format(&self) -> Str<'show> {
         Str::from(self.clone())
    }
}

impl<'show> Show<'show> for &'show str {
    fn format(&self) -> Str<'show> {
        Str::from(*self)
    }
}

impl<'show> Show<'show> for Str<'show> {
    fn format(&self) -> Str<'show> {
        *self
    }
}
