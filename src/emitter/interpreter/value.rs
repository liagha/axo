use crate::data::Str;

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'a> {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    String(Str<'a>),
    Pointer(Box<Value<'a>>),
    Array(Vec<Value<'a>>),
    Tuple(Vec<Value<'a>>),
    Structure(Str<'a>, Vec<Value<'a>>),
    Union(Str<'a>, Box<Value<'a>>),
    Function(Str<'a>),
    Void,
}

impl<'a> Value<'a> {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Integer(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Pointer(inner) => !matches!(**inner, Value::Void),
            _ => false,
        }
    }

    pub fn tag(&self) -> Value<'a> {
        match self {
            Value::Structure(_, fields) => fields.first().cloned().unwrap_or(Value::Void),
            other => other.clone(),
        }
    }
}
