use crate::data::Scale;
use crate::format::{Display, Formatter, Result};

#[derive(Clone, Debug)]
pub enum ErrorKind {
    InvalidModule { reason: String },
    BuilderError { reason: String },
    InvalidType,
    UnsupportedFloatWidth { width: Scale },
    SemanticError { message: String },
    Arithmetic(ArithmeticError),
    Bitwise(BitwiseError),
    Function(FunctionError),
    Variable(VariableError),
    ControlFlow(ControlFlowError),
    DataStructure(DataStructureError),
}

#[derive(Clone, Debug)]
pub enum ArithmeticError {
    InvalidOperandType {
        side: String,
        instruction: String,
    }
}

#[derive(Clone, Debug)]
pub enum BitwiseError {
    InvalidOperandType { instruction: String },
}

#[derive(Clone, Debug)]
pub enum FunctionError {
    IncompatibleReturnType,
    Undefined { name: String },
    NotInFunctionContext,
}

#[derive(Clone, Debug)]
pub enum VariableError {
    AddressOfRValue,
    DereferenceNonPointer,
    NotAValue { name: String },
    Undefined { name: String },
    BindingWithoutInitializer { name: String },
    BindingTypeMismatch { name: String },
    AssignmentTypeMismatch,
    InvalidAssignmentTarget,
}

#[derive(Clone, Debug)]
pub enum ControlFlowError {
    BreakOutsideLoop,
    ContinueOutsideLoop,
}

#[derive(Clone, Debug)]
pub enum DataStructureError {
    FieldMissingAnnotation { struct_name: String, field_name: String },
    NotAStructType { name: String },
    UnknownStructType { name: String },
    ConstructorFieldTypeMismatch { struct_name: String, field_name: String },
    UnknownField { struct_name: String, field_name: String },
    TooManyInitializers { struct_name: String },
    ConstructorPositionalArgTypeMismatch { struct_name: String, index: usize },
    InvalidModuleAccess,
    InvalidMemberAccessExpression,
    AccessOnNonStructType { field_name: String },
    EmptyArray,
    ArrayLiteralTypeMismatch { index: usize },
    IndexMissingArgument,
    TupleIndexNotConstant,
    ArrayIndexNotConstant,
    NotIndexable,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ErrorKind::InvalidModule { reason } => {
                write!(f, "invalid LLVM module: {}.", reason)
            }
            ErrorKind::BuilderError { reason } => {
                write!(f, "builder error: {}", reason)
            }
            ErrorKind::InvalidType => {
                write!(f, "invalid LLVM type")
            }
            ErrorKind::UnsupportedFloatWidth { width } => {
                write!(f, "invalid LLVM float width: {}", width)
            }
            ErrorKind::SemanticError { message } => {
                write!(f, "semantic error: {}", message)
            }
            ErrorKind::Bitwise(e) => write!(f, "{}", e),
            ErrorKind::Function(e) => write!(f, "{}", e),
            ErrorKind::Variable(e) => write!(f, "{}", e),
            ErrorKind::ControlFlow(e) => write!(f, "{}", e),
            ErrorKind::DataStructure(e) => write!(f, "{}", e),
            ErrorKind::Arithmetic(_) => write!(f, "arithmetic error"),
        }
    }
}

impl Display for BitwiseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            BitwiseError::InvalidOperandType { instruction } => {
                write!(f, "bitwise {} requires integer operands", instruction)
            }
        }
    }
}

impl Display for FunctionError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            FunctionError::IncompatibleReturnType => {
                write!(f, "incompatible return type provided")
            }
            FunctionError::Undefined { name } => {
                write!(f, "undefined function or primitive cast '{}'", name)
            }
            FunctionError::NotInFunctionContext => {
                write!(f, "operation cannot be performed outside of a function context")
            }
        }
    }
}

impl Display for VariableError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            VariableError::AddressOfRValue => {
                write!(f, "cannot take the address of an rvalue or non-existent entity")
            }
            VariableError::DereferenceNonPointer => {
                write!(f, "cannot dereference a non-pointer value")
            }
            VariableError::NotAValue { name } => {
                write!(f, "identifier '{}' is not a usable value", name)
            }
            VariableError::Undefined { name } => {
                write!(f, "undefined identifier '{}'", name)
            }
            VariableError::BindingWithoutInitializer { name } => {
                write!(f, "binding '{}' has no initializer", name)
            }
            VariableError::BindingTypeMismatch { name } => {
                write!(f, "type mismatch in binding for '{}'", name)
            }
            VariableError::AssignmentTypeMismatch => {
                write!(f, "type mismatch in variable assignment")
            }
            VariableError::InvalidAssignmentTarget => {
                write!(f, "invalid assignment target")
            }
        }
    }
}

impl Display for ControlFlowError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ControlFlowError::BreakOutsideLoop => {
                write!(f, "break statement outside of a loop")
            }
            ControlFlowError::ContinueOutsideLoop => {
                write!(f, "continue statement outside of a loop")
            }
        }
    }
}

impl Display for DataStructureError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            DataStructureError::FieldMissingAnnotation { struct_name, field_name } => {
                write!(f, "struct field '{}' in '{}' is missing a type annotation", field_name, struct_name)
            }
            DataStructureError::NotAStructType { name } => {
                write!(f, "'{}' is not a struct type", name)
            }
            DataStructureError::UnknownStructType { name } => {
                write!(f, "unknown struct type '{}'", name)
            }
            DataStructureError::ConstructorFieldTypeMismatch { struct_name, field_name } => {
                write!(f, "type mismatch for field '{}' in constructor for '{}'", field_name, struct_name)
            }
            DataStructureError::UnknownField { struct_name, field_name } => {
                write!(f, "struct '{}' has no field named '{}'", struct_name, field_name)
            }
            DataStructureError::TooManyInitializers { struct_name } => {
                write!(f, "too many positional initializers for struct '{}'", struct_name)
            }
            DataStructureError::ConstructorPositionalArgTypeMismatch { struct_name, index } => {
                write!(f, "type mismatch for positional argument {} in constructor for '{}'", index, struct_name)
            }
            DataStructureError::InvalidModuleAccess => write!(f, "invalid module access"),
            DataStructureError::InvalidMemberAccessExpression => write!(f, "struct member access must use a simple name"),
            DataStructureError::AccessOnNonStructType { field_name } => {
                write!(f, "attempted to access field '{}' on a non-struct type or value", field_name)
            }
            DataStructureError::EmptyArray => write!(f, "cannot create an empty array without a type annotation"),
            DataStructureError::ArrayLiteralTypeMismatch { index } => {
                write!(f, "type mismatch in array literal: element {} has an incompatible type", index)
            }
            DataStructureError::IndexMissingArgument => write!(f, "index operation requires at least one index argument"),
            DataStructureError::TupleIndexNotConstant => write!(f, "tuple index must be a compile-time constant"),
            DataStructureError::ArrayIndexNotConstant => write!(f, "array value index must be a compile-time constant"),
            DataStructureError::NotIndexable => write!(f, "type cannot be indexed or invalid index provided"),
        }
    }
}