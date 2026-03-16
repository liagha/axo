use {
    crate::{
        data::*,
        format::Show,
    }
};

impl<
    'show,
    Target: Show<'show, Verbosity = u8>,
    Value: Show<'show, Verbosity = u8>,
    Type: Show<'show, Verbosity = u8>,
> Show<'show> for Binding<Target, Value, Type>
{
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => format!(
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

            _ => self.format(verbosity - 1),
        }
    }
}

impl<'show, Target: Show<'show, Verbosity= u8>, Member: Show<'show, Verbosity= u8>> Show<'show>
for Aggregate<Target, Member>
{
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => format!(
                "({})[{}]",
                self.target.format(verbosity),
                self.members.format(verbosity)
            )
                .into(),

            _ => self.format(verbosity - 1),
        }
    }
}

impl<
    'show,
    Target: Show<'show, Verbosity= u8>,
    Parameter: Show<'show, Verbosity= u8>,
    Body: Show<'show, Verbosity= u8>,
    Output: Show<'show, Verbosity= u8>,
> Show<'show> for Function<Target, Parameter, Body, Output>
{
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => format!(
                "Function({}{} : {})[{}]{{ {} }}",
                format!("{:?} | ", self.interface),
                self.target.format(verbosity),
                self.output.format(verbosity),
                self.members.format(verbosity),
                self.body.format(verbosity)
            )
                .into(),

            _ => self.format(verbosity - 1),
        }
    }
}

impl<'show, Target: Show<'show, Verbosity= u8>> Show<'show>
for Module<Target>
{
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => format!(
                "Module({})",
                self.target.format(verbosity),
            )
                .into(),

            _ => self.format(verbosity - 1),
        }
    }
}

impl<'show, Delimiter: Show<'show, Verbosity = u8>, Member: Show<'show, Verbosity = u8>> Show<'show> for Delimited<Delimiter, Member> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => {
                format!(
                    "Delimited({} | {})[{}]({})",
                    self.start.format(verbosity),
                    self.separator.format(verbosity),
                    self.members.format(verbosity),
                    self.end.format(verbosity),
                ).into()
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}

impl<'show, Left: Show<'show, Verbosity = u8>, Operator: Show<'show, Verbosity = u8>, Right: Show<'show, Verbosity = u8>> Show<'show> for Binary<Left, Operator, Right> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => {
                format!(
                    "Binary({} {} {})",
                    self.left.format(verbosity),
                    self.operator.format(verbosity),
                    self.right.format(verbosity)
                ).into()
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}

impl<'show, Operator: Show<'show, Verbosity = u8>, Operand: Show<'show, Verbosity = u8>> Show<'show> for Unary<Operator, Operand> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => {
                format!(
                    "Unary({} {})",
                    self.operator.format(verbosity),
                    self.operand.format(verbosity)
                ).into()
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}

impl<'show, Target: Show<'show, Verbosity = u8>, Member: Show<'show, Verbosity = u8>> Show<'show> for Index<Target, Member> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => {
                format!(
                    "Index({})[{}]",
                    self.target.format(verbosity),
                    self.members.format(verbosity),
                ).into()
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}

impl<'show, Target: Show<'show, Verbosity = u8>, Member: Show<'show, Verbosity = u8>> Show<'show> for Invoke<Target, Member> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'show> {
        match verbosity {
            0 => {
                format!(
                    "Invoke({})[{}]",
                    self.target.format(verbosity),
                    self.members.format(verbosity),
                ).into()
            }

            _ => {
                self.format(verbosity - 1)
            }
        }
    }
}
