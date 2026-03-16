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
            Verbosity::Minimal => format!(
                "Binding({:?} | {}{}{})",
                self.kind,
                self.target.format(verbosity),
                format!(" : {}", self.annotation.format(verbosity)),
                if let Some(value) = &self.value {
                    format!(" = {}", value.format(verbosity))
                } else {
                    "".to_string()
                }
            )
                .into(),

            _ => self.format(verbosity.fallback()),
        }
    }
}

impl<'show, Target: Show<'show>, Member: Show<'show>> Show<'show>
for Aggregate<Target, Member>
{
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Minimal => format!(
                "({})[{}]",
                self.target.format(verbosity),
                self.members.format(verbosity)
            )
                .into(),

            _ => self.format(verbosity.fallback()),
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
            Verbosity::Minimal => format!(
                "Function({}{} : {})[{}]{{ {} }}",
                format!("{:?} | ", self.interface),
                self.target.format(verbosity),
                self.output.format(verbosity),
                self.members.format(verbosity),
                self.body.format(verbosity)
            )
                .into(),

            _ => self.format(verbosity.fallback()),
        }
    }
}

impl<'show, Target: Show<'show>> Show<'show>
for Module<Target>
{
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Minimal => format!(
                "Module({})",
                self.target.format(verbosity),
            )
                .into(),

            _ => self.format(verbosity.fallback()),
        }
    }
}

impl<'show, Delimiter: Show<'show>, Member: Show<'show>> Show<'show> for Delimited<Delimiter, Member> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Minimal => {
                format!(
                    "Delimited({} | {})[{}]({})",
                    self.start.format(verbosity),
                    self.separator.format(verbosity),
                    self.members.format(verbosity),
                    self.end.format(verbosity),
                ).into()
            }

            _ => {
                self.format(verbosity.fallback())
            }
        }
    }
}

impl<'show, Left: Show<'show>, Operator: Show<'show>, Right: Show<'show>> Show<'show> for Binary<Left, Operator, Right> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Minimal => {
                format!(
                    "Binary({} {} {})",
                    self.left.format(verbosity),
                    self.operator.format(verbosity),
                    self.right.format(verbosity)
                ).into()
            }

            _ => {
                self.format(verbosity.fallback())
            }
        }
    }
}

impl<'show, Operator: Show<'show>, Operand: Show<'show>> Show<'show> for Unary<Operator, Operand> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Minimal => {
                format!(
                    "Unary({} {})",
                    self.operator.format(verbosity),
                    self.operand.format(verbosity)
                ).into()
            }

            _ => {
                self.format(verbosity.fallback())
            }
        }
    }
}

impl<'show, Target: Show<'show>, Member: Show<'show>> Show<'show> for Index<Target, Member> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Minimal => {
                format!(
                    "Index({})[{}]",
                    self.target.format(verbosity),
                    self.members.format(verbosity),
                ).into()
            }

            _ => {
                self.format(verbosity.fallback())
            }
        }
    }
}

impl<'show, Target: Show<'show>, Member: Show<'show>> Show<'show> for Invoke<Target, Member> {
    fn format(&self, verbosity: Verbosity) -> Str<'show> {
        match verbosity {
            Verbosity::Minimal => {
                format!(
                    "Invoke({})[{}]",
                    self.target.format(verbosity),
                    self.members.format(verbosity),
                ).into()
            }

            _ => {
                self.format(verbosity.fallback())
            }
        }
    }
}
