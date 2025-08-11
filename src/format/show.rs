use {
    crate::{
        data::{string::Str},
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

impl<'show, Item: Display> Show<'show> for Item {
    fn format(&self) -> Str<'show> {
        Str::from(self.to_string())
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