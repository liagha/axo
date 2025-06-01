use {
    crate::{
        format::Display,
        axo_span::Span,
    }
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Hint<M: Display> {
    pub message: M,
    pub action: Vec<Action>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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