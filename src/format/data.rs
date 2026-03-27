use crate::{
    data::*,
    format::{Show, Stencil},
};

impl<'show, Target: Show<'show>, Value: Show<'show>, Type: Show<'show>> Show<'show>
    for Binding<Target, Value, Type>
{
    fn format(&self, config: Stencil) -> Stencil {
        let mut stencil = config
            .clone()
            .new("Binding")
            .field("kind", format!("{:?}", self.kind))
            .field("target", self.target.format(config.clone()))
            .field("annotation", self.annotation.format(config.clone()));

        if let Some(value) = &self.value {
            stencil = stencil.field("value", value.format(config.clone()));
        }

        stencil
    }
}

impl<'show, Target: Show<'show>, Member: Show<'show>> Show<'show> for Aggregate<Target, Member> {
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Aggregate")
            .field("target", self.target.format(config.clone()))
            .field("members", self.members.format(config.clone()))
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
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Function")
            .field("interface", format!("{:?}", self.interface))
            .field("target", self.target.format(config.clone()))
            .field("members", self.members.format(config.clone()))
            .field("output", self.output.format(config.clone()))
            .field("body", self.body.format(config.clone()))
    }
}

impl<'show, Target: Show<'show>> Show<'show> for Module<Target> {
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Module")
            .field("target", self.target.format(config.clone()))
    }
}

impl<'show, Delimiter: Show<'show>, Member: Show<'show>> Show<'show>
    for Delimited<Delimiter, Member>
{
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Delimited")
            .field("start", self.start.format(config.clone()))
            .field("separator", self.separator.format(config.clone()))
            .field("members", self.members.format(config.clone()))
            .field("end", self.end.format(config.clone()))
    }
}

impl<'show, Left: Show<'show>, Operator: Show<'show>, Right: Show<'show>> Show<'show>
    for Binary<Left, Operator, Right>
{
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Binary")
            .field("left", self.left.format(config.clone()))
            .field("operator", self.operator.format(config.clone()))
            .field("right", self.right.format(config.clone()))
    }
}

impl<'show, Operator: Show<'show>, Operand: Show<'show>> Show<'show> for Unary<Operator, Operand> {
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Unary")
            .field("operator", self.operator.format(config.clone()))
            .field("operand", self.operand.format(config.clone()))
    }
}

impl<'show, Target: Show<'show>, Member: Show<'show>> Show<'show> for Index<Target, Member> {
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Index")
            .field("target", self.target.format(config.clone()))
            .field("members", self.members.format(config.clone()))
    }
}

impl<'show, Target: Show<'show>, Member: Show<'show>> Show<'show> for Invoke<Target, Member> {
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Invoke")
            .field("target", self.target.format(config.clone()))
            .field("members", self.members.format(config.clone()))
    }
}
