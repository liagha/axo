use {
    crate::{
        format::Display,
        axo_cursor::Span,
    }
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Hint<M: Display> {
    pub message: M,
    pub action: Vec<Action>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Action {
    Add(String, Span),
    AddLine(String, usize),
    Remove(Span),
    RemoveLine(usize),
    Replace(String, Span),
    ReplaceLine(String, usize),
    Switch(Span, Span),
    SwitchLine(usize, usize),
}