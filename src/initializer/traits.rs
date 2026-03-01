use crate::data::Str;
use crate::format::Show;
use crate::initializer::Preference;

impl<'preference> Show<'preference> for Preference<'preference> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'preference> {
        match verbosity {
            0 => {
                "".to_string()
            }

            1 => {
                format!("Preference({}, {}, {:?})", self.target.format(verbosity), self.value.format(verbosity), self.span)
            }

            _ => {
                unimplemented!("the verbosity `{}` wasn't implemented for Preference.", verbosity);
            }
        }.into()
    }
}