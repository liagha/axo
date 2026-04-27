pub use crate::generator::{
    AlignmentError,
    BitwiseError,
    BuilderError,
    ControlFlowError,
    DataStructureError,
    ErrorKind,
    FunctionError,
    OrderingError,
    VariableError,
};

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
            inkwell::builder::BuilderError::GEPPointee => BuilderError::GEPPointee,
            inkwell::builder::BuilderError::GEPIndex => BuilderError::GEPIndex,
            inkwell::builder::BuilderError::CmpxchgOrdering(err) => {
                BuilderError::OrderingError(err.into())
            }
            inkwell::builder::BuilderError::AtomicOrdering(err) => {
                BuilderError::OrderingError(err.into())
            }
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

impl From<inkwell::builder::CmpxchgOrderingError> for OrderingError {
    fn from(error: inkwell::builder::CmpxchgOrderingError) -> Self {
        match error {
            inkwell::builder::CmpxchgOrderingError::WeakerThanMonotic => {
                OrderingError::WeakerThanMonotic
            }
            inkwell::builder::CmpxchgOrderingError::WeakerSuccessOrdering => {
                OrderingError::WeakerSuccessOrdering
            }
            inkwell::builder::CmpxchgOrderingError::ReleaseOrAcqRel => {
                OrderingError::ReleaseOrAcqRel
            }
        }
    }
}

impl From<inkwell::values::AtomicError> for OrderingError {
    fn from(error: inkwell::values::AtomicError) -> Self {
        match error {
            inkwell::values::AtomicError::ReleaseOnLoad => OrderingError::ReleaseOnLoad,
            inkwell::values::AtomicError::AcquireRelease => OrderingError::AcquireRelease,
            inkwell::values::AtomicError::AcquireOnStore => OrderingError::AcquireOnStore,
            inkwell::values::AtomicError::InvalidOrderingOnFence => OrderingError::InvalidOrderingOnFence,
            inkwell::values::AtomicError::InvalidOrderingOnAtomicRMW => OrderingError::InvalidOrderingOnAtomicRMW,
        }
    }
}
