pub enum Type {
    Any,
    Unit,
    Bool,
    Integer(u16, bool),
    Float(u16),
    String,
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Struct(String),
    Function(Vec<Type>, Box<Type>),
    Type(Box<Type>),
}