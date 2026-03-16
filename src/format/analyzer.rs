use {
    crate::{
        data::Str,
        format::{Show, Verbosity},
        analyzer::{Analysis, AnalysisKind}
    },
};

impl<'analysis> Show<'analysis> for Analysis<'analysis> {
    fn format(&self, verbosity: Verbosity) -> Str<'analysis> {
        if verbosity == Verbosity::Off {
            return "".into();
        }

        match &self.kind {
            AnalysisKind::Integer { value, size, signed } => match verbosity {
                Verbosity::Minimal => format!("{}", value).into(),
                Verbosity::Detailed => format!("Integer[{}]({}{})", size, if *signed { "Signed | " } else { "" }, value).into(),
                Verbosity::Debug => format!("Integer {{\n    value: {},\n    size: {},\n    signed: {}\n}}", value, size, signed).into(),
                _ => "".into(),
            },
            AnalysisKind::Float { value, size } => match verbosity {
                Verbosity::Minimal => format!("{}", value).into(),
                Verbosity::Detailed => format!("Float[{}]({})", size, value).into(),
                Verbosity::Debug => format!("Float {{\n    value: {},\n    size: {}\n}}", value, size).into(),
                _ => "".into(),
            },
            AnalysisKind::Boolean { value } => match verbosity {
                Verbosity::Minimal => format!("{}", value).into(),
                Verbosity::Detailed => format!("Boolean({})", value).into(),
                Verbosity::Debug => format!("Boolean {{\n    value: {}\n}}", value).into(),
                _ => "".into(),
            },
            AnalysisKind::String { value } => match verbosity {
                Verbosity::Minimal => format!("\"{}\"", value).into(),
                Verbosity::Detailed => format!("String(\"{}\")", value).into(),
                Verbosity::Debug => format!("String {{\n    value: \"{}\"\n}}", value).into(),
                _ => "".into(),
            },
            AnalysisKind::Character { value } => match verbosity {
                Verbosity::Minimal => format!("'{}'", value).into(),
                Verbosity::Detailed => format!("Character('{}')", value).into(),
                Verbosity::Debug => format!("Character {{\n    value: '{}'\n}}", value).into(),
                _ => "".into(),
            },
            AnalysisKind::Array(array) => match verbosity {
                Verbosity::Minimal => format!("[{}]", array.format(verbosity)).into(),
                Verbosity::Detailed => format!("Array({})", array.format(verbosity)).into(),
                Verbosity::Debug => format!("Array {{\n{}\n}}", array.format(verbosity).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::Tuple(tuple) => match verbosity {
                Verbosity::Minimal => format!("({})", tuple.format(verbosity)).into(),
                Verbosity::Detailed => format!("Tuple({})", tuple.format(verbosity)).into(),
                Verbosity::Debug => format!("Tuple {{\n{}\n}}", tuple.format(verbosity).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::Negate(analysis) => match verbosity {
                Verbosity::Minimal => format!("-{}", analysis.format(verbosity)).into(),
                Verbosity::Detailed => format!("Negate({})", analysis.format(verbosity)).into(),
                Verbosity::Debug => format!("Negate {{\n{}\n}}", format!("target: {}", analysis.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::SizeOf(analysis) => match verbosity {
                Verbosity::Minimal => format!("sizeof({})", analysis.format(verbosity)).into(),
                Verbosity::Detailed => format!("SizeOf({})", analysis.format(verbosity)).into(),
                Verbosity::Debug => format!("SizeOf {{\n{}\n}}", format!("target: {}", analysis.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::Add(left, right) => format_binary("Add", "+", left, right, verbosity),
            AnalysisKind::Subtract(left, right) => format_binary("Subtract", "-", left, right, verbosity),
            AnalysisKind::Multiply(left, right) => format_binary("Multiply", "*", left, right, verbosity),
            AnalysisKind::Divide(left, right) => format_binary("Divide", "/", left, right, verbosity),
            AnalysisKind::Modulus(left, right) => format_binary("Modulus", "%", left, right, verbosity),
            AnalysisKind::LogicalAnd(left, right) => format_binary("LogicalAnd", "&&", left, right, verbosity),
            AnalysisKind::LogicalOr(left, right) => format_binary("LogicalOr", "||", left, right, verbosity),
            AnalysisKind::LogicalXOr(left, right) => format_binary("LogicalXOr", "^^", left, right, verbosity),
            AnalysisKind::BitwiseAnd(left, right) => format_binary("BitwiseAnd", "&", left, right, verbosity),
            AnalysisKind::BitwiseOr(left, right) => format_binary("BitwiseOr", "|", left, right, verbosity),
            AnalysisKind::BitwiseXOr(left, right) => format_binary("BitwiseXOr", "^", left, right, verbosity),
            AnalysisKind::ShiftLeft(left, right) => format_binary("ShiftLeft", "<<", left, right, verbosity),
            AnalysisKind::ShiftRight(left, right) => format_binary("ShiftRight", ">>", left, right, verbosity),
            AnalysisKind::Equal(left, right) => format_binary("Equal", "==", left, right, verbosity),
            AnalysisKind::NotEqual(left, right) => format_binary("NotEqual", "!=", left, right, verbosity),
            AnalysisKind::Less(left, right) => format_binary("Less", "<", left, right, verbosity),
            AnalysisKind::LessOrEqual(left, right) => format_binary("LessOrEqual", "<=", left, right, verbosity),
            AnalysisKind::Greater(left, right) => format_binary("Greater", ">", left, right, verbosity),
            AnalysisKind::GreaterOrEqual(left, right) => format_binary("GreaterOrEqual", ">=", left, right, verbosity),
            AnalysisKind::LogicalNot(target) => match verbosity {
                Verbosity::Minimal => format!("!{}", target.format(verbosity)).into(),
                Verbosity::Detailed => format!("LogicalNot({})", target.format(verbosity)).into(),
                Verbosity::Debug => format!("LogicalNot {{\n{}\n}}", format!("target: {}", target.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::BitwiseNot(value) => match verbosity {
                Verbosity::Minimal => format!("~{}", value.format(verbosity)).into(),
                Verbosity::Detailed => format!("BitwiseNot({})", value.format(verbosity)).into(),
                Verbosity::Debug => format!("BitwiseNot {{\n{}\n}}", format!("target: {}", value.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::AddressOf(value) => match verbosity {
                Verbosity::Minimal => format!("&{}", value.format(verbosity)).into(),
                Verbosity::Detailed => format!("AddressOf({})", value.format(verbosity)).into(),
                Verbosity::Debug => format!("AddressOf {{\n{}\n}}", format!("target: {}", value.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::Dereference(value) => match verbosity {
                Verbosity::Minimal => format!("*{}", value.format(verbosity)).into(),
                Verbosity::Detailed => format!("Dereference({})", value.format(verbosity)).into(),
                Verbosity::Debug => format!("Dereference {{\n{}\n}}", format!("target: {}", value.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::Index(index) => index.format(verbosity),
            AnalysisKind::Invoke(invoke) => invoke.format(verbosity),
            AnalysisKind::Block(block) => match verbosity {
                Verbosity::Minimal => format!("{{ {} }}", block.format(verbosity)).into(),
                Verbosity::Detailed => format!("Block({})", block.format(verbosity)).into(),
                Verbosity::Debug => format!("Block {{\n{}\n}}", block.format(verbosity).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::Conditional(condition, then, alternate) => match verbosity {
                Verbosity::Minimal => format!(
                    "if {} {{ {} }} else {{ {} }}",
                    condition.format(verbosity),
                    then.format(verbosity),
                    alternate.format(verbosity)
                ).into(),
                Verbosity::Detailed => format!(
                    "Conditional({}, {}, {})",
                    condition.format(verbosity),
                    then.format(verbosity),
                    alternate.format(verbosity)
                ).into(),
                Verbosity::Debug => format!(
                    "Conditional {{\n{},\n{},\n{}\n}}",
                    format!("condition: {}", condition.format(verbosity)).indent(verbosity),
                    format!("then: {}", then.format(verbosity)).indent(verbosity),
                    format!("alternate: {}", alternate.format(verbosity)).indent(verbosity)
                ).into(),
                _ => "".into(),
            },
            AnalysisKind::While(condition, then) => match verbosity {
                Verbosity::Minimal => format!("while {} {{ {} }}", condition.format(verbosity), then.format(verbosity)).into(),
                Verbosity::Detailed => format!("While({}, {})", condition.format(verbosity), then.format(verbosity)).into(),
                Verbosity::Debug => format!(
                    "While {{\n{},\n{}\n}}",
                    format!("condition: {}", condition.format(verbosity)).indent(verbosity),
                    format!("then: {}", then.format(verbosity)).indent(verbosity)
                ).into(),
                _ => "".into(),
            },
            AnalysisKind::Return(value) => match verbosity {
                Verbosity::Minimal => format!("return {}", if let Some(v) = value { v.format(verbosity).to_string() } else { "".to_string() }).into(),
                Verbosity::Detailed => format!("Return({})", value.format(verbosity)).into(),
                Verbosity::Debug => format!("Return {{\n{}\n}}", format!("value: {}", value.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::Break(value) => match verbosity {
                Verbosity::Minimal => format!("break {}", if let Some(v) = value { v.format(verbosity).to_string() } else { "".to_string() }).into(),
                Verbosity::Detailed => format!("Break({})", value.format(verbosity)).into(),
                Verbosity::Debug => format!("Break {{\n{}\n}}", format!("value: {}", value.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::Continue(value) => match verbosity {
                Verbosity::Minimal => format!("continue {}", if let Some(v) = value { v.format(verbosity).to_string() } else { "".to_string() }).into(),
                Verbosity::Detailed => format!("Continue({})", value.format(verbosity)).into(),
                Verbosity::Debug => format!("Continue {{\n{}\n}}", format!("value: {}", value.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::Usage(target) => match verbosity {
                Verbosity::Minimal => target.format(verbosity),
                Verbosity::Detailed => format!("Usage({})", target.format(verbosity)).into(),
                Verbosity::Debug => format!("Usage {{\n{}\n}}", format!("target: {}", target.format(verbosity)).indent(verbosity)).into(),
                _ => "".into(),
            },
            AnalysisKind::Access(target, value) => match verbosity {
                Verbosity::Minimal => format!("{}.{}", target.format(verbosity), value.format(verbosity)).into(),
                Verbosity::Detailed => format!("Access({} | {})", target.format(verbosity), value.format(verbosity)).into(),
                Verbosity::Debug => format!(
                    "Access {{\n{},\n{}\n}}",
                    format!("target: {}", target.format(verbosity)).indent(verbosity),
                    format!("value: {}", value.format(verbosity)).indent(verbosity)
                ).into(),
                _ => "".into(),
            },
            AnalysisKind::Constructor(constructor) => constructor.format(verbosity),
            AnalysisKind::Assign(target, value) => match verbosity {
                Verbosity::Minimal => format!("{} = {}", target.format(verbosity), value.format(verbosity)).into(),
                Verbosity::Detailed => format!("Assign({} | {})", target.format(verbosity), value.format(verbosity)).into(),
                Verbosity::Debug => format!(
                    "Assign {{\n{},\n{}\n}}",
                    format!("target: {}", target.format(verbosity)).indent(verbosity),
                    format!("value: {}", value.format(verbosity)).indent(verbosity)
                ).into(),
                _ => "".into(),
            },
            AnalysisKind::Store(target, value) => match verbosity {
                Verbosity::Minimal => format!("*{} = {}", target.format(verbosity), value.format(verbosity)).into(),
                Verbosity::Detailed => format!("Store({} | {})", target.format(verbosity), value.format(verbosity)).into(),
                Verbosity::Debug => format!(
                    "Store {{\n{},\n{}\n}}",
                    format!("target: {}", target.format(verbosity)).indent(verbosity),
                    format!("value: {}", value.format(verbosity)).indent(verbosity)
                ).into(),
                _ => "".into(),
            },
            AnalysisKind::Binding(binding) => binding.format(verbosity),
            AnalysisKind::Structure(structure) => match verbosity {
                Verbosity::Minimal => format!("struct {}", structure.format(verbosity)).into(),
                Verbosity::Detailed => format!("Structure({})", structure.format(verbosity)).into(),
                Verbosity::Debug => format!("Structure {{\n{}\n}}", structure.format(verbosity).indent(verbosity)).into(),
                _ => "".into()
            },
            AnalysisKind::Union(union) => match verbosity {
                Verbosity::Minimal => format!("union {}", union.format(verbosity)).into(),
                Verbosity::Detailed => format!("Union({})", union.format(verbosity)).into(),
                Verbosity::Debug => format!("Union {{\n{}\n}}", union.format(verbosity).indent(verbosity)).into(),
                _ => "".into()
            },
            AnalysisKind::Enumeration(enumeration) => match verbosity {
                Verbosity::Minimal => format!("enum {}", enumeration.format(verbosity)).into(),
                Verbosity::Detailed => format!("Enumeration({})", enumeration.format(verbosity)).into(),
                Verbosity::Debug => format!("Enumeration {{\n{}\n}}", enumeration.format(verbosity).indent(verbosity)).into(),
                _ => "".into()
            },
            AnalysisKind::Function(function) => function.format(verbosity),
            AnalysisKind::Module(name, members) => match verbosity {
                Verbosity::Minimal => format!("mod {} {{ {} }}", name.format(verbosity), members.format(verbosity)).into(),
                Verbosity::Detailed => format!("Module({} | [{}])", name.format(verbosity), members.format(verbosity)).into(),
                Verbosity::Debug => format!(
                    "Module {{\n{},\n{}\n}}",
                    format!("name: {}", name.format(verbosity)).indent(verbosity),
                    format!("members: {}", members.format(verbosity)).indent(verbosity)
                ).into(),
                _ => "".into()
            },
        }
    }
}

fn format_binary<'a, L: Show<'a>, R: Show<'a>>(
    name: &str,
    op: &str,
    left: &L,
    right: &R,
    verbosity: Verbosity
) -> Str<'a> {
    match verbosity {
        Verbosity::Minimal => format!("{} {} {}", left.format(verbosity), op, right.format(verbosity)).into(),
        Verbosity::Detailed => format!("{}({}, {})", name, left.format(verbosity), right.format(verbosity)).into(),
        Verbosity::Debug => format!(
            "{} {{\n{},\n{}\n}}",
            name,
            format!("left: {}", left.format(verbosity)).indent(verbosity),
            format!("right: {}", right.format(verbosity)).indent(verbosity)
        ).into(),
        _ => "".into(),
    }
}
