use crate::{
    data::Scale,
    format::{Display, Formatter, Result, Show, Stencil},
    resolver::Type,
};

#[derive(Clone, Debug)]
pub enum ErrorKind<'error> {
    InvalidType(Type<'error>),
    UnsupportedFloatWidth(Scale),
    Cast,
    Bitwise(BitwiseError),
    Function(FunctionError),
    Variable(VariableError),
    ControlFlow(ControlFlowError),
    DataStructure(DataStructureError),
    BuilderError(BuilderError),
    Verification(String),
    Normalize,
    SizeOf,
    Negate,
    Boolean,
}

#[derive(Clone, Eq, Debug, PartialEq)]
pub enum AlignmentError {
    NonPowerOfTwo(u32),
    SrcNonPowerOfTwo(u32),
    DestNonPowerOfTwo(u32),
    Unsized,
    UnalignedInstruction,
}

#[derive(Clone, Eq, Debug, PartialEq)]
pub enum OrderingError {
    WeakerThanMonotic,
    WeakerSuccessOrdering,
    ReleaseOrAcqRel,
}

#[derive(Clone, Eq, Debug, PartialEq)]
pub enum BuilderError {
    Function,
    Parent,
    BlockInsertion,
    UnsetPosition,
    AlignmentError(AlignmentError),
    ExtractOutOfRange,
    BitwidthError,
    PointeeTypeMismatch,
    NotSameType,
    NotPointerOrInteger,
    OrderingError(OrderingError),
    GEPPointee,
    GEPIndex,
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
    MissingReturn,
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
    FieldMissingAnnotation {
        struct_name: String,
        field_name: String,
    },
    NotAStructType {
        name: String,
    },
    UnknownStructType {
        name: String,
    },
    ConstructorFieldTypeMismatch {
        struct_name: String,
        field_name: String,
    },
    UnknownField {
        target: String,
        member: String,
    },
    TooManyInitializers {
        target: String,
    },
    ConstructorPositionalArgTypeMismatch {
        struct_name: String,
        index: usize,
    },
    InvalidModuleAccess,
    InvalidMemberAccessExpression,
    AccessOnNonStructType {
        field_name: String,
    },
    EmptyArray,
    ArrayLiteralTypeMismatch {
        index: usize,
    },
    IndexMissingArgument,
    TupleIndexNotConstant,
    ArrayIndexNotConstant,
    NotIndexable,
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ErrorKind::InvalidType(typing) => {
                write!(f, "invalid LLVM type {}", typing.format(Stencil::default()))
            }
            ErrorKind::UnsupportedFloatWidth(width) => {
                write!(f, "invalid LLVM float width: {}", width)
            }
            ErrorKind::Bitwise(error) => write!(f, "{}", error),
            ErrorKind::Function(error) => write!(f, "{}", error),
            ErrorKind::Variable(error) => write!(f, "{}", error),
            ErrorKind::ControlFlow(error) => write!(f, "{}", error),
            ErrorKind::DataStructure(error) => write!(f, "{}", error),
            ErrorKind::Verification(error) => write!(f, "verification error: {}", error),
            ErrorKind::Normalize => write!(f, "normalization error"),
            ErrorKind::BuilderError(error) => write!(f, "builder error: {}", error),
            ErrorKind::Cast => write!(f, "Unsupported or incompatible cast operation"),
            ErrorKind::SizeOf => write!(f, "Cannot compute the byte size of the provided type"),
            ErrorKind::Negate => {
                write!(f, "Operand cannot be negated (must be an Integer or Float)")
            }
            ErrorKind::Boolean => write!(f, "not a Boolean"),
        }
    }
}

impl Display for AlignmentError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            AlignmentError::NonPowerOfTwo(v) => {
                write!(
                    f,
                    "{} is not a power of two and cannot be used for alignment",
                    v
                )
            }
            AlignmentError::SrcNonPowerOfTwo(_v) => {
                write!(f, "The src_align_bytes argument was not a power of two.")
            }
            AlignmentError::DestNonPowerOfTwo(_v) => {
                write!(f, "The dest_align_bytes argument was not a power of two.")
            }
            AlignmentError::Unsized => {
                write!(
                    f,
                    "type is unsized and cannot be aligned. Suggestion: Align memory manually."
                )
            }
            AlignmentError::UnalignedInstruction => {
                write!(f, "value is not an alloca, load, or store instruction.")
            }
        }
    }
}

impl Display for OrderingError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            OrderingError::WeakerThanMonotic => {
                write!(
                    f,
                    "Both success and failure orderings must be monotonic or stronger."
                )
            }
            OrderingError::WeakerSuccessOrdering => {
                write!(
                    f,
                    "The failure ordering may not be stronger than the success ordering."
                )
            }
            OrderingError::ReleaseOrAcqRel => {
                write!(
                    f,
                    "The failure ordering may not be release or acquire release."
                )
            }
        }
    }
}

impl Display for BuilderError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            BuilderError::Parent => {
                write!(f, "parent error")
            }
            BuilderError::Function => {
                write!(f, "function error")
            }
            BuilderError::BlockInsertion => {
                write!(f, "Builder block cannot be inserted.")
            }
            BuilderError::UnsetPosition => {
                write!(f, "Builder position is not set")
            }
            BuilderError::AlignmentError(error) => {
                write!(f, "Alignment error: {}", error)
            }
            BuilderError::ExtractOutOfRange => {
                write!(f, "Aggregate extract index out of range")
            }
            BuilderError::BitwidthError => {
                write!(
                    f,
                    "The bitwidth of value must be a power of 2 and greater than or equal to 8."
                )
            }
            BuilderError::PointeeTypeMismatch => {
                write!(f, "Pointee type does not match the value's type")
            }
            BuilderError::NotSameType => {
                write!(f, "Values must have the same type")
            }
            BuilderError::NotPointerOrInteger => {
                write!(f, "Values must have pointer or integer type")
            }
            BuilderError::OrderingError(_) => {
                write!(f, "Ordering error or mismatch")
            }
            BuilderError::GEPPointee => {
                write!(f, "GEP pointee is not a struct")
            }
            BuilderError::GEPIndex => {
                write!(f, "GEP index out of range")
            }
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
                write!(
                    f,
                    "operation cannot be performed outside of a function context"
                )
            }
            FunctionError::MissingReturn => {
                write!(f, "missing return value")
            }
        }
    }
}

impl Display for VariableError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            VariableError::AddressOfRValue => {
                write!(
                    f,
                    "cannot take the address of an rvalue or non-existent entity"
                )
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
            DataStructureError::FieldMissingAnnotation {
                struct_name,
                field_name,
            } => {
                write!(
                    f,
                    "struct field '{}' in '{}' is missing a type annotation",
                    field_name, struct_name
                )
            }
            DataStructureError::NotAStructType { name } => {
                write!(f, "'{}' is not a struct type", name)
            }
            DataStructureError::UnknownStructType { name } => {
                write!(f, "unknown struct type '{}'", name)
            }
            DataStructureError::ConstructorFieldTypeMismatch {
                struct_name,
                field_name,
            } => {
                write!(
                    f,
                    "type mismatch for field '{}' in constructor for '{}'",
                    field_name, struct_name
                )
            }
            DataStructureError::UnknownField {
                target: struct_name,
                member: field_name,
            } => {
                write!(
                    f,
                    "struct '{}' has no field named '{}'",
                    struct_name, field_name
                )
            }
            DataStructureError::TooManyInitializers {
                target: struct_name,
            } => {
                write!(
                    f,
                    "too many positional initializers for struct '{}'",
                    struct_name
                )
            }
            DataStructureError::ConstructorPositionalArgTypeMismatch { struct_name, index } => {
                write!(
                    f,
                    "type mismatch for positional argument {} in constructor for '{}'",
                    index, struct_name
                )
            }
            DataStructureError::InvalidModuleAccess => write!(f, "invalid module access"),
            DataStructureError::InvalidMemberAccessExpression => {
                write!(f, "struct member access must use a simple name")
            }
            DataStructureError::AccessOnNonStructType { field_name } => {
                write!(
                    f,
                    "attempted to access field '{}' on a non-struct type or value",
                    field_name
                )
            }
            DataStructureError::EmptyArray => {
                write!(f, "cannot create an empty array without a type annotation")
            }
            DataStructureError::ArrayLiteralTypeMismatch { index } => {
                write!(
                    f,
                    "type mismatch in array literal: element {} has an incompatible type",
                    index
                )
            }
            DataStructureError::IndexMissingArgument => {
                write!(f, "index operation requires at least one index argument")
            }
            DataStructureError::TupleIndexNotConstant => {
                write!(f, "tuple index must be a compile-time constant")
            }
            DataStructureError::ArrayIndexNotConstant => {
                write!(f, "array value index must be a compile-time constant")
            }
            DataStructureError::NotIndexable => {
                write!(f, "type cannot be indexed or invalid index provided")
            }
        }
    }
}

impl From<inkwell::builder::BuilderError> for BuilderError {
    fn from(error: inkwell::builder::BuilderError) -> Self {
        match error {
            inkwell::builder::BuilderError::UnsetPosition => BuilderError::UnsetPosition,
            inkwell::builder::BuilderError::AlignmentError(error) => {
                BuilderError::AlignmentError(error.into())
            }
            inkwell::builder::BuilderError::ExtractOutOfRange => BuilderError::ExtractOutOfRange,
            inkwell::builder::BuilderError::BitwidthError => BuilderError::BitwidthError,
            inkwell::builder::BuilderError::PointeeTypeMismatch => {
                BuilderError::PointeeTypeMismatch
            }
            inkwell::builder::BuilderError::NotSameType => BuilderError::NotSameType,
            inkwell::builder::BuilderError::NotPointerOrInteger => {
                BuilderError::NotPointerOrInteger
            }
            inkwell::builder::BuilderError::OrderingError(error) => {
                BuilderError::OrderingError(error.into())
            }
            inkwell::builder::BuilderError::GEPPointee => BuilderError::GEPPointee,
            inkwell::builder::BuilderError::GEPIndex => BuilderError::GEPIndex,
        }
    }
}

impl From<inkwell::error::AlignmentError> for AlignmentError {
    fn from(error: inkwell::error::AlignmentError) -> Self {
        match error {
            inkwell::error::AlignmentError::NonPowerOfTwo(value) => {
                AlignmentError::NonPowerOfTwo(value.into())
            }
            inkwell::error::AlignmentError::SrcNonPowerOfTwo(value) => {
                AlignmentError::SrcNonPowerOfTwo(value.into())
            }
            inkwell::error::AlignmentError::DestNonPowerOfTwo(value) => {
                AlignmentError::DestNonPowerOfTwo(value.into())
            }
            inkwell::error::AlignmentError::Unsized => AlignmentError::Unsized,
            inkwell::error::AlignmentError::UnalignedInstruction => {
                AlignmentError::UnalignedInstruction
            }
        }
    }
}

impl From<inkwell::builder::OrderingError> for OrderingError {
    fn from(error: inkwell::builder::OrderingError) -> Self {
        match error {
            inkwell::builder::OrderingError::WeakerThanMonotic => OrderingError::WeakerThanMonotic,
            inkwell::builder::OrderingError::WeakerSuccessOrdering => {
                OrderingError::WeakerSuccessOrdering
            }
            inkwell::builder::OrderingError::ReleaseOrAcqRel => OrderingError::ReleaseOrAcqRel,
        }
    }
}
