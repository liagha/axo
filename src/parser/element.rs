use {
    crate::{
        data::Str,
        formation::classifier::Classifier,
        scanner::{Token, TokenKind},
        schema::{
            Access, Assign, Binary, Delimited, Conditional,
            Index, Invoke, Cycle, Label, Procedural, While, Structure, Unary,
        },
        tracker::{Span, Spanned},
    },
    super::Symbol,
};

pub struct Element<'element> {
    pub kind: ElementKind<'element>,
    pub span: Span<'element>,
}

pub enum ElementKind<'element> {
    Literal(Token<'element>),

    Procedural(Procedural<Box<Element<'element>>>),

    Delimited(Delimited<Token<'element>, Element<'element>>), 

    Unary(Unary<Token<'element>, Box<Element<'element>>>),

    Binary(Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>>),

    Label(Label<Box<Element<'element>>, Box<Element<'element>>>),

    Access(Access<Box<Element<'element>>, Box<Element<'element>>>),

    Index(Index<Box<Element<'element>>, Element<'element>>),

    Invoke(Invoke<Box<Element<'element>>, Element<'element>>),

    Construct(Structure<Box<Element<'element>>, Element<'element>>),

    Conditional(Conditional<Box<Element<'element>>, Box<Element<'element>>, Box<Element<'element>>>),

    While(While<Box<Element<'element>>, Box<Element<'element>>>),

    Cycle(Cycle<Box<Element<'element>>, Box<Element<'element>>>),

    Symbolize(Symbol<'element>),

    Assign(Assign<Box<Element<'element>>, Box<Element<'element>>>),

    Return(Option<Box<Element<'element>>>),

    Break(Option<Box<Element<'element>>>),

    Continue(Option<Box<Element<'element>>>),
}

impl<'element> Element<'element> {
    pub fn new(kind: ElementKind<'element>, span: Span<'element>) -> Element<'element> {
        Element { kind, span }
    }
}

impl<'element> ElementKind<'element> {
    #[inline]
    pub fn literal(kind: Token<'element>) -> Self {
        ElementKind::Literal(kind)
    }

    #[inline]
    pub fn procedural(proc: Procedural<Box<Element<'element>>>) -> Self {
        ElementKind::Procedural(proc)
    }

    #[inline]
    pub fn delimited(delimited: Delimited<Token<'element>, Element<'element>>) -> Self {
        ElementKind::Delimited(delimited)
    }

    #[inline]
    pub fn unary(unary: Unary<Token<'element>, Box<Element<'element>>>) -> Self {
        ElementKind::Unary(unary)
    }

    #[inline]
    pub fn binary(binary: Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>>) -> Self {
        ElementKind::Binary(binary)
    }

    #[inline]
    pub fn label(label: Label<Box<Element<'element>>, Box<Element<'element>>>) -> Self {
        ElementKind::Label(label)
    }

    #[inline]
    pub fn access(access: Access<Box<Element<'element>>, Box<Element<'element>>>) -> Self {
        ElementKind::Access(access)
    }

    #[inline]
    pub fn index(index: Index<Box<Element<'element>>, Element<'element>>) -> Self {
        ElementKind::Index(index)
    }

    #[inline]
    pub fn invoke(invoke: Invoke<Box<Element<'element>>, Element<'element>>) -> Self {
        ElementKind::Invoke(invoke)
    }

    #[inline]
    pub fn construct(construct: Structure<Box<Element<'element>>, Element<'element>>) -> Self {
        ElementKind::Construct(construct)
    }

    #[inline]
    pub fn conditional(conditional: Conditional<Box<Element<'element>>, Box<Element<'element>>, Box<Element<'element>>>) -> Self {
        ElementKind::Conditional(conditional)
    }

    #[inline]
    pub fn repeat(repeat: While<Box<Element<'element>>, Box<Element<'element>>>) -> Self {
        ElementKind::While(repeat)
    }

    #[inline]
    pub fn iterate(iterate: Cycle<Box<Element<'element>>, Box<Element<'element>>>) -> Self {
        ElementKind::Cycle(iterate)
    }

    #[inline]
    pub fn symbolize(symbol: Symbol<'element>) -> Self {
        ElementKind::Symbolize(symbol)
    }

    #[inline]
    pub fn assign(assign: Assign<Box<Element<'element>>, Box<Element<'element>>>) -> Self {
        ElementKind::Assign(assign)
    }

    #[inline]
    pub fn produce(element: Option<Box<Element<'element>>>) -> Self {
        ElementKind::Return(element)
    }

    #[inline]
    pub fn abort(element: Option<Box<Element<'element>>>) -> Self {
        ElementKind::Break(element)
    }

    #[inline]
    pub fn pass(element: Option<Box<Element<'element>>>) -> Self {
        ElementKind::Continue(element)
    }

    #[inline(always)]
    pub fn is_literal(&self) -> bool {
        matches!(self, ElementKind::Literal(_))
    }

    #[inline(always)]
    pub fn is_procedural(&self) -> bool {
        matches!(self, ElementKind::Procedural(_))
    }

    #[inline(always)]
    pub fn is_delimited(&self) -> bool {
        matches!(self, ElementKind::Delimited(_))
    }

    #[inline(always)]
    pub fn is_unary(&self) -> bool {
        matches!(self, ElementKind::Unary(_))
    }

    #[inline(always)]
    pub fn is_binary(&self) -> bool {
        matches!(self, ElementKind::Binary(_))
    }

    #[inline(always)]
    pub fn is_label(&self) -> bool {
        matches!(self, ElementKind::Label(_))
    }

    #[inline(always)]
    pub fn is_access(&self) -> bool {
        matches!(self, ElementKind::Access(_))
    }

    #[inline(always)]
    pub fn is_index(&self) -> bool {
        matches!(self, ElementKind::Index(_))
    }

    #[inline(always)]
    pub fn is_invoke(&self) -> bool {
        matches!(self, ElementKind::Invoke(_))
    }

    #[inline(always)]
    pub fn is_construct(&self) -> bool {
        matches!(self, ElementKind::Construct(_))
    }

    #[inline(always)]
    pub fn is_conditional(&self) -> bool {
        matches!(self, ElementKind::Conditional(_))
    }

    #[inline(always)]
    pub fn is_repeat(&self) -> bool {
        matches!(self, ElementKind::While(_))
    }

    #[inline(always)]
    pub fn is_iterate(&self) -> bool {
        matches!(self, ElementKind::Cycle(_))
    }

    #[inline(always)]
    pub fn is_symbolize(&self) -> bool {
        matches!(self, ElementKind::Symbolize(_))
    }

    #[inline(always)]
    pub fn is_assign(&self) -> bool {
        matches!(self, ElementKind::Assign(_))
    }

    #[inline(always)]
    pub fn is_produce(&self) -> bool {
        matches!(self, ElementKind::Return(_))
    }

    #[inline(always)]
    pub fn is_abort(&self) -> bool {
        matches!(self, ElementKind::Break(_))
    }

    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        matches!(self, ElementKind::Continue(_))
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_literal(self) -> Token<'element> {
        match self {
            ElementKind::Literal(token_kind) => token_kind,
            _ => panic!("called `unwrap_literal` on non-Literal variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_procedural(self) -> Procedural<Box<Element<'element>>> {
        match self {
            ElementKind::Procedural(proc) => proc,
            _ => panic!("called `unwrap_procedural` on non-Procedural variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_delimited(self) -> Delimited<Token<'element>, Element<'element>> {
        match self {
            ElementKind::Delimited(delimited) => delimited,
            _ => panic!("called `unwrap_delimited` on non-Group variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_unary(self) -> Unary<Token<'element>, Box<Element<'element>>> {
        match self {
            ElementKind::Unary(unary) => unary,
            _ => panic!("called `unwrap_unary` on non-Unary variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_binary(self) -> Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>> {
        match self {
            ElementKind::Binary(binary) => binary,
            _ => panic!("called `unwrap_binary` on non-Binary variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_label(self) -> Label<Box<Element<'element>>, Box<Element<'element>>> {
        match self {
            ElementKind::Label(label) => label,
            _ => panic!("called `unwrap_label` on non-Label variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_access(self) -> Access<Box<Element<'element>>, Box<Element<'element>>> {
        match self {
            ElementKind::Access(access) => access,
            _ => panic!("called `unwrap_access` on non-Access variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_index(self) -> Index<Box<Element<'element>>, Element<'element>> {
        match self {
            ElementKind::Index(index) => index,
            _ => panic!("called `unwrap_index` on non-Index variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_invoke(self) -> Invoke<Box<Element<'element>>, Element<'element>> {
        match self {
            ElementKind::Invoke(invoke) => invoke,
            _ => panic!("called `unwrap_invoke` on non-Invoke variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_construct(self) -> Structure<Box<Element<'element>>, Element<'element>> {
        match self {
            ElementKind::Construct(construct) => construct,
            _ => panic!("called `unwrap_construct` on non-Construct variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_conditional(self) -> Conditional<Box<Element<'element>>, Box<Element<'element>>, Box<Element<'element>>> {
        match self {
            ElementKind::Conditional(conditional) => conditional,
            _ => panic!("called `unwrap_conditional` on non-Conditional variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_repeat(self) -> While<Box<Element<'element>>, Box<Element<'element>>> {
        match self {
            ElementKind::While(repeat) => repeat,
            _ => panic!("called `unwrap_repeat` on non-Repeat variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_iterate(self) -> Cycle<Box<Element<'element>>, Box<Element<'element>>> {
        match self {
            ElementKind::Cycle(iterate) => iterate,
            _ => panic!("called `unwrap_iterate` on non-Iterate variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_symbolize(self) -> Symbol<'element> {
        match self {
            ElementKind::Symbolize(symbol) => symbol,
            _ => panic!("called `unwrap_symbolize` on non-Symbolize variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_assign(self) -> Assign<Box<Element<'element>>, Box<Element<'element>>> {
        match self {
            ElementKind::Assign(assign) => assign,
            _ => panic!("called `unwrap_assign` on non-Assign variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_produce(self) -> Option<Box<Element<'element>>> {
        match self {
            ElementKind::Return(element) => element,
            _ => panic!("called `unwrap_produce` on non-Produce variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_abort(self) -> Option<Box<Element<'element>>> {
        match self {
            ElementKind::Break(element) => element,
            _ => panic!("called `unwrap_abort` on non-Abort variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_pass(self) -> Option<Box<Element<'element>>> {
        match self {
            ElementKind::Continue(element) => element,
            _ => panic!("called `unwrap_pass` on non-Pass variant."),
        }
    }

    #[inline(always)]
    pub fn try_unwrap_literal(&self) -> Option<&Token<'element>> {
        match self {
            ElementKind::Literal(token_kind) => Some(token_kind),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_procedural(&self) -> Option<&Procedural<Box<Element<'element>>>> {
        match self {
            ElementKind::Procedural(proc) => Some(proc),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_delimited(&self) -> Option<&Delimited<Token<'element>, Element<'element>>> {
        match self {
            ElementKind::Delimited(delimited) => Some(delimited),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_unary(&self) -> Option<&Unary<Token<'element>, Box<Element<'element>>>> {
        match self {
            ElementKind::Unary(unary) => Some(unary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_binary(&self) -> Option<&Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>>> {
        match self {
            ElementKind::Binary(binary) => Some(binary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_label(&self) -> Option<&Label<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Label(label) => Some(label),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_access(&self) -> Option<&Access<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Access(access) => Some(access),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_index(&self) -> Option<&Index<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Index(index) => Some(index),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_invoke(&self) -> Option<&Invoke<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Invoke(invoke) => Some(invoke),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_construct(&self) -> Option<&Structure<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Construct(construct) => Some(construct),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_conditional(&self) -> Option<&Conditional<Box<Element<'element>>, Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Conditional(conditional) => Some(conditional),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_repeat(&self) -> Option<&While<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::While(repeat) => Some(repeat),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_iterate(&self) -> Option<&Cycle<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Cycle(iterate) => Some(iterate),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_symbolize(&self) -> Option<&Symbol<'element>> {
        match self {
            ElementKind::Symbolize(symbol) => Some(symbol),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_assign(&self) -> Option<&Assign<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Assign(assign) => Some(assign),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_produce(&self) -> Option<&Option<Box<Element<'element>>>> {
        match self {
            ElementKind::Return(element) => Some(element),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_abort(&self) -> Option<&Option<Box<Element<'element>>>> {
        match self {
            ElementKind::Break(element) => Some(element),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_pass(&self) -> Option<&Option<Box<Element<'element>>>> {
        match self {
            ElementKind::Continue(element) => Some(element),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_literal_mut(&mut self) -> Option<&mut Token<'element>> {
        match self {
            ElementKind::Literal(kind) => Some(kind),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_procedural_mut(&mut self) -> Option<&mut Procedural<Box<Element<'element>>>> {
        match self {
            ElementKind::Procedural(proc) => Some(proc),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_delimited_mut(&mut self) -> Option<&mut Delimited<Token<'element>, Element<'element>>> {
        match self {
            ElementKind::Delimited(delimited) => Some(delimited),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_unary_mut(&mut self) -> Option<&mut Unary<Token<'element>, Box<Element<'element>>>> {
        match self {
            ElementKind::Unary(unary) => Some(unary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_binary_mut(&mut self) -> Option<&mut Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>>> {
        match self {
            ElementKind::Binary(binary) => Some(binary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_label_mut(&mut self) -> Option<&mut Label<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Label(label) => Some(label),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_access_mut(&mut self) -> Option<&mut Access<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Access(access) => Some(access),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_index_mut(&mut self) -> Option<&mut Index<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Index(index) => Some(index),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_invoke_mut(&mut self) -> Option<&mut Invoke<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Invoke(invoke) => Some(invoke),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_construct_mut(&mut self) -> Option<&mut Structure<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Construct(construct) => Some(construct),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_conditional_mut(&mut self) -> Option<&mut Conditional<Box<Element<'element>>, Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Conditional(conditional) => Some(conditional),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_repeat_mut(&mut self) -> Option<&mut While<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::While(repeat) => Some(repeat),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_iterate_mut(&mut self) -> Option<&mut Cycle<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Cycle(iterate) => Some(iterate),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_symbolize_mut(&mut self) -> Option<&mut Symbol<'element>> {
        match self {
            ElementKind::Symbolize(symbol) => Some(symbol),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_assign_mut(&mut self) -> Option<&mut Assign<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Assign(assign) => Some(assign),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_produce_mut(&mut self) -> Option<&mut Option<Box<Element<'element>>>> {
        match self {
            ElementKind::Return(element) => Some(element),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_abort_mut(&mut self) -> Option<&mut Option<Box<Element<'element>>>> {
        match self {
            ElementKind::Break(element) => Some(element),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_pass_mut(&mut self) -> Option<&mut Option<Box<Element<'element>>>> {
        match self {
            ElementKind::Continue(element) => Some(element),
            _ => None,
        }
    }
}