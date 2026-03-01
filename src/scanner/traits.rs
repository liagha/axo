use {
    super::{Character, Token, TokenKind},
    crate::{
        data::Str,
        format::Show,
        tracker::{Span, Spanned},
    },
};

impl<'token> Show<'token> for TokenKind<'token> {
    type Verbosity = u16;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'token> {
        match verbosity { 
            0 => {
                match self {
                    TokenKind::Boolean(boolean) => format!("{}", boolean),
                    TokenKind::Float(number) => format!("{}", number),
                    TokenKind::Integer(number) => format!("{}", number),
                    TokenKind::Operator(operator) => format!("{:?}", operator),
                    TokenKind::Punctuation(punctuation) => format!("{:?}", punctuation),
                    TokenKind::Identifier(identifier) => format!("{}", identifier),
                    TokenKind::String(string) => format!("\"{}\"", string),
                    TokenKind::Character(character) => format!("'{}'", character),
                    TokenKind::Comment(comment) => format!("//{}", comment),
                }
            }  
            
            1 => {
                match self {
                    TokenKind::Boolean(boolean) => format!("Boolean({})", boolean),
                    TokenKind::Float(number) => format!("Float({})", number),
                    TokenKind::Integer(number) => format!("Integer({})", number),
                    TokenKind::Operator(operator) => format!("Operator({:?})", operator),
                    TokenKind::Punctuation(punctuation) => format!("Punctuation({:?})", punctuation),
                    TokenKind::Identifier(identifier) => format!("Identifier({})", identifier),
                    TokenKind::String(string) => format!("String({})", string),
                    TokenKind::Character(character) => format!("Character('{}')", character),
                    TokenKind::Comment(comment) => format!("Comment({})", comment),
                }
            }
            
            _ => {
                unimplemented!("the verbosity wasn't implemented for TokenKind.");
            }
        }.into()
    }
}

impl<'token> Show<'token> for Token<'token> {
    type Verbosity = u16;
    
    fn format(&self, verbosity: Self::Verbosity) -> Str<'token> {
        match verbosity { 
            0 => {
                format!("{}", self.kind.format(verbosity))
            }
            
            1 => {
                format!("{}", self.kind.format(verbosity))
            }
            
            2 => {
                format!("{:?} | {:?}", self.kind.format(verbosity), self.span)
            }

            _ => {
                unimplemented!("the verbosity wasn't implemented for Token.");
            }
        }.into()
    }
}

impl<'token> PartialEq for Token<'token> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'token> Eq for Token<'token> {}

impl<'character> Spanned<'character> for Character<'character> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'character> {
        self.span
    }

    #[track_caller]
    fn span(self) -> Span<'character> {
        self.span
    }
}

impl<'token> Spanned<'token> for Token<'token> {
    #[track_caller]
    fn borrow_span(&self) -> Span<'token> {
        self.span
    }

    #[track_caller]
    fn span(self) -> Span<'token> {
        self.span
    }
}
