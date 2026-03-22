use crate::analyzer::{Analysis, AnalysisKind};
use crate::data::{Aggregate, Binding, Boolean, Char, Float, Function, Index, Integer, Invoke, Scale, Str};
use crate::internal::cache::{Encode, Decode};
use crate::resolver::Type;
use crate::tracker::Span;

impl<'analysis> Encode for Analysis<'analysis> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.kind.encode(buffer);
        self.span.encode(buffer);
        self.typing.encode(buffer);
    }
}

impl<'analysis> Decode<'analysis> for Analysis<'analysis> {
    fn decode(buffer: &'analysis [u8], cursor: &mut usize) -> Self {
        Analysis {
            kind: AnalysisKind::decode(buffer, cursor),
            span: Span::decode(buffer, cursor),
            typing: Type::decode(buffer, cursor),
        }
    }
}

impl<'analysis> Encode for AnalysisKind<'analysis> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            AnalysisKind::Integer { value, size, signed } => {
                buffer.push(0);
                value.encode(buffer);
                size.encode(buffer);
                signed.encode(buffer);
            }
            AnalysisKind::Float { value, size } => {
                buffer.push(1);
                value.0.encode(buffer);
                size.encode(buffer);
            }
            AnalysisKind::Boolean { value } => {
                buffer.push(2);
                value.encode(buffer);
            }
            AnalysisKind::String { value } => {
                buffer.push(3);
                value.encode(buffer);
            }
            AnalysisKind::Character { value } => {
                buffer.push(4);
                value.encode(buffer);
            }
            AnalysisKind::Array(items) => {
                buffer.push(5);
                items.encode(buffer);
            }
            AnalysisKind::Tuple(items) => {
                buffer.push(6);
                items.encode(buffer);
            }
            AnalysisKind::Negate(expr) => {
                buffer.push(7);
                expr.encode(buffer);
            }
            AnalysisKind::SizeOf(ty) => {
                buffer.push(8);
                ty.encode(buffer);
            }
            AnalysisKind::Add(left, right) => {
                buffer.push(9);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::Subtract(left, right) => {
                buffer.push(10);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::Multiply(left, right) => {
                buffer.push(11);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::Divide(left, right) => {
                buffer.push(12);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::Modulus(left, right) => {
                buffer.push(13);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::LogicalAnd(left, right) => {
                buffer.push(14);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::LogicalOr(left, right) => {
                buffer.push(15);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::LogicalNot(expr) => {
                buffer.push(16);
                expr.encode(buffer);
            }
            AnalysisKind::LogicalXOr(left, right) => {
                buffer.push(17);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::BitwiseAnd(left, right) => {
                buffer.push(18);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::BitwiseOr(left, right) => {
                buffer.push(19);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::BitwiseNot(expr) => {
                buffer.push(20);
                expr.encode(buffer);
            }
            AnalysisKind::BitwiseXOr(left, right) => {
                buffer.push(21);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::ShiftLeft(left, right) => {
                buffer.push(22);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::ShiftRight(left, right) => {
                buffer.push(23);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::AddressOf(expr) => {
                buffer.push(24);
                expr.encode(buffer);
            }
            AnalysisKind::Dereference(expr) => {
                buffer.push(25);
                expr.encode(buffer);
            }
            AnalysisKind::Equal(left, right) => {
                buffer.push(26);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::NotEqual(left, right) => {
                buffer.push(27);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::Less(left, right) => {
                buffer.push(28);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::LessOrEqual(left, right) => {
                buffer.push(29);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::Greater(left, right) => {
                buffer.push(30);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::GreaterOrEqual(left, right) => {
                buffer.push(31);
                left.encode(buffer);
                right.encode(buffer);
            }
            AnalysisKind::Index(index) => {
                buffer.push(32);
                index.encode(buffer);
            }
            AnalysisKind::Invoke(invoke) => {
                buffer.push(33);
                invoke.encode(buffer);
            }
            AnalysisKind::Block(stmts) => {
                buffer.push(34);
                stmts.encode(buffer);
            }
            AnalysisKind::Conditional(cond, then_branch, else_branch) => {
                buffer.push(35);
                cond.encode(buffer);
                then_branch.encode(buffer);
                else_branch.encode(buffer);
            }
            AnalysisKind::While(cond, body) => {
                buffer.push(36);
                cond.encode(buffer);
                body.encode(buffer);
            }
            AnalysisKind::Return(expr) => {
                buffer.push(37);
                expr.encode(buffer);
            }
            AnalysisKind::Break(expr) => {
                buffer.push(38);
                expr.encode(buffer);
            }
            AnalysisKind::Continue(expr) => {
                buffer.push(39);
                expr.encode(buffer);
            }
            AnalysisKind::Usage(name) => {
                buffer.push(40);
                name.encode(buffer);
            }
            AnalysisKind::Access(object, field) => {
                buffer.push(41);
                object.encode(buffer);
                field.encode(buffer);
            }
            AnalysisKind::Constructor(aggregate) => {
                buffer.push(42);
                aggregate.encode(buffer);
            }
            AnalysisKind::Assign(target, expr) => {
                buffer.push(43);
                target.encode(buffer);
                expr.encode(buffer);
            }
            AnalysisKind::Store(target, expr) => {
                buffer.push(44);
                target.encode(buffer);
                expr.encode(buffer);
            }
            AnalysisKind::Binding(binding) => {
                buffer.push(45);
                binding.encode(buffer);
            }
            AnalysisKind::Structure(aggregate) => {
                buffer.push(46);
                aggregate.encode(buffer);
            }
            AnalysisKind::Union(aggregate) => {
                buffer.push(47);
                aggregate.encode(buffer);
            }
            AnalysisKind::Function(func) => {
                buffer.push(48);
                func.encode(buffer);
            }
            AnalysisKind::Module(name, items) => {
                buffer.push(49);
                name.encode(buffer);
                items.encode(buffer);
            }
        }
    }
}

impl<'analysis> Decode<'analysis> for AnalysisKind<'analysis> {
    fn decode(buffer: &'analysis [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            0 => AnalysisKind::Integer {
                value: Integer::decode(buffer, cursor),
                size: Scale::decode(buffer, cursor),
                signed: Boolean::decode(buffer, cursor),
            },
            1 => AnalysisKind::Float {
                value: Float(f64::decode(buffer, cursor)),
                size: Scale::decode(buffer, cursor),
            },
            2 => AnalysisKind::Boolean {
                value: Boolean::decode(buffer, cursor),
            },
            3 => AnalysisKind::String {
                value: Str::decode(buffer, cursor),
            },
            4 => AnalysisKind::Character {
                value: Char::decode(buffer, cursor),
            },
            5 => AnalysisKind::Array(Vec::decode(buffer, cursor)),
            6 => AnalysisKind::Tuple(Vec::decode(buffer, cursor)),
            7 => AnalysisKind::Negate(Box::decode(buffer, cursor)),
            8 => AnalysisKind::SizeOf(Type::decode(buffer, cursor)),
            9 => AnalysisKind::Add(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            10 => AnalysisKind::Subtract(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            11 => AnalysisKind::Multiply(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            12 => AnalysisKind::Divide(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            13 => AnalysisKind::Modulus(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            14 => AnalysisKind::LogicalAnd(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            15 => AnalysisKind::LogicalOr(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            16 => AnalysisKind::LogicalNot(Box::decode(buffer, cursor)),
            17 => AnalysisKind::LogicalXOr(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            18 => AnalysisKind::BitwiseAnd(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            19 => AnalysisKind::BitwiseOr(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            20 => AnalysisKind::BitwiseNot(Box::decode(buffer, cursor)),
            21 => AnalysisKind::BitwiseXOr(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            22 => AnalysisKind::ShiftLeft(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            23 => AnalysisKind::ShiftRight(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            24 => AnalysisKind::AddressOf(Box::decode(buffer, cursor)),
            25 => AnalysisKind::Dereference(Box::decode(buffer, cursor)),
            26 => AnalysisKind::Equal(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            27 => AnalysisKind::NotEqual(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            28 => AnalysisKind::Less(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            29 => AnalysisKind::LessOrEqual(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            30 => AnalysisKind::Greater(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            31 => AnalysisKind::GreaterOrEqual(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            32 => AnalysisKind::Index(Index::decode(buffer, cursor)),
            33 => AnalysisKind::Invoke(Invoke::decode(buffer, cursor)),
            34 => AnalysisKind::Block(Vec::decode(buffer, cursor)),
            35 => AnalysisKind::Conditional(
                Box::decode(buffer, cursor),
                Box::decode(buffer, cursor),
                Option::decode(buffer, cursor),
            ),
            36 => AnalysisKind::While(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            37 => AnalysisKind::Return(Option::decode(buffer, cursor)),
            38 => AnalysisKind::Break(Option::decode(buffer, cursor)),
            39 => AnalysisKind::Continue(Option::decode(buffer, cursor)),
            40 => AnalysisKind::Usage(Str::decode(buffer, cursor)),
            41 => AnalysisKind::Access(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            42 => AnalysisKind::Constructor(Aggregate::decode(buffer, cursor)),
            43 => AnalysisKind::Assign(Str::decode(buffer, cursor), Box::decode(buffer, cursor)),
            44 => AnalysisKind::Store(Box::decode(buffer, cursor), Box::decode(buffer, cursor)),
            45 => AnalysisKind::Binding(Binding::decode(buffer, cursor)),
            46 => AnalysisKind::Structure(Aggregate::decode(buffer, cursor)),
            47 => AnalysisKind::Union(Aggregate::decode(buffer, cursor)),
            48 => AnalysisKind::Function(Function::decode(buffer, cursor)),
            49 => AnalysisKind::Module(Str::decode(buffer, cursor), Vec::decode(buffer, cursor)),
            _ => panic!("Invalid tag for AnalysisKind"),
        }
    }
}
