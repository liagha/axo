use crate::analyzer::{Analysis, Instruction};
use crate::data::Str;
use crate::format::Show;

impl<'analysis> Show<'analysis> for Analysis<'analysis> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'analysis> {
        match verbosity {
            0 => {
                format!("{}", self.instruction.format(verbosity))
            }

            _ => self.format(verbosity - 1).to_string(),
        }
        .into()
    }
}

impl<'analysis> Show<'analysis> for Instruction<'analysis> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'analysis> {
        match verbosity {
            0 => {
                match self {
                    Instruction::Integer {
                        value,
                        size,
                        signed,
                    } => {
                        format!(
                            "Integer[{}]({}{})",
                            size,
                            if *signed { "Signed | " } else { "" },
                            value
                        )
                    }
                    Instruction::Float { value, size } => {
                        format!("Float[{}]({})", size, value)
                    }
                    Instruction::Boolean { value } => {
                        format!("Boolean({})", value)
                    }
                    Instruction::String { value } => {
                        format!("String({})", value)
                    }
                    Instruction::Character { value } => {
                        format!("Character({})", value)
                    }
                    Instruction::Array(array) => {
                        format!("Array({})", array.format(verbosity))
                    }
                    Instruction::Tuple(tuple) => {
                        format!("Tuple({})", tuple.format(verbosity))
                    }
                    Instruction::Add(left, right) => {
                        format!(
                            "Add({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::Subtract(left, right) => {
                        format!(
                            "Subtract({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::Multiply(left, right) => {
                        format!(
                            "Multiply({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::Divide(left, right) => {
                        format!(
                            "Divide({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::Modulus(left, right) => {
                        format!(
                            "Modulus({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::LogicalAnd(left, right) => {
                        format!(
                            "Logical(And)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::LogicalOr(left, right) => {
                        format!(
                            "Logical(Or)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::LogicalNot(target) => {
                        format!("Logical(Not)({})", target.format(verbosity))
                    }
                    Instruction::LogicalXOr(left, right) => {
                        format!(
                            "Logical(XOr)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::BitwiseAnd(left, right) => {
                        format!(
                            "Bitwise(And)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::BitwiseOr(left, right) => {
                        format!(
                            "Bitwise(Or)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::BitwiseNot(value) => {
                        format!("Bitwise(Not)({})", value.format(verbosity))
                    }
                    Instruction::BitwiseXOr(left, right) => {
                        format!(
                            "Bitwise(XOr)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::ShiftLeft(left, right) => {
                        format!(
                            "Shift(Left)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::ShiftRight(left, right) => {
                        format!(
                            "Shift(Right)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::AddressOf(value) => {
                        format!("Address({})", value.format(verbosity))
                    }
                    Instruction::Dereference(value) => {
                        format!("Dereference({})", value.format(verbosity))
                    }
                    Instruction::Equal(left, right) => {
                        format!(
                            "Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::NotEqual(left, right) => {
                        format!(
                            "Equal(Not)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::Less(left, right) => {
                        format!(
                            "Less({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::LessOrEqual(left, right) => {
                        format!(
                            "Less/Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::Greater(left, right) => {
                        format!(
                            "Greater({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::GreaterOrEqual(left, right) => {
                        format!(
                            "Greater/Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Instruction::Index(index) => {
                        format!(
                            "Index({})[{}]",
                            index.target.format(verbosity),
                            index.members.format(verbosity),
                        )
                    }
                    Instruction::Invoke(invoke) => {
                        format!(
                            "Invoke({})[{}]",
                            invoke.target.format(verbosity),
                            invoke.members.format(verbosity),
                        )
                    }
                    Instruction::Block(block) => {
                        format!("Block({})", block.format(verbosity))
                    }
                    Instruction::Conditional(condition, then, alternate) => {
                        format!(
                            "Conditional({}, {}, {})",
                            condition.format(verbosity),
                            then.format(verbosity),
                            alternate.format(verbosity)
                        )
                    }
                    Instruction::While(condition, then) => {
                        format!(
                            "While({}, {})",
                            condition.format(verbosity),
                            then.format(verbosity)
                        )
                    }
                    Instruction::Cycle(condition, then) => {
                        format!(
                            "Cycle({}, {})",
                            condition.format(verbosity),
                            then.format(verbosity)
                        )
                    }
                    Instruction::Return(value) => {
                        format!(
                            "Return{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        )
                    }
                    Instruction::Break(value) => {
                        format!(
                            "Break{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        )
                    }
                    Instruction::Continue(value) => {
                        format!(
                            "Continue{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        )
                    }
                    Instruction::Usage(target) => {
                        format!("Usage({})", target.format(verbosity))
                    }
                    Instruction::Access(target, value) => {
                        format!(
                            "Access({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        )
                    }
                    Instruction::Constructor(constructor) => {
                        format!(
                            "Constructor({})[{}]",
                            constructor.target.format(verbosity),
                            constructor.members.format(verbosity)
                        )
                    }
                    Instruction::Assign(target, value) => {
                        format!(
                            "Assign({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        )
                    }
                    Instruction::Store(target, value) => {
                        format!(
                            "Store({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        )
                    }
                    Instruction::Binding(binding) => {
                        format!(
                            "Binding[{}]({}{}){}",
                            if let Some(annotation) = &binding.annotation { annotation.format(verbosity) } else { "".into() },
                            if binding.constant { "Constant | " } else { "" },
                            binding.target.format(verbosity),
                            if let Some(value) = &binding.value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        )
                    }
                    Instruction::Structure(structure) => {
                        format!(
                            "Structure({})[{}]",
                            structure.target.format(verbosity),
                            structure.members.format(verbosity)
                        )
                    }
                    Instruction::Enumeration(enumeration) => {
                        format!(
                            "Enumeration({})[{}]",
                            enumeration.target.format(verbosity),
                            enumeration.members.format(verbosity)
                        )
                    }
                    Instruction::Method(method) => {
                        format!(
                            "Method({})[{}{}]{{{}}}",
                            method.target.format(verbosity),
                            if method.variadic { "Variadic | " } else { "" },
                            method.members.format(verbosity),
                            method.body.format(verbosity)
                        )
                    }
                    Instruction::Module(name, members) => {
                        format!(
                            "Module({})[{}]",
                            name.format(verbosity),
                            members.format(verbosity)
                        )
                    }
                }
            },

            _ => self.format(verbosity - 1).to_string(),
        }
        .into()
    }
}
