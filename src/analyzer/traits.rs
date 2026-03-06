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
                        )
                    }
                    Analysis::Float { value, size } => {
                        format!("Float[{}]({})", size, value)
                    }
                    Analysis::Boolean { value } => {
                        format!("Boolean({})", value)
                    }
                    Analysis::String { value } => {
                        format!("String({})", value)
                    }
                    Analysis::Character { value } => {
                        format!("Character({})", value)
                    }
                    Analysis::Array(array) => {
                        format!("Array({})", array.format(verbosity))
                    }
                    Analysis::Tuple(tuple) => {
                        format!("Tuple({})", tuple.format(verbosity))
                    }
                    Analysis::Add(left, right) => {
                        format!(
                            "Add({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::Subtract(left, right) => {
                        format!(
                            "Subtract({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::Multiply(left, right) => {
                        format!(
                            "Multiply({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::Divide(left, right) => {
                        format!(
                            "Divide({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::Modulus(left, right) => {
                        format!(
                            "Modulus({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::LogicalAnd(left, right) => {
                        format!(
                            "Logical(And)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::LogicalOr(left, right) => {
                        format!(
                            "Logical(Or)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::LogicalNot(target) => {
                        format!("Logical(Not)({})", target.format(verbosity))
                    }
                    Analysis::LogicalXOr(left, right) => {
                        format!(
                            "Logical(XOr)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::BitwiseAnd(left, right) => {
                        format!(
                            "Bitwise(And)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::BitwiseOr(left, right) => {
                        format!(
                            "Bitwise(Or)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::BitwiseNot(value) => {
                        format!("Bitwise(Not)({})", value.format(verbosity))
                    }
                    Analysis::BitwiseXOr(left, right) => {
                        format!(
                            "Bitwise(XOr)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::ShiftLeft(left, right) => {
                        format!(
                            "Shift(Left)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::ShiftRight(left, right) => {
                        format!(
                            "Shift(Right)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::AddressOf(value) => {
                        format!("Address({})", value.format(verbosity))
                    }
                    Analysis::Dereference(value) => {
                        format!("Dereference({})", value.format(verbosity))
                    }
                    Analysis::Equal(left, right) => {
                        format!(
                            "Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::NotEqual(left, right) => {
                        format!(
                            "Equal(Not)({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::Less(left, right) => {
                        format!(
                            "Less({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::LessOrEqual(left, right) => {
                        format!(
                            "Less/Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::Greater(left, right) => {
                        format!(
                            "Greater({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::GreaterOrEqual(left, right) => {
                        format!(
                            "Greater/Equal({}, {})",
                            left.format(verbosity),
                            right.format(verbosity)
                        )
                    }
                    Analysis::Index(index) => {
                        index.format(verbosity).to_string()
                    }
                    Analysis::Invoke(invoke) => {
                        invoke.format(verbosity).to_string()
                    }
                    Analysis::Block(block) => {
                        format!("Block({})", block.format(verbosity))
                    }
                    Analysis::Conditional(condition, then, alternate) => {
                        format!(
                            "Conditional({}, {}, {})",
                            condition.format(verbosity),
                            then.format(verbosity),
                            alternate.format(verbosity)
                        )
                    }
                    Analysis::While(condition, then) => {
                        format!(
                            "While({}, {})",
                            condition.format(verbosity),
                            then.format(verbosity)
                        )
                    }
                    Analysis::Cycle(condition, then) => {
                        format!(
                            "Cycle({}, {})",
                            condition.format(verbosity),
                            then.format(verbosity)
                        )
                    }
                    Analysis::Return(value) => {
                        format!(
                            "Return{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        )
                    }
                    Analysis::Break(value) => {
                        format!(
                            "Break{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        )
                    }
                    Analysis::Continue(value) => {
                        format!(
                            "Continue{}",
                            if let Some(value) = value {
                                format!("({})", value.format(verbosity))
                            } else {
                                String::new()
                            }
                        )
                    }
                    Analysis::Usage(target) => {
                        format!("Usage({})", target.format(verbosity))
                    }
                    Analysis::Access(target, value) => {
                        format!(
                            "Access({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        )
                    }
                    Analysis::Constructor(constructor) => {
                        format!(
                            "Constructor({})[{}]",
                            constructor.target.format(verbosity),
                            constructor.members.format(verbosity)
                        )
                    }
                    Analysis::Assign(target, value) => {
                        format!(
                            "Assign({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        )
                    }
                    Analysis::Store(target, value) => {
                        format!(
                            "Store({})({})",
                            target.format(verbosity),
                            value.format(verbosity)
                        )
                    }
                    Analysis::Binding(binding) => {
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
                    Analysis::Structure(structure) => {
                        format!(
                            "Structure({})[{}]",
                            structure.target.format(verbosity),
                            structure.members.format(verbosity)
                        )
                    }
                    Analysis::Enumeration(enumeration) => {
                        format!(
                            "Enumeration({})[{}]",
                            enumeration.target.format(verbosity),
                            enumeration.members.format(verbosity)
                        )
                    }
                    Analysis::Method(method) => {
                        format!(
                            "Method({})[{}{}]{{{}}}",
                            method.target.format(verbosity),
                            if method.variadic { "Variadic | " } else { "" },
                            method.members.format(verbosity),
                            method.body.format(verbosity)
                        )
                    }
                    Analysis::Module(name, members) => {
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
