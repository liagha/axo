use {
    crate::{
        axo_lexer::{
            Token, TokenKind,
            OperatorKind, PunctuationKind,
        },
        axo_parser::{
            delimiter::Delimiter,
            item::ItemParser,
            error::ErrorKind,
            Element, ElementKind,
            ParseError, Parser,
            ControlFlow
        },
        axo_span::Span,
    }
};
use crate::axo_data::peekable::Peekable;

pub type ParseFunction = fn(&mut Parser) -> Element;

pub trait Primary {
    fn parse_token(&mut self) -> Element;
    fn parse_primary(&mut self) -> Element;
    fn parse_term(&mut self) -> Element;

    fn parse_unary(&mut self, function: ParseFunction) -> Element;

    fn parse_binary(&mut self, function: ParseFunction, minimum: u8) -> Element;

    fn parse_basic(&mut self) -> Element {
        self.parse_binary(Parser::parse_primary, 0)
    }

    fn parse_complex(&mut self) -> Element {
        self.parse_binary(Parser::parse_term, 0)
    }
}

impl Primary for Parser {
    fn parse_token(&mut self) -> Element {
        let token = self.next().unwrap();
        let Token { kind, span } = token.clone();

        let element = match kind {
            TokenKind::Identifier(ident) => Element {
                kind: ElementKind::Identifier(ident),
                span,
            },
            _ => {
                let Token { kind, span } = token;

                Element {
                    kind: ElementKind::Literal(kind),
                    span,
                }
            },
        };

        element
    }

    fn parse_primary(&mut self) -> Element {
        if let Some(token) = self.peek().cloned() {
            let Token { kind, span } = token.clone();

            match kind {
                TokenKind::Punctuation(PunctuationKind::LeftBrace) => self.parse_braced(Self::parse_complex),
                TokenKind::Punctuation(PunctuationKind::LeftBracket) => self.parse_bracketed(Self::parse_complex),
                TokenKind::Punctuation(PunctuationKind::LeftParen) => self.parse_parenthesized(Self::parse_complex),
                TokenKind::Identifier(_)
                | TokenKind::String(_)
                | TokenKind::Character(_)
                | TokenKind::Boolean(_)
                | TokenKind::Float(_)
                | TokenKind::Integer(_)
                | TokenKind::Operator(_) => {
                    let element = self.parse_token();

                    element
                }

                kind => {
                    self.next();
                    self.error(&ParseError::new(ErrorKind::UnexpectedToken(kind), span))
                },
            }
        } else {
            self.error(&ParseError::new(ErrorKind::UnexpectedEndOfFile, self.current_span()))
        }
    }

    fn parse_term(&mut self) -> Element {
        if let Some(token) = self.peek().cloned() {
            let Token { kind, span } = token.clone();

            match kind {
                TokenKind::Identifier(ref ident) => match ident.as_str() {
                    "var" | "const" => self.parse_variable(),
                    "procedural" => self.parse_procedural(),
                    "if" => self.parse_conditional(),
                    "loop" => self.parse_loop(),
                    "while" => self.parse_while(),
                    "for" => self.parse_for(),
                    "fn" => self.parse_function(),
                    "macro" => self.parse_macro(),
                    "use" => self.parse_use(),
                    "return" => self.parse_return(),
                    "break" => self.parse_break(),
                    "continue" => self.parse_continue(),
                    "struct" => self.parse_struct(),
                    "enum" => self.parse_enum(),
                    "impl" => self.parse_impl(),
                    "trait" => self.parse_trait(),
                    "match" => self.parse_match(),
                    "else" => {
                        self.next();

                        self.error(&ParseError::new(ErrorKind::DanglingElse, span))
                    },
                    _ => {
                        let element = self.parse_token();

                        element
                    },
                },
                TokenKind::String(_)
                | TokenKind::Character(_)
                | TokenKind::Boolean(_)
                | TokenKind::Float(_)
                | TokenKind::Integer(_)
                | TokenKind::Operator(_) => {
                    let element = self.parse_token();

                    element
                }
                _ => self.parse_primary()
            }
        } else {
            self.error(&ParseError::new(ErrorKind::UnexpectedEndOfFile, self.current_span()))
        }
    }
    fn parse_unary(&mut self, function: ParseFunction) -> Element {
        if let Some(Token {
                        kind: TokenKind::Operator(operator),
                        span: Span { start, .. },
                    }) = self.peek().cloned()
        {
            if operator.is_prefix() {
                let operator = self.next().unwrap();

                if self.peek().is_none() {
                    return self.error(&ParseError::new(
                        ErrorKind::MissingOperand,
                        operator.span
                    ));
                }

                let unary = self.parse_unary(function);
                let end = unary.span.end.clone();

                let span = self.span(start, end);

                let kind = ElementKind::Unary {
                    operator,
                    operand: unary.into()
                };

                let element = Element { kind, span };

                return element;
            }
        }

        let mut element = function(self);

        while let Some(Token {
                           kind: TokenKind::Operator(operator),
                           span: Span { end, .. },
                       }) = self.peek().cloned()
        {

            if operator.is_postfix() {
                let operator = self.next().unwrap();
                let span = self.span(element.span.start.clone(), end);

                let kind = ElementKind::Unary {
                    operator,
                    operand: element.into()
                };

                element = Element { kind, span };
            } else {
                break;
            }
        }

        element
    }

    fn parse_binary(&mut self, function: ParseFunction, minimum: u8) -> Element {
        let mut left = self.parse_unary(function);

        while let Some(Token { kind: TokenKind::Operator(op), .. }) = self.peek().cloned() {
            let precedence = op.precedence();

            if let Some(precedence) = precedence {
                if precedence < minimum {
                    break;
                }

                let operator = self.next().unwrap();

                if self.peek().is_none() {
                    return self.error(&ParseError::new(
                        ErrorKind::MissingOperand,
                        operator.span
                    ));
                }

                let right = self.parse_binary(function, precedence + 1);

                let start = left.span.start.clone();
                let end = right.span.end.clone();
                let span = self.span(start, end);

                let kind = ElementKind::Binary {
                    left: left.into(),
                    operator,
                    right: right.into()
                };

                left = Element::new(kind, span);
            } else {
                break;
            }
        }

        left
    }
}
