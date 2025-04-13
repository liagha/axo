use core::fmt::Display;
use crate::axo_lexer::Span;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Hint<T: Display> {
    pub message: T,
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