use crate::{
    data::{Boolean, Char, Float, Integer, Str},
    internal::cache::{Decode, Encode},
    scanner::{Character, OperatorKind, PunctuationKind, Token, TokenKind},
    tracker::{Span, Spanned},
};

impl<'token> PartialEq for Token<'token> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'token> Eq for Token<'token> {}

impl<'character> Spanned<'character> for Character<'character> {
    #[track_caller]
    fn span(&self) -> Span<'character> {
        self.span
    }
}

impl<'token> Spanned<'token> for Token<'token> {
    #[track_caller]
    fn span(&self) -> Span<'token> {
        self.span
    }
}

impl<'token> Encode for Token<'token> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.kind.encode(buffer);
        self.span.encode(buffer);
    }
}

impl<'token> Decode<'token> for Token<'token> {
    fn decode(buffer: &'token [u8], cursor: &mut usize) -> Self {
        Token {
            kind: TokenKind::decode(buffer, cursor),
            span: Span::decode(buffer, cursor),
        }
    }
}

impl<'token> Encode for TokenKind<'token> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            TokenKind::Float(v) => {
                buffer.push(0);
                v.0.encode(buffer);
            }
            TokenKind::Integer(v) => {
                buffer.push(1);
                v.encode(buffer);
            }
            TokenKind::Boolean(v) => {
                buffer.push(2);
                v.encode(buffer);
            }
            TokenKind::String(v) => {
                buffer.push(3);
                v.encode(buffer);
            }
            TokenKind::Character(v) => {
                buffer.push(4);
                v.encode(buffer);
            }
            TokenKind::Operator(v) => {
                buffer.push(5);
                v.encode(buffer);
            }
            TokenKind::Identifier(v) => {
                buffer.push(6);
                v.encode(buffer);
            }
            TokenKind::Punctuation(v) => {
                buffer.push(7);
                v.encode(buffer);
            }
            TokenKind::Comment(v) => {
                buffer.push(8);
                v.encode(buffer);
            }
        }
    }
}

impl<'token> Decode<'token> for TokenKind<'token> {
    fn decode(buffer: &'token [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            0 => TokenKind::Float(Float(f64::decode(buffer, cursor))),
            1 => TokenKind::Integer(Integer::decode(buffer, cursor)),
            2 => TokenKind::Boolean(Boolean::decode(buffer, cursor)),
            3 => TokenKind::String(Str::decode(buffer, cursor)),
            4 => TokenKind::Character(Char::decode(buffer, cursor)),
            5 => TokenKind::Operator(OperatorKind::decode(buffer, cursor)),
            6 => TokenKind::Identifier(Str::decode(buffer, cursor)),
            7 => TokenKind::Punctuation(PunctuationKind::decode(buffer, cursor)),
            8 => TokenKind::Comment(Str::decode(buffer, cursor)),
            _ => panic!(),
        }
    }
}
