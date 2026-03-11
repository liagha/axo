use crate::data::Str;
use crate::format::Show;
use crate::parser::Symbol;
use crate::resolver::scope::Scope;

impl<'scope> Show<'scope> for Scope<Symbol<'scope>> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'scope> {
        match verbosity {
            _ => {
                format!("{}", self.symbols.format(verbosity))
            }
        }.into()
    }
}