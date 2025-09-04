use crate::data::Str;
use crate::internal::hash::Set;
use crate::parser::{Element, ElementKind, Symbol, Symbolic};
use crate::resolver::Resolver;
use crate::resolver::scope::Scope;
use crate::scanner::{OperatorKind, Token, TokenKind};
use crate::schema::*;
use crate::tracker::{Location, Position, Span};

impl<'resolver> Resolver<'resolver> {
    pub fn desugar(&mut self, element: Element<'resolver>) -> Element<'resolver> {
        match element.kind {
            ElementKind::Literal(literal) => {
                self.desugar_literal(literal, element.span)
            }
            ElementKind::Unary(unary) => {
                self.desugar_unary(unary, element.span)
            }
            ElementKind::Binary(binary) => {
                let left = Box::new(self.desugar(*binary.left));
                let right = Box::new(self.desugar(*binary.right));
                Element::new(
                    ElementKind::Binary(Binary::new(left, binary.operator, right)),
                    element.span
                )
            }
            ElementKind::Access(access) => {
                let target = Box::new(self.desugar(*access.target));
                let member = Box::new(self.desugar(*access.member));
                Element::new(
                    ElementKind::Access(Access::new(target, member)),
                    element.span
                )
            }
            ElementKind::Index(index) => {
                let target = Box::new(self.desugar(*index.target));
                let indexes = index.indexes.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Index(Index::new(target, indexes)),
                    element.span
                )
            }
            ElementKind::Invoke(invoke) => {
                let target = Box::new(self.desugar(*invoke.target));
                let arguments = invoke.arguments.into_iter().map(|a| self.desugar(a)).collect();
                Element::new(
                    ElementKind::Invoke(Invoke::new(target, arguments)),
                    element.span
                )
            }
            ElementKind::Construct(construct) => {
                let target = Box::new(self.desugar(*construct.target));
                let fields = construct.members.into_iter().map(|f| self.desugar(f)).collect();
                Element::new(
                    ElementKind::Construct(Structure::new(target, fields)),
                    element.span
                )
            }
            ElementKind::Group(group) => {
                let items = group.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Group(Group::new(items)),
                    element.span
                )
            }
            ElementKind::Sequence(sequence) => {
                let items = sequence.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Sequence(Sequence::new(items)),
                    element.span
                )
            }
            ElementKind::Collection(collection) => {
                let items = collection.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Collection(Collection::new(items)),
                    element.span
                )
            }
            ElementKind::Series(series) => {
                let items = series.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Series(Series::new(items)),
                    element.span
                )
            }
            ElementKind::Bundle(bundle) => {
                let items = bundle.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Bundle(Bundle::new(items)),
                    element.span
                )
            }
            ElementKind::Block(block) => {
                let items = block.items.into_iter().map(|i| self.desugar(i)).collect();
                Element::new(
                    ElementKind::Block(Block::new(items)),
                    element.span
                )
            }
            ElementKind::Label(label) => {
                let label_val = Box::new(self.desugar(*label.label));
                let element_val = Box::new(self.desugar(*label.element));
                Element::new(
                    ElementKind::Label(Label::new(label_val, element_val)),
                    element.span
                )
            }
            ElementKind::Assign(assign) => {
                let target = Box::new(self.desugar(*assign.target));
                let value = Box::new(self.desugar(*assign.value));
                Element::new(
                    ElementKind::Assign(Assign::new(target, value)),
                    element.span
                )
            }
            ElementKind::Conditional(conditional) => {
                let condition = Box::new(self.desugar(*conditional.condition));
                let then = Box::new(self.desugar(*conditional.then));
                let alternate = conditional.alternate.map(|a| Box::new(self.desugar(*a)));
                Element::new(
                    ElementKind::Conditional(Conditional::new(condition, then, alternate)),
                    element.span
                )
            }
            ElementKind::While(repeat) => {
                let condition = repeat.condition.map(|c| Box::new(self.desugar(*c)));
                let body = Box::new(self.desugar(*repeat.body));
                Element::new(
                    ElementKind::While(While::new(condition, body)),
                    element.span
                )
            }
            ElementKind::Cycle(cycle) => {
                let clause = Box::new(self.desugar(*cycle.clause));
                let body = Box::new(self.desugar(*cycle.body));
                Element::new(
                    ElementKind::Cycle(Cycle::new(clause, body)),
                    element.span
                )
            }
            ElementKind::Return(value) => {
                let value = value.map(|v| Box::new(self.desugar(*v)));
                Element::new(ElementKind::Return(value), element.span)
            }
            ElementKind::Break(value) => {
                let value = value.map(|v| Box::new(self.desugar(*v)));
                Element::new(ElementKind::Break(value), element.span)
            }
            ElementKind::Continue(value) => {
                let value = value.map(|v| Box::new(self.desugar(*v)));
                Element::new(ElementKind::Continue(value), element.span)
            }
            ElementKind::Symbolize(mut symbol) => {
                self.desugar_symbol(&mut symbol);
                Element::new(ElementKind::Symbolize(symbol), element.span)
            }
            ElementKind::Procedural(procedural) => {
                let body = Box::new(self.desugar(*procedural.body));
                Element::new(
                    ElementKind::Procedural(Procedural::new(body)),
                    element.span
                )
            }
            _ => element,
        }
    }

    fn desugar_literal(&self, literal: Token<'resolver>, span: Span<'resolver>) -> Element<'resolver> {
        let type_name = match literal.kind {
            TokenKind::Float(_) => "Float",
            TokenKind::Integer(_) => "Integer",
            TokenKind::Boolean(_) => "Boolean",
            TokenKind::String(_) => "String",
            TokenKind::Character(_) => "Character",
            _ => return Element::new(ElementKind::Literal(literal), span),
        };

        let target = Element::new(
            ElementKind::Literal(
                Token::new(
                    TokenKind::Identifier(Str::from(type_name)),
                    Span::void()
                )
            ),
            Span::void()
        );

        let member = Element::new(
            ElementKind::Invoke(
                Invoke::new(
                    Box::new(Element::new(
                        ElementKind::Literal(
                            Token::new(
                                TokenKind::Identifier(Str::from("new")),
                                Span::void()
                            )
                        ),
                        Span::void()
                    )),
                    vec![Element::new(ElementKind::Literal(literal), span)]
                )
            ),
            Span::void()
        );

        Element::new(
            ElementKind::Access(Access::new(Box::new(target), Box::new(member))),
            span
        )
    }

    fn desugar_unary(&mut self, unary: Unary<Token<'resolver>, Box<Element<'resolver>>>, span: Span<'resolver>) -> Element<'resolver> {
        let operand = Box::new(self.desugar(*unary.operand));

        if let TokenKind::Operator(operator) = &unary.operator.kind {
            if matches!(operator.as_slice(), [OperatorKind::Exclamation]) {
                let member = Element::new(
                    ElementKind::Invoke(
                        Invoke::new(
                            Box::new(Element::new(
                                ElementKind::Literal(
                                    Token::new(
                                        TokenKind::Identifier(Str::from("not")),
                                        Span::void()
                                    )
                                ),
                                Span::void()
                            )),
                            Vec::new()
                        )
                    ),
                    Span::void()
                );

                return Element::new(
                    ElementKind::Access(Access::new(operand, Box::new(member))),
                    span
                );
            }
        }

        Element::new(
            ElementKind::Unary(Unary::new(unary.operator, operand)),
            span
        )
    }

    fn desugar_symbol(&mut self, symbol: &mut Symbol<'resolver>) {
        match &mut symbol.kind {
            Symbolic::Inclusion(inclusion) => {
                *inclusion.target = self.desugar(*inclusion.target.clone());
            }
            Symbolic::Extension(extension) => {
                extension.target = Box::new(self.desugar(*extension.target.clone()));
                extension.members.iter_mut().for_each(|member| {
                    self.desugar_symbol(member); 
                });
            }
            Symbolic::Binding(binding) => {
                if let Some(value) = &binding.value {
                    binding.value = Some(Box::new(self.desugar(*value.clone())));
                }
            }
            Symbolic::Structure(structure) => {
                structure.members.iter_mut().for_each(|member| {
                    self.desugar_symbol(member);  
                });
            }
            Symbolic::Enumeration(enumeration) => {
                enumeration.members.iter_mut().for_each(|member| {
                    self.desugar_symbol(member);
                });
            }
            Symbolic::Method(method) => {
                method.members.iter_mut().for_each(|member| {
                    self.desugar_symbol(member);
                });

                method.body = Box::new(self.desugar(*method.body.clone()));
                
                if let Some(output) = &method.output {
                    method.output = Some(Box::new(self.desugar(*output.clone())));
                }
            }
            _ => {}
        }
    }
}