use crate::{
    analyzer::{Analysis, AnalysisKind},
    format::{Show, Stencil},
};

impl<'analysis> Show<'analysis> for Analysis<'analysis> {
    fn format(&self, config: Stencil) -> Stencil {
        self.kind.format(config)
    }
}

impl<'analysis> Show<'analysis> for AnalysisKind<'analysis> {
    fn format(&self, config: Stencil) -> Stencil {
        let base = config.clone().new("AnalysisKind");
        match &self {
            AnalysisKind::Integer {
                value,
                size,
                signed,
            } => base
                .variant("Integer")
                .field("value", value.to_string())
                .field("size", size.to_string())
                .field("signed", signed.to_string()),
            AnalysisKind::Float { value, size } => base
                .variant("Float")
                .field("value", value.to_string())
                .field("size", size.to_string()),
            AnalysisKind::Boolean { value } => {
                base.variant("Boolean").field("value", value.to_string())
            }
            AnalysisKind::String { value } => base
                .variant("String")
                .field("value", format!("\"{}\"", value)),
            AnalysisKind::Character { value } => base
                .variant("Character")
                .field("value", format!("'{}'", value)),
            AnalysisKind::Array(array) => base
                .variant("Array")
                .field("elements", array.format(config.clone())),
            AnalysisKind::Tuple(tuple) => base
                .variant("Tuple")
                .field("elements", tuple.format(config.clone())),
            AnalysisKind::Negate(analysis) => base
                .variant("Negate")
                .field("target", analysis.format(config.clone())),
            AnalysisKind::SizeOf(analysis) => base
                .variant("SizeOf")
                .field("target", analysis.format(config.clone())),
            AnalysisKind::Add(left, right) => {
                format_binary(&config, "AnalysisKind", "Add", left, right)
            }
            AnalysisKind::Subtract(left, right) => {
                format_binary(&config, "AnalysisKind", "Subtract", left, right)
            }
            AnalysisKind::Multiply(left, right) => {
                format_binary(&config, "AnalysisKind", "Multiply", left, right)
            }
            AnalysisKind::Divide(left, right) => {
                format_binary(&config, "AnalysisKind", "Divide", left, right)
            }
            AnalysisKind::Modulus(left, right) => {
                format_binary(&config, "AnalysisKind", "Modulus", left, right)
            }
            AnalysisKind::LogicalAnd(left, right) => {
                format_binary(&config, "AnalysisKind", "LogicalAnd", left, right)
            }
            AnalysisKind::LogicalOr(left, right) => {
                format_binary(&config, "AnalysisKind", "LogicalOr", left, right)
            }
            AnalysisKind::LogicalXOr(left, right) => {
                format_binary(&config, "AnalysisKind", "LogicalXOr", left, right)
            }
            AnalysisKind::BitwiseAnd(left, right) => {
                format_binary(&config, "AnalysisKind", "BitwiseAnd", left, right)
            }
            AnalysisKind::BitwiseOr(left, right) => {
                format_binary(&config, "AnalysisKind", "BitwiseOr", left, right)
            }
            AnalysisKind::BitwiseXOr(left, right) => {
                format_binary(&config, "AnalysisKind", "BitwiseXOr", left, right)
            }
            AnalysisKind::ShiftLeft(left, right) => {
                format_binary(&config, "AnalysisKind", "ShiftLeft", left, right)
            }
            AnalysisKind::ShiftRight(left, right) => {
                format_binary(&config, "AnalysisKind", "ShiftRight", left, right)
            }
            AnalysisKind::Equal(left, right) => {
                format_binary(&config, "AnalysisKind", "Equal", left, right)
            }
            AnalysisKind::NotEqual(left, right) => {
                format_binary(&config, "AnalysisKind", "NotEqual", left, right)
            }
            AnalysisKind::Less(left, right) => {
                format_binary(&config, "AnalysisKind", "Less", left, right)
            }
            AnalysisKind::LessOrEqual(left, right) => {
                format_binary(&config, "AnalysisKind", "LessOrEqual", left, right)
            }
            AnalysisKind::Greater(left, right) => {
                format_binary(&config, "AnalysisKind", "Greater", left, right)
            }
            AnalysisKind::GreaterOrEqual(left, right) => {
                format_binary(&config, "AnalysisKind", "GreaterOrEqual", left, right)
            }
            AnalysisKind::LogicalNot(target) => base
                .variant("LogicalNot")
                .field("target", target.format(config.clone())),
            AnalysisKind::BitwiseNot(value) => base
                .variant("BitwiseNot")
                .field("target", value.format(config.clone())),
            AnalysisKind::AddressOf(value) => base
                .variant("AddressOf")
                .field("target", value.format(config.clone())),
            AnalysisKind::Dereference(value) => base
                .variant("Dereference")
                .field("target", value.format(config.clone())),
            AnalysisKind::Index(index) => index.format(config.clone()),
            AnalysisKind::Invoke(invoke) => invoke.format(config.clone()),
            AnalysisKind::Block(block) => base
                .variant("Block")
                .field("statements", block.format(config.clone())),
            AnalysisKind::Conditional(condition, then, alternate) => base
                .variant("Conditional")
                .field("condition", condition.format(config.clone()))
                .field("then", then.format(config.clone()))
                .field("alternate", alternate.format(config.clone())),
            AnalysisKind::While(condition, then) => base
                .variant("While")
                .field("condition", condition.format(config.clone()))
                .field("then", then.format(config.clone())),
            AnalysisKind::Return(value) => {
                let mut base = base.variant("Return");
                if let Some(value) = value {
                    base = base.field("value", value.format(config.clone()));
                }
                base
            }
            AnalysisKind::Break(value) => {
                let mut base = base.variant("Break");
                if let Some(value) = value {
                    base = base.field("value", value.format(config.clone()));
                }
                base
            }
            AnalysisKind::Continue(value) => {
                let mut base = base.variant("Continue");
                if let Some(value) = value {
                    base = base.field("value", value.format(config.clone()));
                }
                base
            }
            AnalysisKind::Usage(target) => base
                .variant("Usage")
                .field("target", target.format(config.clone())),
            AnalysisKind::Access(target, value) => base
                .variant("Access")
                .field("target", target.format(config.clone()))
                .field("value", value.format(config.clone())),
            AnalysisKind::Constructor(constructor) => constructor.format(config.clone()),
            AnalysisKind::Assign(target, value) => base
                .variant("Assign")
                .field("target", target.format(config.clone()))
                .field("value", value.format(config.clone())),
            AnalysisKind::Store(target, value) => base
                .variant("Store")
                .field("target", target.format(config.clone()))
                .field("value", value.format(config.clone())),
            AnalysisKind::Binding(binding) => binding.format(config.clone()),
            AnalysisKind::Structure(structure) => base
                .variant("Structure")
                .field("structure", structure.format(config.clone())),
            AnalysisKind::Union(union) => base
                .variant("Union")
                .field("union", union.format(config.clone())),
            AnalysisKind::Function(function) => function.format(config.clone()),
            AnalysisKind::Module(name, members) => base
                .variant("Module")
                .field("name", name.format(config.clone()))
                .field("members", members.format(config.clone())),
        }
    }
}

fn format_binary<'analysis, Left: Show<'analysis>, Right: Show<'analysis>>(
    config: &Stencil,
    head: &str,
    variant: &str,
    left: &Left,
    right: &Right,
) -> Stencil {
    config
        .clone()
        .new(head)
        .variant(variant)
        .field("left", left.format(config.clone()))
        .field("right", right.format(config.clone()))
}
