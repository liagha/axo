use {
    crate::{
        data::Str,
        format::Show,
        initializer::Preference,
        tracker::{Span, Spanned},
    }
};

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
                self.format(verbosity - 1).to_string()
            }
        }.into()
    }
}

impl<'preference> Spanned<'preference> for Preference<'preference> {
    fn borrow_span(&self) -> Span<'preference> {
        self.span.clone()
    }

    fn span(self) -> Span<'preference> {
        self.span
    }
}
