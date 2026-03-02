use crate::{
    data::{Boolean, Scale, Str},
    parser::{Element, ElementKind},
    scanner::{OperatorKind, PunctuationKind, Token, TokenKind},
    tracker::Span,
};
use crate::checker::{unify, CheckError, Checkable, ErrorKind, Type, TypeKind};
use crate::data::*;
use crate::format::Show;

fn mismatch<'element>(
    left: Type<'element>,
    right: Type<'element>,
    span: Span<'element>,
) -> CheckError<'element> {
    CheckError::new(ErrorKind::Mismatch(left, right), span)
}

fn invalid(operator: Token) -> CheckError {
    let span = operator.span;
    CheckError::new(ErrorKind::InvalidOperation(operator), span)
}

fn primitive<'element>(name: &str, span: Span<'element>) -> Option<Type<'element>> {
    match TypeKind::from_name(name) {
        Some(TypeKind::Integer { bits, signed }) => Some(Type::integer(bits, signed, span)),
        Some(TypeKind::Float { bits }) => Some(Type::float(bits, span)),
        Some(TypeKind::Boolean) => Some(Type::boolean(span)),
        Some(TypeKind::Char) => Some(Type::character(span)),
        Some(_) | None => match name {
            "String" => Some(Type::string(span)),
            "Type" => Some(Type::new(
                TypeKind::Type(Box::new(Type::new(TypeKind::Infer, span))),
                span,
            )),
            _ => None,
        },
    }
}

fn integer(members: &[Element]) -> (Option<Scale>, Option<Boolean>) {
    let mut size = None;
    let mut signed = None;

    for member in members {
        if let ElementKind::Binary(binary) = &member.kind {
            if let TokenKind::Operator(operator) = &binary.operator.kind {
                if operator.as_slice() != [OperatorKind::Equal] {
                    continue;
                }
            } else {
                continue;
            }

            let field = binary.left.brand().and_then(|token| match token.kind {
                TokenKind::Identifier(name) => Some(name),
                _ => None,
            });

            match field.as_ref().and_then(|name| name.as_str()) {
                Some("size") => {
                    if let ElementKind::Literal(Token {
                        kind: TokenKind::Integer(value),
                        ..
                    }) = binary.right.kind
                    {
                        size = value.try_into().ok();
                    }
                }
                Some("signed") => {
                    if let ElementKind::Literal(Token {
                        kind: TokenKind::Boolean(value),
                        ..
                    }) = binary.right.kind
                    {
                        signed = Some(value);
                    }
                }
                _ => {}
            }
        }
    }

    (size, signed)
}

fn float(members: &[Element]) -> Option<Scale> {
    for member in members {
        if let ElementKind::Binary(binary) = &member.kind {
            if let TokenKind::Operator(operator) = &binary.operator.kind {
                if operator.as_slice() != [OperatorKind::Equal] {
                    continue;
                }
            } else {
                continue;
            }

            let field = binary.left.brand().and_then(|token| match token.kind {
                TokenKind::Identifier(name) => Some(name),
                _ => None,
            });

            if matches!(field.as_ref().and_then(|name| name.as_str()), Some("size")) {
                if let ElementKind::Literal(Token {
                    kind: TokenKind::Integer(value),
                    ..
                }) = binary.right.kind
                {
                    return value.try_into().ok();
                }
            }
        }
    }

    None
}

fn numeric<'element>(
    left: Type<'element>,
    right: Type<'element>,
    span: Span<'element>,
) -> Result<Type<'element>, CheckError<'element>> {
    if left.is_infer() && right.is_numeric() {
        return Ok(right);
    }
    if right.is_infer() && left.is_numeric() {
        return Ok(left);
    }
    if left.is_infer() && right.is_infer() {
        return Ok(Type::new(TypeKind::Infer, span));
    }
    match (&left.kind, &right.kind) {
        (
            TypeKind::Integer {
                bits: left_bits,
                signed: left_signed,
            },
            TypeKind::Integer {
                bits: right_bits,
                signed: right_signed,
            },
        ) => Ok(Type::integer(
            (*left_bits).max(*right_bits),
            *left_signed || *right_signed,
            span,
        )),
        (TypeKind::Float { bits: left_bits }, TypeKind::Float { bits: right_bits }) => {
            Ok(Type::float((*left_bits).max(*right_bits), span))
        }
        (TypeKind::Float { bits }, TypeKind::Integer { .. })
        | (TypeKind::Integer { .. }, TypeKind::Float { bits }) => Ok(Type::float(*bits, span)),
        _ => Err(mismatch(left, right, span)),
    }
}

fn compatible<'element>(left: &Type<'element>, right: &Type<'element>) -> bool {
    unify(left, right).is_some()
}

fn target<'element>(element: &Element<'element>) -> Option<Str<'element>> {
    element.brand().and_then(|token| match token.kind {
        TokenKind::Identifier(name) => Some(name),
        _ => None,
    })
}

fn addressable(element: &Element) -> bool {
    match &element.kind {
        ElementKind::Literal(Token {
            kind: TokenKind::Identifier(_),
            ..
        }) => true,
        ElementKind::Index(_) => true,
        ElementKind::Binary(binary) => {
            matches!(binary.operator.kind, TokenKind::Operator(OperatorKind::Dot))
        }
        ElementKind::Unary(unary) => {
            matches!(unary.operator.kind, TokenKind::Operator(OperatorKind::Star))
        }
        _ => false,
    }
}

impl<'element> Checkable<'element> for Element<'element> {
    fn infer(&self) -> Result<Type<'element>, CheckError<'element>> {
        match self.kind.clone() {
            ElementKind::Literal(literal) => match literal.kind {
                TokenKind::Integer(_) => Ok(Type::integer(64, true, literal.span)),
                TokenKind::Float(_) => Ok(Type::float(64, literal.span)),
                TokenKind::Boolean(_) => Ok(Type::boolean(literal.span)),
                TokenKind::String(_) => Ok(Type::string(literal.span)),
                TokenKind::Character(_) => Ok(Type::character(literal.span)),
                _ => Ok(Type::new(TypeKind::Infer, literal.span)),
            },
            ElementKind::Delimited(delimited) => delimited.infer(),
            ElementKind::Unary(unary) => {
                let operand = unary.operand.infer()?;
                let operator = match unary.operator.kind.clone() {
                    TokenKind::Operator(operator) => operator,
                    _ => return Err(invalid(unary.operator)),
                };

                match operator.as_slice() {
                    [OperatorKind::Exclamation] => {
                        if operand.is_boolean() {
                            Ok(Type::boolean(self.span))
                        } else {
                            Err(mismatch(Type::boolean(self.span), operand, self.span))
                        }
                    }
                    [OperatorKind::Tilde] => {
                        if operand.is_integer() {
                            Ok(operand)
                        } else {
                            Err(mismatch(
                                Type::integer(64, true, self.span),
                                operand,
                                self.span,
                            ))
                        }
                    }
                    [OperatorKind::Plus] | [OperatorKind::Minus] => {
                        if operand.is_numeric() {
                            Ok(operand)
                        } else {
                            Err(mismatch(
                                Type::integer(64, true, self.span),
                                operand,
                                self.span,
                            ))
                        }
                    }
                    [OperatorKind::Ampersand] => {
                        if addressable(&unary.operand) {
                            Ok(Type::pointer(operand, self.span))
                        } else {
                            Err(invalid(unary.operator))
                        }
                    }
                    [OperatorKind::Star] => match operand.kind {
                        TypeKind::Pointer { to } => Ok(*to),
                        TypeKind::Infer => Ok(Type::new(TypeKind::Infer, self.span)),
                        _ => Err(mismatch(
                            Type::pointer(Type::new(TypeKind::Infer, self.span), self.span),
                            operand,
                            self.span,
                        )),
                    },
                    _ => Err(invalid(unary.operator)),
                }
            }
            ElementKind::Binary(binary) => {
                let mut left = binary.left.infer()?;
                let mut right = binary.right.infer()?;
                let operator = match binary.operator.kind.clone() {
                    TokenKind::Operator(operator) => operator,
                    _ => return Err(invalid(binary.operator)),
                };

                match operator.as_slice() {
                    [OperatorKind::Equal] => {
                        if compatible(&left, &right) {
                            Ok(left)
                        } else {
                            Err(mismatch(left, right, binary.operator.span))
                        }
                    }
                    [OperatorKind::Plus] => {
                        if left.is_pointer() {
                            if right.is_integer() {
                                left.span = binary.operator.span;
                                return Ok(left);
                            }
                            return Err(invalid(binary.operator.clone()));
                        }

                        if right.is_pointer() {
                            if left.is_integer() {
                                right.span = binary.operator.span;
                                return Ok(right);
                            }
                            return Err(invalid(binary.operator.clone()));
                        }

                        numeric(left, right, binary.operator.span)
                    }
                    [OperatorKind::Minus] => {
                        if left.is_pointer() && right.is_pointer() {
                            if left == right {
                                return Ok(Type::integer(64, true, binary.operator.span));
                            }
                            return Err(mismatch(left, right, binary.operator.span));
                        }

                        if left.is_pointer() {
                            if right.is_integer() {
                                left.span = binary.operator.span;
                                return Ok(left);
                            }
                            return Err(invalid(binary.operator.clone()));
                        }

                        if right.is_pointer() {
                            return Err(invalid(binary.operator.clone()));
                        }

                        numeric(left, right, binary.operator.span)
                    }
                    [OperatorKind::Star] | [OperatorKind::Slash] | [OperatorKind::Percent] => {
                        numeric(left, right, binary.operator.span)
                    }
                    [OperatorKind::Ampersand]
                    | [OperatorKind::Pipe]
                    | [OperatorKind::Caret]
                    | [OperatorKind::LeftAngle, OperatorKind::LeftAngle]
                    | [OperatorKind::RightAngle, OperatorKind::RightAngle] => {
                        if left.is_integer() && right.is_integer() {
                            Ok(left)
                        } else {
                            Err(mismatch(
                                Type::integer(64, true, self.span),
                                right,
                                self.span,
                            ))
                        }
                    }
                    [OperatorKind::Ampersand, OperatorKind::Ampersand]
                    | [OperatorKind::Pipe, OperatorKind::Pipe] => {
                        if left.is_boolean() && right.is_boolean() {
                            Ok(Type::boolean(binary.operator.span))
                        } else {
                            Err(mismatch(Type::boolean(self.span), right, self.span))
                        }
                    }
                    [OperatorKind::Equal, OperatorKind::Equal]
                    | [OperatorKind::Exclamation, OperatorKind::Equal]
                    | [OperatorKind::LeftAngle]
                    | [OperatorKind::LeftAngle, OperatorKind::Equal]
                    | [OperatorKind::RightAngle]
                    | [OperatorKind::RightAngle, OperatorKind::Equal] => {
                        if compatible(&left, &right) {
                            Ok(Type::boolean(binary.operator.span))
                        } else {
                            Err(mismatch(left, right, binary.operator.span))
                        }
                    }
                    [OperatorKind::Dot] => Ok(right),
                    _ => Err(invalid(binary.operator)),
                }
            }
            ElementKind::Index(index) => {
                if index.members.is_empty() {
                    return Ok(Type::unit(self.span));
                }

                let target = index.target.infer()?;
                let index_type = index.members[0].infer()?;
                if !index_type.is_integer() {
                    return Err(mismatch(
                        Type::integer(64, true, self.span),
                        index_type,
                        self.span,
                    ));
                }

                match target.kind {
                    TypeKind::Array { member, .. } => Ok(*member),
                    TypeKind::Tuple { members } => {
                        if let ElementKind::Literal(Token {
                            kind: TokenKind::Integer(value),
                            ..
                        }) = &index.members[0].kind
                        {
                            if let Ok(idx) = usize::try_from(*value) {
                                if idx < members.len() {
                                    return Ok(members[idx].clone());
                                }
                            }
                        }

                        let token = Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                            self.span,
                        );
                        Err(invalid(token))
                    }
                    _ => {
                        let token = Token::new(
                            TokenKind::Punctuation(PunctuationKind::LeftBracket),
                            self.span,
                        );
                        Err(invalid(token))
                    }
                }
            }
            ElementKind::Invoke(invoke) => {
                let primitive = target(&invoke.target).and_then(|name| name.as_str());

                match primitive {
                    Some("if") => {
                        if invoke.members.len() != 3 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("if")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }

                        let condition = invoke.members[0].infer()?;
                        if !condition.is_boolean() {
                            return Err(mismatch(
                                Type::boolean(self.span),
                                condition,
                                invoke.members[0].span,
                            ));
                        }

                        let then = invoke.members[1].infer()?;
                        let otherwise = invoke.members[2].infer()?;

                        if let Some(unified) = unify(&then, &otherwise) {
                            Ok(unified)
                        } else {
                            Err(mismatch(then, otherwise, self.span))
                        }
                    }
                    Some("while") => {
                        if invoke.members.len() != 2 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("while")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }

                        let condition = invoke.members[0].infer()?;
                        if !condition.is_boolean() {
                            return Err(mismatch(
                                Type::boolean(self.span),
                                condition,
                                invoke.members[0].span,
                            ));
                        }

                        invoke.members[1].infer()?;

                        Ok(Type::unit(self.span))
                    }
                    Some("for") => {
                        if invoke.members.len() != 4 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("for")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }

                        invoke.members[0].infer()?;
                        let condition = invoke.members[1].infer()?;
                        if !condition.is_boolean() {
                            return Err(mismatch(
                                Type::boolean(self.span),
                                condition,
                                invoke.members[1].span,
                            ));
                        }
                        invoke.members[2].infer()?;
                        invoke.members[3].infer()?;

                        Ok(Type::unit(self.span))
                    }
                    Some("is_some" | "is_none") => {
                        if invoke.members.len() != 1 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("is_some")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }

                        invoke.members[0].infer()?;
                        Ok(Type::boolean(self.span))
                    }
                    Some("unwrap") => {
                        if invoke.members.len() != 1 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("unwrap")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }

                        invoke.members[0].infer()?;
                        Ok(Type::new(TypeKind::Infer, self.span))
                    }
                    Some("or_else") => {
                        if invoke.members.len() != 2 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("or_else")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }

                        invoke.members[0].infer()?;
                        let fallback = invoke.members[1].infer()?;
                        Ok(fallback)
                    }
                    Some("Some" | "None") => {
                        if invoke.members.len() != 2 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("Some")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }

                        invoke.members[0].infer()?;
                        invoke.members[1].infer()?;
                        Ok(Type::new(TypeKind::Infer, self.span))
                    }
                    Some("print" | "eprint") => {
                        if invoke.members.len() != 1 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("print")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }

                        let value = invoke.members[0].infer()?;
                        if !matches!(value.kind, TypeKind::String)
                            && !value.is_boolean()
                            && !value.is_numeric()
                            && !value.is_infer()
                        {
                            return Err(mismatch(
                                Type::string(self.span),
                                value,
                                invoke.members[0].span,
                            ));
                        }

                        Ok(Type::unit(self.span))
                    }
                    Some("print_raw" | "eprint_raw") => {
                        if invoke.members.len() != 1 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("print")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }

                        let value = invoke.members[0].infer()?;
                        if !matches!(value.kind, TypeKind::String) {
                            return Err(mismatch(
                                Type::string(self.span),
                                value,
                                invoke.members[0].span,
                            ));
                        }

                        Ok(Type::unit(self.span))
                    }
                    Some("read_line") => {
                        if !invoke.members.is_empty() {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("read_line")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }
                        Ok(Type::string(self.span))
                    }
                    Some("len") => {
                        if invoke.members.len() != 1 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("len")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }
                        let value = invoke.members[0].infer()?;
                        if !matches!(value.kind, TypeKind::String) {
                            return Err(mismatch(
                                Type::string(self.span),
                                value,
                                invoke.members[0].span,
                            ));
                        }
                        Ok(Type::integer(64, true, self.span))
                    }
                    Some("write") => {
                        if invoke.members.len() != 2 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("write")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }
                        let fd = invoke.members[0].infer()?;
                        if !fd.is_integer() {
                            return Err(mismatch(
                                Type::integer(64, true, self.span),
                                fd,
                                invoke.members[0].span,
                            ));
                        }
                        let value = invoke.members[1].infer()?;
                        if !matches!(value.kind, TypeKind::String) {
                            return Err(mismatch(
                                Type::string(self.span),
                                value,
                                invoke.members[1].span,
                            ));
                        }
                        Ok(Type::integer(64, true, self.span))
                    }
                    Some("alloc") => {
                        if invoke.members.len() != 1 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("alloc")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }
                        let size = invoke.members[0].infer()?;
                        if !size.is_integer() {
                            return Err(mismatch(
                                Type::integer(64, true, self.span),
                                size,
                                invoke.members[0].span,
                            ));
                        }
                        Ok(Type::pointer(Type::new(TypeKind::Infer, self.span), self.span))
                    }
                    Some("free") => {
                        if invoke.members.len() != 2 {
                            let token = invoke.target.brand().unwrap_or(Token::new(
                                TokenKind::Identifier(Str::from("free")),
                                self.span,
                            ));
                            return Err(invalid(token));
                        }
                        let ptr = invoke.members[0].infer()?;
                        if !ptr.is_pointer() {
                            return Err(mismatch(
                                Type::pointer(Type::new(TypeKind::Infer, self.span), self.span),
                                ptr,
                                invoke.members[0].span,
                            ));
                        }
                        let size = invoke.members[1].infer()?;
                        if !size.is_integer() {
                            return Err(mismatch(
                                Type::integer(64, true, self.span),
                                size,
                                invoke.members[1].span,
                            ));
                        }
                        Ok(Type::unit(self.span))
                    }
                    _ => Ok(Type::new(TypeKind::Infer, self.span)),
                }
            }
            ElementKind::Construct(construct) => {
                if let Some(target) = construct.target.brand() {
                    if let TokenKind::Identifier(name) = target.kind {
                        if let Some(name) = name.as_str() {
                            match name {
                                "Integer" | "Int32" | "Int64" => {
                                    let (size, signed) = integer(&construct.members);
                                    return Ok(Type::integer(
                                        if name == "Int32" {
                                            32
                                        } else {
                                            size.unwrap_or(64)
                                        },
                                        signed.unwrap_or(true),
                                        self.span,
                                    ));
                                }
                                "Float" => {
                                    let size = float(&construct.members).unwrap_or(64);
                                    return Ok(Type::float(size, self.span));
                                }
                                _ => {
                                    if let Some(kind) = primitive(name, self.span) {
                                        return Ok(kind);
                                    }
                                }
                            }
                        }
                    }
                }

                let members: Result<Vec<Box<Type<'element>>>, CheckError<'element>> = construct
                    .members
                    .iter()
                    .map(|field| field.infer().map(Box::new))
                    .collect();

                let structure = Structure::new(
                    Str::from(construct.target.brand().unwrap().format(0)),
                    members?,
                );

                Ok(Type::new(TypeKind::Structure(structure), self.span))
            }
            ElementKind::Symbolize(_) => Ok(Type::unit(self.span)),
        }
    }
}

impl<'delimited> Checkable<'delimited> for Delimited<Token<'delimited>, Element<'delimited>> {
    fn infer(&self) -> Result<Type<'delimited>, CheckError<'delimited>> {
        match (
            &self.start.kind,
            self.separator.as_ref().map(|token| &token.kind),
            &self.end.kind,
        ) {
            (
                TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                None,
                TokenKind::Punctuation(PunctuationKind::RightParenthesis),
            )
            | (
                TokenKind::Punctuation(PunctuationKind::LeftParenthesis),
                Some(TokenKind::Punctuation(PunctuationKind::Comma)),
                TokenKind::Punctuation(PunctuationKind::RightParenthesis),
            ) => {
                if self.separator.is_none() && self.members.len() == 1 {
                    return self.members[0].infer();
                }

                let members: Result<Vec<Type<'delimited>>, CheckError<'delimited>> =
                    self.members.iter().map(|field| field.infer()).collect();

                Ok(Type::new(
                    TypeKind::Tuple { members: members? },
                    Span::void(),
                ))
            }

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
                if let Some(item) = self.members.last() {
                    item.infer()
                } else {
                    Ok(Type::unit(Span::void()))
                }
            }

            (
                TokenKind::Punctuation(PunctuationKind::LeftBracket),
                _,
                TokenKind::Punctuation(PunctuationKind::RightBracket),
            ) => {
                if self.members.is_empty() {
                    return Ok(Type::new(
                        TypeKind::Array {
                            member: Box::new(Type::new(TypeKind::Infer, Span::void())),
                            size: 0,
                        },
                        Span::void(),
                    ));
                }

                let mut member_type = self.members[0].infer()?;
                for member in self.members.iter().skip(1) {
                    let current = member.infer()?;
                    if member_type == current {
                        continue;
                    }
                    if member_type.is_numeric() && current.is_numeric() {
                        member_type = numeric(member_type, current, member.span)?;
                        continue;
                    }
                    if member_type.is_infer() {
                        member_type = current;
                        continue;
                    }
                    if current.is_infer() {
                        continue;
                    }
                    return Err(mismatch(member_type, current, member.span));
                }

                Ok(Type::new(
                    TypeKind::Array {
                        member: Box::new(member_type),
                        size: self.members.len() as Scale,
                    },
                    Span::void(),
                ))
            }

            _ => Ok(Type::unit(Span::void())),
        }
    }
}
