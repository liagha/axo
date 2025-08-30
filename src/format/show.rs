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
        self.format().lines()
            .into_iter()
            .map(|line| format!("    {}", line))
            .collect::<Vec<_>>()
            .join("\n")
            .into()
    }
}

impl<'show, Item: Display> Show<'show> for [Item] {
    fn format(&self) -> Str<'show> {
        self.iter()
            .map(|form| Str::from(form.to_string()))
            .collect::<Vec<Str>>()
            .join(", ")
            .into()
    }
}

impl<'show, Item: Display> Show<'show> for Set<Item> {
    fn format(&self) -> Str<'show> {
        self.iter()
            .map(|form| Str::from(form.to_string()))
            .collect::<Vec<Str>>()
            .join(", ")
            .into()
    }
}

impl<'show> Show<'show> for String {
    fn format(&self) -> Str<'show> {
         self.clone().into()
    }
}

impl<'show> Show<'show> for &'show str {
    fn format(&self) -> Str<'show> {
        (*self).into()
    }
}

impl<'show> Show<'show> for Str<'show> {
    fn format(&self) -> Str<'show> {
        *self
    }
}
