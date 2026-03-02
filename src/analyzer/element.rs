use crate::{
    data::Str,
    parser::{Element, ElementKind},
    resolver::Resolver,
    scanner::{PunctuationKind, Token, TokenKind},
};
use crate::analyzer::{Analysis, Analyzable, AnalyzeError, ErrorKind, Instruction};
use crate::data::schema::*;
use crate::format::Show;

mod operation;

#[derive(Clone, Copy)]
pub(crate) struct Analyzer {
    pub method: bool,
    pub cycle: bool,
}

impl Analyzer {
    pub fn root() -> Self {
        Self {
            method: false,
            cycle: false,
        }
    }

    pub fn method(self) -> Self {
        Self {
            method: true,
            ..self
        }
    }

    pub fn cycle(self) -> Self {
        Self {
            cycle: true,
            ..self
        }
    }
}

fn primitive<'element>(target: &Element<'element>) -> Option<&'element str> {
    let token = target.brand()?;
    match token.kind {
        TokenKind::Identifier(identifier) => identifier.as_str(),
        _ => None,
    }
}

fn arity<'element>(
    name: &str,
    members: usize,
    expected: &str,
    valid: bool,
    span: crate::tracker::Span<'element>,
) -> Result<(), AnalyzeError<'element>> {
    if valid {
        Ok(())
    } else {
        Err(AnalyzeError::new(
            ErrorKind::InvalidPrimitiveArity {
                name: name.to_string(),
                expected: expected.to_string(),
                found: members,
            },
            span,
        ))
    }
}

fn invoke<'element>(
    call: &Invoke<Box<Element<'element>>, Element<'element>>,
    resolver: &Resolver<'element>,
    context: Analyzer,
    span: crate::tracker::Span<'element>,
) -> Result<Analysis<'element>, AnalyzeError<'element>> {
    let name = primitive(&call.target);

    match name {
        Some("if") => {
            arity(
                "if",
                call.members.len(),
                "3 arguments",
                call.members.len() == 3,
                span,
            )?;
            let condition = analyze(&call.members[0], resolver, context)?;
            let then = analyze(&call.members[1], resolver, context)?;
            let otherwise = analyze(&call.members[2], resolver, context)?;
            Ok(Analysis::new(Instruction::Conditional(
                Box::new(condition),
                Box::new(then),
                Box::new(otherwise),
            )))
        }
        Some("while") => {
            arity(
                "while",
                call.members.len(),
                "2 arguments",
                call.members.len() == 2,
                span,
            )?;
            let loop_context = context.cycle();
            let condition = analyze(&call.members[0], resolver, loop_context)?;
            let body = analyze(&call.members[1], resolver, loop_context)?;
            Ok(Analysis::new(Instruction::While(
                Box::new(condition),
                Box::new(body),
            )))
        }
        Some("for") => {
            arity(
                "for",
                call.members.len(),
                "4 arguments",
                call.members.len() == 4,
                span,
            )?;
            let init = analyze(&call.members[0], resolver, context)?;
            let loop_context = context.cycle();
            let condition = analyze(&call.members[1], resolver, loop_context)?;
            let step = analyze(&call.members[2], resolver, loop_context)?;
            let body = analyze(&call.members[3], resolver, loop_context)?;
            let while_body = Analysis::new(Instruction::Block(vec![body, step]));
            Ok(Analysis::new(Instruction::Block(vec![
                init,
                Analysis::new(Instruction::While(
                    Box::new(condition),
                    Box::new(while_body),
                )),
            ])))
        }
        Some("break") => {
            arity(
                "break",
                call.members.len(),
                "0 arguments",
                call.members.is_empty(),
                span,
            )?;
            if !context.cycle {
                return Err(AnalyzeError::new(
                    ErrorKind::InvalidPrimitiveContext {
                        name: "break".to_string(),
                        expected: "inside loop body".to_string(),
                    },
                    span,
                ));
            }
            Ok(Analysis::new(Instruction::Break(None)))
        }
        Some("continue") => {
            arity(
                "continue",
                call.members.len(),
                "0 arguments",
                call.members.is_empty(),
                span,
            )?;
            if !context.cycle {
                return Err(AnalyzeError::new(
                    ErrorKind::InvalidPrimitiveContext {
                        name: "continue".to_string(),
                        expected: "inside loop body".to_string(),
                    },
                    span,
                ));
            }
            Ok(Analysis::new(Instruction::Continue(None)))
        }
        Some("return") => {
            arity(
                "return",
                call.members.len(),
                "0 or 1 arguments",
                call.members.len() <= 1,
                span,
            )?;
            if !context.method {
                return Err(AnalyzeError::new(
                    ErrorKind::InvalidPrimitiveContext {
                        name: "return".to_string(),
                        expected: "inside function body".to_string(),
                    },
                    span,
                ));
            }
            let value = if call.members.is_empty() {
                None
            } else {
                Some(Box::new(analyze(&call.members[0], resolver, context)?))
            };
            Ok(Analysis::new(Instruction::Return(value)))
        }
        _ => {
            let target = analyze(&call.target, resolver, context)?;
            let arguments: Result<Vec<Box<Analysis<'element>>>, AnalyzeError<'element>> = call
                .members
                .iter()
                .map(|member| analyze(member, resolver, context).map(Box::new))
                .collect();
            Ok(Analysis::new(Instruction::Invoke(Invoke::new(
                Box::new(target),
                arguments?,
            ))))
        }
    }
}

pub(crate) fn analyze<'element>(
    element: &Element<'element>,
    resolver: &Resolver<'element>,
    context: Analyzer,
) -> Result<Analysis<'element>, AnalyzeError<'element>> {
    match &element.kind {
        ElementKind::Literal(literal) => literal.analyze(resolver),

        ElementKind::Delimited(delimited) => {
            match (
                &delimited.start.kind,
                delimited.separator.as_ref().map(|token| &token.kind),
                &delimited.end.kind,
            ) {
                (
                    TokenKind::Punctuation(PunctuationKind::LeftBrace),
                    None,
                    TokenKind::Punctuation(PunctuationKind::RightBrace),
                )
                | (
                    TokenKind::Punctuation(PunctuationKind::LeftBrace),
                    Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                    TokenKind::Punctuation(PunctuationKind::RightBrace),
                ) => {
                    let items: Result<Vec<Analysis<'element>>, AnalyzeError<'element>> = delimited
                        .members
                        .iter()
                        .map(|item| analyze(item, resolver, context))
                        .collect();

                    Ok(Analysis::new(Instruction::Block(items?)))
                }
                (
                    TokenKind::Punctuation(PunctuationKind::LeftBracket),
                    _,
                    TokenKind::Punctuation(PunctuationKind::RightBracket),
                ) => {
                    let items: Result<Vec<Box<Analysis<'element>>>, AnalyzeError<'element>> =
                        delimited
                            .members
                            .iter()
                            .map(|item| analyze(item, resolver, context).map(Box::new))
                            .collect();

                    Ok(Analysis::new(Instruction::Array(items?)))
                }
                (
                    TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                    None,
                    TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                ) => {
                    if delimited.members.len() == 1 {
                        analyze(&delimited.members[0], resolver, context)
                    } else {
                        let items: Result<Vec<Box<Analysis<'element>>>, AnalyzeError<'element>> =
                            delimited
                                .members
                                .iter()
                                .map(|item| analyze(item, resolver, context).map(Box::new))
                                .collect();
                        Ok(Analysis::new(Instruction::Tuple(items?)))
                    }
                }
                (
                    TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                    Some(_),
                    TokenKind::Punctuation(PunctuationKind::RightParenthesis),
                ) => {
                    let items: Result<Vec<Box<Analysis<'element>>>, AnalyzeError<'element>> =
                        delimited
                            .members
                            .iter()
                            .map(|item| analyze(item, resolver, context).map(Box::new))
                            .collect();

                    Ok(Analysis::new(Instruction::Tuple(items?)))
                }

                _ => Err(AnalyzeError::new(ErrorKind::Unimplemented, element.span)),
            }
        }

        ElementKind::Unary(unary) => analyze_unary(unary, resolver, context),

        ElementKind::Binary(item) => binary(item, resolver, context),

        ElementKind::Index(index) => {
            let target = analyze(&index.target, resolver, context)?;
            let indexes: Result<Vec<Box<Analysis<'element>>>, AnalyzeError<'element>> = index
                .members
                .iter()
                .map(|member| analyze(member, resolver, context).map(Box::new))
                .collect();
            Ok(Analysis::new(Instruction::Index(Index::new(
                Box::new(target),
                indexes?,
            ))))
        }

        ElementKind::Invoke(call) => invoke(call, resolver, context, element.span),

        ElementKind::Construct(constructor) => {
            let target = constructor
                .target
                .brand()
                .map(|s| s.format(1))
                .unwrap_or_default();

            let members: Vec<Box<Analysis<'element>>> = constructor
                .members
                .iter()
                .map(|member| analyze(member, resolver, context).map(Box::new))
                .collect::<Result<Vec<Box<Analysis<'element>>>, AnalyzeError<'element>>>()?;

            match target.as_str().unwrap() {
                "Integer" => {
                    let mut value_opt = None;
                    let mut size_opt = None;
                    let mut signed_opt = None;

                    for member in &members {
                        if let Instruction::Assign(field_name, field_value) = &member.instruction {
                            match field_name.as_str().unwrap() {
                                "value" => {
                                    if let Instruction::Integer { value, .. } =
                                        &field_value.instruction
                                    {
                                        value_opt = Some(value.clone());
                                    }
                                }
                                "size" => {
                                    if let Instruction::Integer { value: size, .. } =
                                        &field_value.instruction
                                    {
                                        size_opt = Some(*size);
                                    }
                                }
                                "signed" => {
                                    if let Instruction::Boolean { value: signed } =
                                        &field_value.instruction
                                    {
                                        signed_opt = Some(*signed);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    match (value_opt, size_opt, signed_opt) {
                        (Some(value), Some(size), Some(signed)) => {
                            Ok(Analysis::new(Instruction::Integer {
                                value,
                                size: size.try_into().unwrap(),
                                signed,
                            }))
                        }
                        _ => Err(AnalyzeError::new(
                            ErrorKind::InvalidType,
                            constructor.target.span,
                        )),
                    }
                }
                "Float" => {
                    let mut value_opt = None;
                    let mut size_opt = None;

                    for member in &members {
                        if let Instruction::Assign(field_name, field_value) = &member.instruction {
                            match field_name.as_str().unwrap() {
                                "value" => {
                                    if let Instruction::Float { value, .. } =
                                        &field_value.instruction
                                    {
                                        value_opt = Some(value.clone());
                                    }
                                }
                                "size" => {
                                    if let Instruction::Integer { value: size, .. } =
                                        &field_value.instruction
                                    {
                                        size_opt = Some(*size);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    match (value_opt, size_opt) {
                        (Some(value), Some(size)) => Ok(Analysis::new(Instruction::Float {
                            value,
                            size: size.try_into().unwrap(),
                        })),
                        _ => Err(AnalyzeError::new(
                            ErrorKind::InvalidType,
                            constructor.target.span,
                        )),
                    }
                }
                "Boolean" => {
                    let mut value_opt = None;

                    for member in &members {
                        if let Instruction::Assign(field_name, field_value) = &member.instruction {
                            if field_name.as_str().unwrap() == "value" {
                                if let Instruction::Boolean { value } = &field_value.instruction {
                                    value_opt = Some(*value);
                                }
                            }
                        }
                    }

                    match value_opt {
                        Some(value) => Ok(Analysis::new(Instruction::Boolean { value })),
                        _ => Err(AnalyzeError::new(
                            ErrorKind::InvalidType,
                            constructor.target.span,
                        )),
                    }
                }
                _ => {
                    let analyzed = Structure::new(Str::from(target), members);
                    Ok(Analysis::new(Instruction::Constructor(analyzed)))
                }
            }
        }

        ElementKind::Symbolize(symbol) => super::analyzer::symbol(symbol, resolver, context),
    }
}

impl<'element> Analyzable<'element> for Element<'element> {
    fn analyze(
        &self,
        resolver: &Resolver<'element>,
    ) -> Result<Analysis<'element>, AnalyzeError<'element>> {
        analyze(self, resolver, Analyzer::root())
    }
}

fn binary<'binary>(
    node: &Binary<Box<Element<'binary>>, Token<'binary>, Box<Element<'binary>>>,
    resolver: &Resolver<'binary>,
    context: Analyzer,
) -> Result<Analysis<'binary>, AnalyzeError<'binary>> {
    operation::binary(node, resolver, context)
}

fn analyze_unary<'unary>(
    node: &Unary<Token<'unary>, Box<Element<'unary>>>,
    resolver: &Resolver<'unary>,
    context: Analyzer,
) -> Result<Analysis<'unary>, AnalyzeError<'unary>> {
    operation::analyze_unary(node, resolver, context)
}

impl<'binary> Analyzable<'binary>
    for Binary<Box<Element<'binary>>, Token<'binary>, Box<Element<'binary>>>
{
    fn analyze(
        &self,
        resolver: &Resolver<'binary>,
    ) -> Result<Analysis<'binary>, AnalyzeError<'binary>> {
        binary(self, resolver, Analyzer::root())
    }
}
