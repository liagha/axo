use {
    crate::{
        data::Str,
        internal::hash::Set,
        parser::{Element, ElementKind, Symbol, SymbolKind},
        scanner::{OperatorKind, Token, TokenKind},
        schema::*,
        tracker::{Location, Position, Span},
        resolver::{
            Resolver,
            scope::Scope,
        },
    }
};

pub trait Sugared<'sugared, Output> {
    fn desugar(&self) -> Output;
}

impl<'element> Sugared<'element, Element<'element>> for Element<'element> {
    fn desugar(&self) -> Element<'element> {
        match &self.kind {
            ElementKind::Literal(literal) => literal.desugar(),
            ElementKind::Unary(unary) => {
                let operand = Box::new(unary.operand.desugar());
                let operator = match &unary.operator.kind {
                    TokenKind::Operator(operator) => operator.as_slice(),
                    _ => return Element::new(ElementKind::Unary(Unary::new(unary.operator.clone(), operand)), self.span),
                };

                let method = match operator {
                    [OperatorKind::Minus] => "negate",
                    [OperatorKind::Exclamation] => "not",
                    [OperatorKind::Tilde] => "bitwise_not",
                    _ => return Element::new(ElementKind::Unary(Unary::new(unary.operator.clone(), operand)), self.span),
                };

                let member = Element::new(
                    ElementKind::Invoke(Invoke::new(
                        Box::new(Element::new(
                            ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from(method)), self.span)),
                            self.span,
                        )),
                        vec![],
                    )),
                    self.span,
                );

                Element::new(
                    ElementKind::Binary(Binary::new(
                        operand,
                        Token::new(TokenKind::Operator(OperatorKind::Dot), self.span),
                        Box::new(member)
                    )),
                    self.span
                )
            }
            ElementKind::Binary(binary) => {
                let left = Box::new(binary.left.desugar());
                let right = Box::new(binary.right.desugar());
                let operator = match &binary.operator.kind {
                    TokenKind::Operator(operator) => operator.as_slice(),
                    _ => return Element::new(ElementKind::Binary(Binary::new(left, binary.operator.clone(), right)), self.span),
                };

                let method = match operator {
                    [OperatorKind::Dot] => {
                        return Element::new(ElementKind::Binary(Binary::new(left, binary.operator.clone(), right)), self.span)
                    }
                    [OperatorKind::Equal] => "assign",
                    [OperatorKind::Plus] => "add",
                    [OperatorKind::Minus] => "subtract",
                    [OperatorKind::Star] => "multiply",
                    [OperatorKind::Slash] => "divide",
                    [OperatorKind::Percent] => "modulus",
                    [OperatorKind::Ampersand, OperatorKind::Ampersand] => "and",
                    [OperatorKind::Pipe, OperatorKind::Pipe] => "or",
                    [OperatorKind::Caret] => "xor",
                    [OperatorKind::Ampersand] => "bitwise_and",
                    [OperatorKind::Pipe] => "bitwise_or",
                    [OperatorKind::LeftAngle, OperatorKind::LeftAngle] => "shift_left",
                    [OperatorKind::RightAngle, OperatorKind::RightAngle] => "shift_right",
                    [OperatorKind::Equal, OperatorKind::Equal] => "equal",
                    [OperatorKind::Exclamation, OperatorKind::Equal] => "not_equal",
                    [OperatorKind::LeftAngle] => "less",
                    [OperatorKind::LeftAngle, OperatorKind::Equal] => "less_or_equal",
                    [OperatorKind::RightAngle] => "greater",
                    [OperatorKind::RightAngle, OperatorKind::Equal] => "greater_or_equal",
                    _ => return Element::new(ElementKind::Binary(Binary::new(left, binary.operator.clone(), right)), self.span),
                };

                let method = Box::new(Element::new(
                    ElementKind::Invoke(Invoke::new(
                        Box::new(Element::new(
                            ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from(method)), self.span)),
                            self.span,
                        )),
                        vec![*right],
                    )),
                    self.span,
                ));

                Element::new(
                    ElementKind::Binary(Binary::new(
                        left,
                        Token::new(TokenKind::Operator(OperatorKind::Dot), self.span),
                        method
                    )),
                    self.span
                )
            }
            ElementKind::Index(index) => {
                let target = Box::new(index.target.desugar());
                let members = index.members.iter().map(|member| member.desugar()).collect();
                Element::new(ElementKind::Index(Index::new(target, members)), self.span)
            }
            ElementKind::Invoke(invoke) => {
                let target = Box::new(invoke.target.desugar());
                let members = invoke.members.iter().map(|member| member.desugar()).collect();
                Element::new(ElementKind::Invoke(Invoke::new(target, members)), self.span)
            }
            ElementKind::Construct(construct) => {
                let target = Box::new(construct.target.desugar());
                let members = construct.members.iter().map(|member| member.desugar()).collect();
                Element::new(ElementKind::Construct(Structure::new(target, members)), self.span)
            }
            ElementKind::Delimited(delimited) => {
                let items = delimited.items.iter().map(|item| item.desugar()).collect();
                Element::new(
                    ElementKind::Delimited(Delimited::new(
                        delimited.start.clone(),
                        items,
                        delimited.separator.clone(),
                        delimited.end.clone(),
                    )),
                    self.span,
                )
            }
            ElementKind::Conditional(conditional) => {
                let condition = Box::new(conditional.condition.desugar());
                let then = Box::new(conditional.then.desugar());
                let alternate = conditional.alternate.as_ref().map(|alternate| Box::new(alternate.desugar()));
                Element::new(ElementKind::Conditional(Conditional::new(condition, then, alternate)), self.span)
            }
            ElementKind::While(repeat) => {
                let condition = repeat.condition.as_ref().map(|condition| Box::new(condition.desugar()));
                let body = Box::new(repeat.body.desugar());
                Element::new(ElementKind::While(While::new(condition, body)), self.span)
            }
            ElementKind::Cycle(cycle) => {
                let clause = Box::new(cycle.clause.desugar());
                let body = Box::new(cycle.body.desugar());
                Element::new(ElementKind::Cycle(Cycle::new(clause, body)), self.span)
            }
            ElementKind::Return(value) => {
                let value = value.as_ref().map(|value| Box::new(value.desugar()));
                Element::new(ElementKind::Return(value), self.span)
            }
            ElementKind::Break(value) => {
                let value = value.as_ref().map(|value| Box::new(value.desugar()));
                Element::new(ElementKind::Break(value), self.span)
            }
            ElementKind::Continue(value) => {
                let value = value.as_ref().map(|value| Box::new(value.desugar()));
                Element::new(ElementKind::Continue(value), self.span)
            }
            ElementKind::Symbolize(symbol) => {
                Element::new(ElementKind::Symbolize(symbol.desugar()), self.span)
            }
            ElementKind::Procedural(procedural) => {
                let body = Box::new(procedural.body.desugar());
                Element::new(ElementKind::Procedural(Procedural::new(body)), self.span)
            }
            _ => self.clone(),
        }
    }
}

impl<'token> Sugared<'token, Element<'token>> for Token<'token> {
    fn desugar(&self) -> Element<'token> {
        let span = self.span;

        match &self.kind {
            TokenKind::Float(_) => Element::new(
                ElementKind::Construct(Structure::new(
                    Box::new(Element::new(
                        ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from("Float")), span)),
                        span,
                    )),
                    vec![
                        Element::new(ElementKind::Literal(self.clone()), span),
                        Element::new(ElementKind::Literal(Token::new(TokenKind::Integer(32), span)), span),
                    ],
                )),
                span,
            ),
            TokenKind::Integer(_) => Element::new(
                ElementKind::Construct(Structure::new(
                    Box::new(Element::new(
                        ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from("Integer")), span)),
                        span,
                    )),
                    vec![
                        Element::new(ElementKind::Literal(self.clone()), span),
                        Element::new(ElementKind::Literal(Token::new(TokenKind::Integer(32), span)), span),
                        Element::new(ElementKind::Literal(Token::new(TokenKind::Boolean(true), span)), span),
                    ],
                )),
                span,
            ),
            TokenKind::Boolean(_) => Element::new(
                ElementKind::Construct(Structure::new(
                    Box::new(Element::new(
                        ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from("Boolean")), span)),
                        span,
                    )),
                    vec![Element::new(ElementKind::Literal(self.clone()), span)],
                )),
                span,
            ),
            TokenKind::String(_) => Element::new(
                ElementKind::Construct(Structure::new(
                    Box::new(Element::new(
                        ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from("String")), span)),
                        span,
                    )),
                    vec![Element::new(ElementKind::Literal(self.clone()), span)],
                )),
                span,
            ),
            TokenKind::Character(_) => Element::new(
                ElementKind::Construct(Structure::new(
                    Box::new(Element::new(
                        ElementKind::Literal(Token::new(TokenKind::Identifier(Str::from("Character")), span)),
                        span,
                    )),
                    vec![Element::new(ElementKind::Literal(self.clone()), span)],
                )),
                span,
            ),
            _ => Element::new(ElementKind::Literal(self.clone()), span),
        }
    }
}

impl<'symbol> Sugared<'symbol, Symbol<'symbol>> for Symbol<'symbol> {
    fn desugar(&self) -> Symbol<'symbol> {
        match &self.kind {
            SymbolKind::Inclusion(inclusion) => {
                let mut inclusion = inclusion.clone();
                inclusion.target = Box::new(inclusion.target.desugar());
                Symbol::new(SymbolKind::Inclusion(inclusion), self.span, self.id)
            }
            SymbolKind::Extension(extension) => {
                let mut extension = extension.clone();
                extension.target = Box::new(extension.target.desugar());
                extension.members = extension.members.into_iter().map(|member| member.desugar()).collect();
                Symbol::new(SymbolKind::Extension(extension), self.span, self.id)
            }
            SymbolKind::Binding(binding) => {
                let mut binding = binding.clone();
                binding.value = binding.value.map(|value| Box::new(value.desugar()));
                Symbol::new(SymbolKind::Binding(binding), self.span, self.id)
            }
            SymbolKind::Structure(structure) => {
                let mut structure = structure.clone();
                structure.members = structure.members.into_iter().map(|member| member.desugar()).collect();
                Symbol::new(SymbolKind::Structure(structure), self.span, self.id)
            }
            SymbolKind::Enumeration(enumeration) => {
                let mut enumeration = enumeration.clone();
                enumeration.members = enumeration.members.into_iter().map(|member| member.desugar()).collect();
                Symbol::new(SymbolKind::Enumeration(enumeration), self.span, self.id)
            }
            SymbolKind::Method(method) => {
                let mut method = method.clone();
                method.members = method.members.into_iter().map(|member| member.desugar()).collect();
                method.body = Box::new(method.body.desugar());
                method.output = method.output.map(|output| Box::new(output.desugar()));
                Symbol::new(SymbolKind::Method(method), self.span, self.id)
            }
            _ => Symbol::new(self.kind.clone(), self.span, self.id),
        }
    }
}