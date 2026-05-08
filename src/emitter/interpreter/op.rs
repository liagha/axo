// src/emitter/interpreter/op.rs

use crate::data::{Scale, Str};

#[derive(Clone, Debug)]
pub enum Op<'a> {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    String(Str<'a>),
    Void,

    Load(usize),
    Store(usize),
    DefineGlobal(Str<'a>),
    LoadGlobal(Str<'a>),
    StoreGlobal(Str<'a>),

    Pop,
    Dup,

    Negate,
    Not,
    BitwiseNot,

    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    And,
    Or,
    Xor,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,

    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,

    AddressOf,
    Deref,

    MakeArray(usize),
    MakeTuple(usize),
    MakeStruct(Str<'a>, usize),
    MakeUnion(Str<'a>),

    GetField(usize),
    GetIndex,
    SetIndex,

    Jump(usize),
    JumpIf(usize),
    JumpIfNot(usize),

    Call(Str<'a>, usize),
    CallForeign(Str<'a>, usize),
    Return,

    SizeOf(usize),

    EnterBlock,
    LeaveBlock,

    BreakSignal,
    ContinueSignal,
    ReturnSignal,
}