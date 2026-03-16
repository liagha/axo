use {
    crate::{
        data::*,
        format::{Show, Verbosity},
    }
};

impl<
    'show,
    Target: Show<'show>,
    Value: Show<'show>,
    Type: Show<'show>,
> Show<'show> for Binding<Target, Value, Type>
{
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => format!(
                "let {}: {}{};",
                self.target.format(verbosity),
                self.annotation.format(verbosity),
                if let Some(value) = &self.value {
                    format!(" = {}", value.format(verbosity))
                } else {
                    "".to_string()
                }
            ).into(),
            Verbosity::Detailed => format!(
                "Binding({:?} | {} : {}{})",
                self.kind,
                self.target.format(verbosity),
                self.annotation.format(verbosity),
                if let Some(value) = &self.value {
                    format!(" = {}", value.format(verbosity))
                } else {
                    "".to_string()
                }
            ).into(),
            Verbosity::Debug => format!(
                "Binding {{\n{},\n{},\n{},\n{}\n}}",
                format!("kind: {:?}", self.kind).indent(verbosity),
                format!("target: {}", self.target.format(verbosity)).indent(verbosity),
                format!("annotation: {}", self.annotation.format(verbosity)).indent(verbosity),
                format!("value: {}", self.value.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}

impl<'show, Target: Show<'show>, Member: Show<'show>> Show<'show> for Aggregate<Target, Member> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => format!(
                "{} {{ {} }}",
                self.target.format(verbosity),
                self.members.format(verbosity)
            ).into(),
            Verbosity::Detailed => format!(
                "Aggregate({} | [{}])",
                self.target.format(verbosity),
                self.members.format(verbosity)
            ).into(),
            Verbosity::Debug => format!(
                "Aggregate {{\n{},\n{}\n}}",
                format!("target: {}", self.target.format(verbosity)).indent(verbosity),
                format!("members: {}", self.members.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}

impl<
    'show,
    Target: Show<'show>,
    Parameter: Show<'show>,
    Body: Show<'show>,
    Output: Show<'show>,
> Show<'show> for Function<Target, Parameter, Body, Output>
{
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => format!(
                "fn {}({}) -> {} {{ {} }}",
                self.target.format(verbosity),
                self.members.format(verbosity),
                self.output.format(verbosity),
                self.body.format(verbosity)
            ).into(),
            Verbosity::Detailed => format!(
                "Function({}{:?} | {} : {})[{}] {{ {} }}",
                self.target.format(verbosity),
                self.interface,
                self.members.format(verbosity),
                self.output.format(verbosity),
                self.members.format(verbosity),
                self.body.format(verbosity)
            ).into(),
            Verbosity::Debug => format!(
                "Function {{\n{},\n{},\n{},\n{},\n{}\n}}",
                format!("interface: {:?}", self.interface).indent(verbosity),
                format!("target: {}", self.target.format(verbosity)).indent(verbosity),
                format!("members: {}", self.members.format(verbosity)).indent(verbosity),
                format!("output: {}", self.output.format(verbosity)).indent(verbosity),
                format!("body: {}", self.body.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}

impl<'show, Target: Show<'show>> Show<'show> for Module<Target> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => format!("mod {};", self.target.format(verbosity)).into(),
            Verbosity::Detailed => format!("Module({})", self.target.format(verbosity)).into(),
            Verbosity::Debug => format!(
                "Module {{\n{}\n}}",
                format!("target: {}", self.target.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}

impl<'show, Delimiter: Show<'show>, Member: Show<'show>> Show<'show> for Delimited<Delimiter, Member> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => format!(
                "{}{}{}",
                self.start.format(verbosity),
                self.members.format(verbosity),
                self.end.format(verbosity),
            ).into(),
            Verbosity::Detailed => format!(
                "Delimited({} | {})[{}]({})",
                self.start.format(verbosity),
                self.separator.format(verbosity),
                self.members.format(verbosity),
                self.end.format(verbosity),
            ).into(),
            Verbosity::Debug => format!(
                "Delimited {{\n{},\n{},\n{},\n{}\n}}",
                format!("start: {}", self.start.format(verbosity)).indent(verbosity),
                format!("separator: {}", self.separator.format(verbosity)).indent(verbosity),
                format!("members: {}", self.members.format(verbosity)).indent(verbosity),
                format!("end: {}", self.end.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}

impl<'show, Left: Show<'show>, Operator: Show<'show>, Right: Show<'show>> Show<'show> for Binary<Left, Operator, Right> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => format!(
                "{} {} {}",
                self.left.format(verbosity),
                self.operator.format(verbosity),
                self.right.format(verbosity)
            ).into(),
            Verbosity::Detailed => format!(
                "Binary({} {} {})",
                self.left.format(verbosity),
                self.operator.format(verbosity),
                self.right.format(verbosity)
            ).into(),
            Verbosity::Debug => format!(
                "Binary {{\n{},\n{},\n{}\n}}",
                format!("left: {}", self.left.format(verbosity)).indent(verbosity),
                format!("operator: {}", self.operator.format(verbosity)).indent(verbosity),
                format!("right: {}", self.right.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}

impl<'show, Operator: Show<'show>, Operand: Show<'show>> Show<'show> for Unary<Operator, Operand> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => format!(
                "{}{}",
                self.operator.format(verbosity),
                self.operand.format(verbosity)
            ).into(),
            Verbosity::Detailed => format!(
                "Unary({} {})",
                self.operator.format(verbosity),
                self.operand.format(verbosity)
            ).into(),
            Verbosity::Debug => format!(
                "Unary {{\n{},\n{}\n}}",
                format!("operator: {}", self.operator.format(verbosity)).indent(verbosity),
                format!("operand: {}", self.operand.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}

impl<'show, Target: Show<'show>, Member: Show<'show>> Show<'show> for Index<Target, Member> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => format!(
                "{}[{}]",
                self.target.format(verbosity),
                self.members.format(verbosity),
            ).into(),
            Verbosity::Detailed => format!(
                "Index({} | [{}])",
                self.target.format(verbosity),
                self.members.format(verbosity),
            ).into(),
            Verbosity::Debug => format!(
                "Index {{\n{},\n{}\n}}",
                format!("target: {}", self.target.format(verbosity)).indent(verbosity),
                format!("members: {}", self.members.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}

impl<'show, Target: Show<'show>, Member: Show<'show>> Show<'show> for Invoke<Target, Member> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Off => "".into(),
            Verbosity::Minimal => format!(
                "{}({})",
                self.target.format(verbosity),
                self.members.format(verbosity),
            ).into(),
            Verbosity::Detailed => format!(
                "Invoke({} | ({}))",
                self.target.format(verbosity),
                self.members.format(verbosity),
            ).into(),
            Verbosity::Debug => format!(
                "Invoke {{\n{},\n{}\n}}",
                format!("target: {}", self.target.format(verbosity)).indent(verbosity),
                format!("members: {}", self.members.format(verbosity)).indent(verbosity)
            ).into(),
        }
    }
}
