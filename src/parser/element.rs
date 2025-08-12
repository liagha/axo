use {
    crate::{
        data::string::Str,
        scanner::{Token, TokenKind},
        schema::{
            Access, Assign, Binary, Block, Bundle, Collection, Conditional, Construct, Group,
            Index, Invoke, Iterate, Label, Procedural, Repeat, Sequence, Series, Unary,
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
    Literal(TokenKind<'element>),

    Identifier(Str<'element>),

    Procedural(Procedural<Box<Element<'element>>>),

    Group(Group<Element<'element>>),

    Sequence(Sequence<Element<'element>>),

    Collection(Collection<Element<'element>>),

    Series(Series<Element<'element>>),

    Bundle(Bundle<Element<'element>>),

    Block(Block<Element<'element>>),

    Unary(Unary<Token<'element>, Box<Element<'element>>>),

    Binary(Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>>),

    Label(Label<Box<Element<'element>>, Box<Element<'element>>>),

    Access(Access<Box<Element<'element>>, Box<Element<'element>>>),

    Index(Index<Box<Element<'element>>, Element<'element>>),

    Invoke(Invoke<Box<Element<'element>>, Element<'element>>),

    Construct(Construct<Box<Element<'element>>, Element<'element>>),

    Conditional(Conditional<Box<Element<'element>>, Box<Element<'element>>, Box<Element<'element>>>),

    Repeat(Repeat<Box<Element<'element>>, Box<Element<'element>>>),

    Iterate(Iterate<Box<Element<'element>>, Box<Element<'element>>>),

    Symbolize(Symbol),

    Assign(Assign<Box<Element<'element>>, Box<Element<'element>>>),

    Produce(Option<Box<Element<'element>>>),

    Abort(Option<Box<Element<'element>>>),

    Pass(Option<Box<Element<'element>>>),
}

impl<'element> Element<'element> {
    pub fn new(kind: ElementKind<'element>, span: Span<'element>) -> Element<'element> {
        Element { kind, span }
    }
}

impl<'element> ElementKind<'element> {
    #[inline]
    pub fn literal(kind: TokenKind<'element>) -> Self {
        ElementKind::Literal(kind)
    }

    #[inline]
    pub fn identifier(name: Str<'element>) -> Self {
        ElementKind::Identifier(name)
    }

    #[inline]
    pub fn procedural(proc: Procedural<Box<Element<'element>>>) -> Self {
        ElementKind::Procedural(proc)
    }

    #[inline]
    pub fn group(group: Group<Element<'element>>) -> Self {
        ElementKind::Group(group)
    }

    #[inline]
    pub fn sequence(seq: Sequence<Element<'element>>) -> Self {
        ElementKind::Sequence(seq)
    }

    #[inline]
    pub fn collection(coll: Collection<Element<'element>>) -> Self {
        ElementKind::Collection(coll)
    }

    #[inline]
    pub fn series(series: Series<Element<'element>>) -> Self {
        ElementKind::Series(series)
    }

    #[inline]
    pub fn bundle(bundle: Bundle<Element<'element>>) -> Self {
        ElementKind::Bundle(bundle)
    }

    #[inline]
    pub fn block(block: Block<Element<'element>>) -> Self {
        ElementKind::Block(block)
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
    pub fn construct(construct: Construct<Box<Element<'element>>, Element<'element>>) -> Self {
        ElementKind::Construct(construct)
    }

    #[inline]
    pub fn conditional(conditional: Conditional<Box<Element<'element>>, Box<Element<'element>>, Box<Element<'element>>>) -> Self {
        ElementKind::Conditional(conditional)
    }

    #[inline]
    pub fn repeat(repeat: Repeat<Box<Element<'element>>, Box<Element<'element>>>) -> Self {
        ElementKind::Repeat(repeat)
    }

    #[inline]
    pub fn iterate(iterate: Iterate<Box<Element<'element>>, Box<Element<'element>>>) -> Self {
        ElementKind::Iterate(iterate)
    }

    #[inline]
    pub fn symbolize(symbol: Symbol) -> Self {
        ElementKind::Symbolize(symbol)
    }

    #[inline]
    pub fn assign(assign: Assign<Box<Element<'element>>, Box<Element<'element>>>) -> Self {
        ElementKind::Assign(assign)
    }

    #[inline]
    pub fn produce(element: Option<Box<Element<'element>>>) -> Self {
        ElementKind::Produce(element)
    }

    #[inline]
    pub fn abort(element: Option<Box<Element<'element>>>) -> Self {
        ElementKind::Abort(element)
    }

    #[inline]
    pub fn pass(element: Option<Box<Element<'element>>>) -> Self {
        ElementKind::Pass(element)
    }

    #[inline(always)]
    pub fn is_literal(&self) -> bool {
        matches!(self, ElementKind::Literal(_))
    }

    #[inline(always)]
    pub fn is_identifier(&self) -> bool {
        matches!(self, ElementKind::Identifier(_))
    }

    #[inline(always)]
    pub fn is_procedural(&self) -> bool {
        matches!(self, ElementKind::Procedural(_))
    }

    #[inline(always)]
    pub fn is_group(&self) -> bool {
        matches!(self, ElementKind::Group(_))
    }

    #[inline(always)]
    pub fn is_sequence(&self) -> bool {
        matches!(self, ElementKind::Sequence(_))
    }

    #[inline(always)]
    pub fn is_collection(&self) -> bool {
        matches!(self, ElementKind::Collection(_))
    }

    #[inline(always)]
    pub fn is_series(&self) -> bool {
        matches!(self, ElementKind::Series(_))
    }

    #[inline(always)]
    pub fn is_bundle(&self) -> bool {
        matches!(self, ElementKind::Bundle(_))
    }

    #[inline(always)]
    pub fn is_block(&self) -> bool {
        matches!(self, ElementKind::Block(_))
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
        matches!(self, ElementKind::Repeat(_))
    }

    #[inline(always)]
    pub fn is_iterate(&self) -> bool {
        matches!(self, ElementKind::Iterate(_))
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
        matches!(self, ElementKind::Produce(_))
    }

    #[inline(always)]
    pub fn is_abort(&self) -> bool {
        matches!(self, ElementKind::Abort(_))
    }

    #[inline(always)]
    pub fn is_pass(&self) -> bool {
        matches!(self, ElementKind::Pass(_))
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_literal(self) -> TokenKind<'element> {
        match self {
            ElementKind::Literal(token_kind) => token_kind,
            _ => panic!("called `unwrap_literal` on non-Literal variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_identifier(self) -> Str<'element> {
        match self {
            ElementKind::Identifier(name) => name,
            _ => panic!("called `unwrap_identifier` on non-Identifier variant."),
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
    pub fn unwrap_group(self) -> Group<Element<'element>> {
        match self {
            ElementKind::Group(group) => group,
            _ => panic!("called `unwrap_group` on non-Group variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_sequence(self) -> Sequence<Element<'element>> {
        match self {
            ElementKind::Sequence(seq) => seq,
            _ => panic!("called `unwrap_sequence` on non-Sequence variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_collection(self) -> Collection<Element<'element>> {
        match self {
            ElementKind::Collection(coll) => coll,
            _ => panic!("called `unwrap_collection` on non-Collection variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_series(self) -> Series<Element<'element>> {
        match self {
            ElementKind::Series(series) => series,
            _ => panic!("called `unwrap_series` on non-Series variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_bundle(self) -> Bundle<Element<'element>> {
        match self {
            ElementKind::Bundle(bundle) => bundle,
            _ => panic!("called `unwrap_bundle` on non-Bundle variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_block(self) -> Block<Element<'element>> {
        match self {
            ElementKind::Block(block) => block,
            _ => panic!("called `unwrap_block` on non-Block variant."),
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
    pub fn unwrap_construct(self) -> Construct<Box<Element<'element>>, Element<'element>> {
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
    pub fn unwrap_repeat(self) -> Repeat<Box<Element<'element>>, Box<Element<'element>>> {
        match self {
            ElementKind::Repeat(repeat) => repeat,
            _ => panic!("called `unwrap_repeat` on non-Repeat variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_iterate(self) -> Iterate<Box<Element<'element>>, Box<Element<'element>>> {
        match self {
            ElementKind::Iterate(iterate) => iterate,
            _ => panic!("called `unwrap_iterate` on non-Iterate variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_symbolize(self) -> Symbol {
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
            ElementKind::Produce(element) => element,
            _ => panic!("called `unwrap_produce` on non-Produce variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_abort(self) -> Option<Box<Element<'element>>> {
        match self {
            ElementKind::Abort(element) => element,
            _ => panic!("called `unwrap_abort` on non-Abort variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_pass(self) -> Option<Box<Element<'element>>> {
        match self {
            ElementKind::Pass(element) => element,
            _ => panic!("called `unwrap_pass` on non-Pass variant."),
        }
    }

    #[inline(always)]
    pub fn try_unwrap_literal(&self) -> Option<&TokenKind<'element>> {
        match self {
            ElementKind::Literal(token_kind) => Some(token_kind),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_identifier(&self) -> Option<&Str<'element>> {
        match self {
            ElementKind::Identifier(name) => Some(name),
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
    pub fn try_unwrap_group(&self) -> Option<&Group<Element<'element>>> {
        match self {
            ElementKind::Group(group) => Some(group),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_sequence(&self) -> Option<&Sequence<Element<'element>>> {
        match self {
            ElementKind::Sequence(seq) => Some(seq),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_collection(&self) -> Option<&Collection<Element<'element>>> {
        match self {
            ElementKind::Collection(coll) => Some(coll),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_series(&self) -> Option<&Series<Element<'element>>> {
        match self {
            ElementKind::Series(series) => Some(series),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_bundle(&self) -> Option<&Bundle<Element<'element>>> {
        match self {
            ElementKind::Bundle(bundle) => Some(bundle),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_block(&self) -> Option<&Block<Element<'element>>> {
        match self {
            ElementKind::Block(block) => Some(block),
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
    pub fn try_unwrap_construct(&self) -> Option<&Construct<Box<Element<'element>>, Element<'element>>> {
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
    pub fn try_unwrap_repeat(&self) -> Option<&Repeat<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Repeat(repeat) => Some(repeat),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_iterate(&self) -> Option<&Iterate<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Iterate(iterate) => Some(iterate),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_symbolize(&self) -> Option<&Symbol> {
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
            ElementKind::Produce(element) => Some(element),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_abort(&self) -> Option<&Option<Box<Element<'element>>>> {
        match self {
            ElementKind::Abort(element) => Some(element),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_pass(&self) -> Option<&Option<Box<Element<'element>>>> {
        match self {
            ElementKind::Pass(element) => Some(element),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_literal_mut(&mut self) -> Option<&mut TokenKind<'element>> {
        match self {
            ElementKind::Literal(kind) => Some(kind),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_identifier_mut(&mut self) -> Option<&mut Str<'element>> {
        match self {
            ElementKind::Identifier(name) => Some(name),
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
    pub fn try_unwrap_group_mut(&mut self) -> Option<&mut Group<Element<'element>>> {
        match self {
            ElementKind::Group(group) => Some(group),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_sequence_mut(&mut self) -> Option<&mut Sequence<Element<'element>>> {
        match self {
            ElementKind::Sequence(seq) => Some(seq),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_collection_mut(&mut self) -> Option<&mut Collection<Element<'element>>> {
        match self {
            ElementKind::Collection(coll) => Some(coll),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_series_mut(&mut self) -> Option<&mut Series<Element<'element>>> {
        match self {
            ElementKind::Series(series) => Some(series),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_bundle_mut(&mut self) -> Option<&mut Bundle<Element<'element>>> {
        match self {
            ElementKind::Bundle(bundle) => Some(bundle),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_block_mut(&mut self) -> Option<&mut Block<Element<'element>>> {
        match self {
            ElementKind::Block(block) => Some(block),
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
    pub fn try_unwrap_construct_mut(&mut self) -> Option<&mut Construct<Box<Element<'element>>, Element<'element>>> {
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
    pub fn try_unwrap_repeat_mut(&mut self) -> Option<&mut Repeat<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Repeat(repeat) => Some(repeat),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_iterate_mut(&mut self) -> Option<&mut Iterate<Box<Element<'element>>, Box<Element<'element>>>> {
        match self {
            ElementKind::Iterate(iterate) => Some(iterate),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_symbolize_mut(&mut self) -> Option<&mut Symbol> {
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
            ElementKind::Produce(element) => Some(element),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_abort_mut(&mut self) -> Option<&mut Option<Box<Element<'element>>>> {
        match self {
            ElementKind::Abort(element) => Some(element),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_pass_mut(&mut self) -> Option<&mut Option<Box<Element<'element>>>> {
        match self {
            ElementKind::Pass(element) => Some(element),
            _ => None,
        }
    }
}