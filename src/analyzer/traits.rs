use crate::analyzer::{Analysis, AnalysisKind};
use crate::data::Str;
use crate::format::Show;

impl<'analysis> Show<'analysis> for Analysis<'analysis> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'analysis> {
        match verbosity {
            0 => {
                match &self.kind {
                    AnalysisKind::Integer {
                        value,
                        size,
                        signed,
                    } => {
                        format!(
                            "Integer[{}]({}{})",
                            size,
                            if *signed { "Signed | " } else { "" },
                            value
                        ).into()
                    }
                    AnalysisKind::Float { value, size } => {
                        format!("Float[{}]({})", size, value).into()
                    }
                    AnalysisKind::Boolean { value } => {
                        format!("Boolean({})", value).into()
                    }
                    AnalysisKind::String { value } => {
                        format!("String({})", value).into()
                    }
                    AnalysisKind::Character { value } => {
                        format!("Character({})", value).into()
                    }
                    AnalysisKind::Array(array) => {
                        format!("Array({})", array.format(verbosity)).into()
                    }
                    AnalysisKind::Tuple(tuple) => {
                        format!("Tuple({})", tuple.format(verbosity)).into()
                    }

                    AnalysisKind::Cast(analysis, typ) => {
                        format!("Cast({}, {})", analysis.format(verbosity), typ.format(verbosity)).into()
                    }
                    AnalysisKind::Negate(analysis) => {
                        format!("Negate({})", analysis.format(verbosity)).into()
                    }
                    AnalysisKind::SizeOf(analysis) => {
                        format!("SizeOf({})", analysis.format(verbosity)).into()
                    }
                    
                    AnalysisKind::Add(left, right) => {
                        format!(
                            "Add({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::Subtract(left, right) => {
                        format!(
                            "Subtract({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::Multiply(left, right) => {
                        format!(
                            "Multiply({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::Divide(left, right) => {
                        format!(
                            "Divide({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::Modulus(left, right) => {
                        format!(
                            "Modulus({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::LogicalAnd(left, right) => {
                        format!(
                            "Logical(And)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::LogicalOr(left, right) => {
                        format!(
                            "Logical(Or)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::LogicalNot(target) => {
                        format!("Logical(Not)({})", target.format(verbosity)).into()
                    }
                    AnalysisKind::LogicalXOr(left, right) => {
                        format!(
                            "Logical(XOr)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::BitwiseAnd(left, right) => {
                        format!(
                            "Bitwise(And)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::BitwiseOr(left, right) => {
                        format!(
                            "Bitwise(Or)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::BitwiseNot(value) => {
                        format!("Bitwise(Not)({})", value.format(verbosity)).into()
                    }
                    AnalysisKind::BitwiseXOr(left, right) => {
                        format!(
                            "Bitwise(XOr)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::ShiftLeft(left, right) => {
                        format!(
                            "Shift(Left)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::ShiftRight(left, right) => {
                        format!(
                            "Shift(Right)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::AddressOf(value) => {
                        format!("Address({})", value.format(verbosity)).into()
                    }
                    AnalysisKind::Dereference(value) => {
                        format!("Dereference({})", value.format(verbosity)).into()
                    }
                    AnalysisKind::Equal(left, right) => {
                        format!(
                            "Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::NotEqual(left, right) => {
                        format!(
                            "Equal(Not)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::Less(left, right) => {
                        format!(
                            "Less({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::LessOrEqual(left, right) => {
                        format!(
                            "Less/Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::Greater(left, right) => {
                        format!(
                            "Greater({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::GreaterOrEqual(left, right) => {
                        format!(
                            "Greater/Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::Index(index) => {
                        index.format(verbosity)
                    }
                    AnalysisKind::Invoke(invoke) => {
                        invoke.format(verbosity)
                    }
                    AnalysisKind::Block(block) => {
                        format!("Block({})", block.format(verbosity)).into()
                    }
                    AnalysisKind::Conditional(condition, then, alternate) => {
                        format!(
                            "Conditional({}, {}, {})",
                            condition.format(verbosity),
                            then.format(verbosity),
                            alternate.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::While(condition, then) => {
                        format!(
                            "While({}, {})",
                            condition.format(verbosity),
                            then.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::Return(value) => {
                        format!(
                            "Return{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        ).into()
                    }
                    AnalysisKind::Break(value) => {
                        format!(
                            "Break{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        ).into()
                    }
                    AnalysisKind::Continue(value) => {
                        format!(
                            "Continue{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        ).into()
                    }
                    AnalysisKind::Usage(target) => {
                        format!("Usage({})", target.format(verbosity)).into()
                    }
                    AnalysisKind::Access(target, value) => {
                        format!(
                            "Access({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::Constructor(constructor) => {
                        constructor.format(verbosity)
                    }
                    AnalysisKind::Assign(target, value) => {
                        format!(
                            "Assign({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::Store(target, value) => {
                        format!(
                            "Store({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        ).into()
                    }
                    AnalysisKind::Binding(binding) => {
                        binding.format(verbosity)
                    }
                    AnalysisKind::Structure(structure) => {
                        structure.format(verbosity)
                    }
                    AnalysisKind::Union(union) => {
                        union.format(verbosity)
                    }
                    AnalysisKind::Function(function) => {
                        function.format(verbosity)
                    }
                    AnalysisKind::Module(name, members) => {
                        format!(
                            "Module({})[{}]",
                            name.format(verbosity),
                            members.format(verbosity)
                        ).into()
                    }
                }
            },

            _ => self.format(verbosity - 1),
        }
    }
}
