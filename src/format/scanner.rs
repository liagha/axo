use crate::{
    format::{Show, Stencil},
    scanner::{Character, Token, TokenKind},
};

impl<'character> Show<'character> for Character {
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Character")
            .field("value", self.value.to_string())
            .field("span", format!("{:?}", self.span))
    }
}

impl<'token> Show<'token> for Token<'token> {
    fn format(&self, config: Stencil) -> Stencil {
        config
            .clone()
            .new("Token")
            .field("kind", self.kind.format(config.clone()))
    }
}

impl<'token> Show<'token> for TokenKind<'token> {
    fn format(&self, config: Stencil) -> Stencil {
        let base = config.clone().new("TokenKind");
        match self {
            TokenKind::Boolean(boolean) => {
                base.variant("Boolean").field("value", boolean.to_string())
            }
            TokenKind::Float(number) => base.variant("Float").field("value", number.to_string()),
            TokenKind::Integer(number) => {
                base.variant("Integer").field("value", number.to_string())
            }
            TokenKind::Operator(operator) => base
                .variant("Operator")
                .field("value", format!("{:?}", operator)),
            TokenKind::Punctuation(punctuation) => base
                .variant("Punctuation")
                .field("value", format!("{:?}", punctuation)),
            TokenKind::Identifier(identifier) => base
                .variant("Identifier")
                .field("value", identifier.to_string()),
            TokenKind::String(string) => base
                .variant("String")
                .field("value", format!("\"{}\"", string)),
            TokenKind::Character(character) => base
                .variant("Character")
                .field("value", format!("'{}'", character)),
            TokenKind::Comment(comment) => base
                .variant("Comment")
                .field("value", format!("\"{}\"", comment)),
        }
    }
}
