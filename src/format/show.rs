use {
    crate::{
        format::Display,
    }
};

pub trait Show {
    fn format(&self) -> String;
    fn indent(&self) -> String {
        self.format().lines()
            .map(|line| format!("    {}", line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl<Item: Display> Show for Item {
    fn format(&self) -> String {
        self.to_string()
    }
}

impl<Item: Display> Show for [Item] {
    fn format(&self) -> String {
        self.iter().map(|form| form.to_string()).collect::<Vec<_>>().join(", ")
    }
}