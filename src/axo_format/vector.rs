use crate::format::Display;

pub trait Show {
    fn format(&self) -> String;
}

impl<Item: Display> Show for Vec<Item> {
    fn format(&self) -> String {
        self.iter().map(|form| form.to_string()).collect::<Vec<_>>().join(", ")
    }
}