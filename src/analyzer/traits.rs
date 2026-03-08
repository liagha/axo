use crate::analyzer::Analysis;
use crate::data::Str;
use crate::format::Show;

impl<'analysis> Show<'analysis> for Analysis<'analysis> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'analysis> {
        match verbosity {
            0 => {
                match self {
                    Analysis::Integer {
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
                    Analysis::Float { value, size } => {
                        format!("Float[{}]({})", size, value).into()
                    }
                    Analysis::Boolean { value } => {
                        format!("Boolean({})", value).into()
                    }
                    Analysis::String { value } => {
                        format!("String({})", value).into()
                    }
                    Analysis::Character { value } => {
                        format!("Character({})", value).into()
                    }
                    Analysis::Array(array) => {
                        format!("Array({})", array.format(verbosity)).into()
                    }
                    Analysis::Tuple(tuple) => {
                        format!("Tuple({})", tuple.format(verbosity)).into()
                    }
                    Analysis::Add(left, right) => {
                        format!(
                            "Add({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::Subtract(left, right) => {
                        format!(
                            "Subtract({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::Multiply(left, right) => {
                        format!(
                            "Multiply({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::Divide(left, right) => {
                        format!(
                            "Divide({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::Modulus(left, right) => {
                        format!(
                            "Modulus({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::LogicalAnd(left, right) => {
                        format!(
                            "Logical(And)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::LogicalOr(left, right) => {
                        format!(
                            "Logical(Or)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::LogicalNot(target) => {
                        format!("Logical(Not)({})", target.format(verbosity)).into()
                    }
                    Analysis::LogicalXOr(left, right) => {
                        format!(
                            "Logical(XOr)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::BitwiseAnd(left, right) => {
                        format!(
                            "Bitwise(And)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::BitwiseOr(left, right) => {
                        format!(
                            "Bitwise(Or)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::BitwiseNot(value) => {
                        format!("Bitwise(Not)({})", value.format(verbosity)).into()
                    }
                    Analysis::BitwiseXOr(left, right) => {
                        format!(
                            "Bitwise(XOr)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::ShiftLeft(left, right) => {
                        format!(
                            "Shift(Left)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::ShiftRight(left, right) => {
                        format!(
                            "Shift(Right)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::AddressOf(value) => {
                        format!("Address({})", value.format(verbosity)).into()
                    }
                    Analysis::Dereference(value) => {
                        format!("Dereference({})", value.format(verbosity)).into()
                    }
                    Analysis::Equal(left, right) => {
                        format!(
                            "Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::NotEqual(left, right) => {
                        format!(
                            "Equal(Not)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::Less(left, right) => {
                        format!(
                            "Less({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::LessOrEqual(left, right) => {
                        format!(
                            "Less/Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::Greater(left, right) => {
                        format!(
                            "Greater({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::GreaterOrEqual(left, right) => {
                        format!(
                            "Greater/Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        ).into()
                    }
                    Analysis::Index(index) => {
                        index.format(verbosity)
                    }
                    Analysis::Invoke(invoke) => {
                        invoke.format(verbosity)
                    }
                    Analysis::Block(block) => {
                        format!("Block({})", block.format(verbosity)).into()
                    }
                    Analysis::Conditional(condition, then, alternate) => {
                        format!(
                            "Conditional({}, {}, {})",
                            condition.format(verbosity),
                            then.format(verbosity),
                            alternate.format(verbosity)
                        ).into()
                    }
                    Analysis::While(condition, then) => {
                        format!(
                            "While({}, {})",
                            condition.format(verbosity),
                            then.format(verbosity)
                        ).into()
                    }
                    Analysis::Return(value) => {
                        format!(
                            "Return{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        ).into()
                    }
                    Analysis::Break(value) => {
                        format!(
                            "Break{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        ).into()
                    }
                    Analysis::Continue(value) => {
                        format!(
                            "Continue{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        ).into()
                    }
                    Analysis::Usage(target) => {
                        format!("Usage({})", target.format(verbosity)).into()
                    }
                    Analysis::Access(target, value) => {
                        format!(
                            "Access({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        ).into()
                    }
                    Analysis::Constructor(constructor) => {
                        constructor.format(verbosity)
                    }
                    Analysis::Assign(target, value) => {
                        format!(
                            "Assign({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        ).into()
                    }
                    Analysis::Store(target, value) => {
                        format!(
                            "Store({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        ).into()
                    }
                    Analysis::Binding(binding) => {
                        binding.format(verbosity)
                    }
                    Analysis::Structure(structure) => {
                        structure.format(verbosity)
                    }
                    Analysis::Enumeration(enumeration) => {
                        enumeration.format(verbosity)
                    }
                    Analysis::Method(method) => {
                        method.format(verbosity)
                    }
                    Analysis::Module(name, members) => {
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
