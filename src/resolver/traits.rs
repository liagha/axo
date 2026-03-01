use crate::data::Str;
use crate::format::Show;
use crate::resolver::scope::Scope;

impl<'scope> Show<'scope> for Scope<'scope> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'scope> {
        match verbosity {
            0 => {
                format!("{}", self.symbols.format(0))
            }

            _ => {
                unimplemented!("the verbosity `{}` wasn't implemented for Scope.", verbosity);
            }
        }.into()
    }
}