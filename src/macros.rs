#[macro_export]
macro_rules! span {
    ($content:expr) => {{
        let file = file!();
        let mut hasher = $crate::internal::hash::DefaultHasher::new();
        $crate::internal::hash::Hash::hash(&file, &mut hasher);
        let identity = $crate::internal::hash::Hasher::finish(&hasher) as $crate::data::Identity;
        let start = (line!() as $crate::data::Offset) * 10000 + (column!() as $crate::data::Offset);
        let length = stringify!($content).len() as $crate::data::Offset;
        $crate::tracker::Span::range(identity, start, start + length)
    }};
}

#[macro_export]
macro_rules! float {
    ($value:expr) => {
        $crate::scanner::Token::new($crate::scanner::TokenKind::float($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! integer {
    ($value:expr) => {
        $crate::scanner::Token::new($crate::scanner::TokenKind::integer($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! boolean {
    ($value:expr) => {
        $crate::scanner::Token::new($crate::scanner::TokenKind::boolean($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! string {
    ($value:expr) => {
        $crate::scanner::Token::new($crate::scanner::TokenKind::string($crate::data::Str::from($value)), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! character {
    ($value:expr) => {
        $crate::scanner::Token::new($crate::scanner::TokenKind::character($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! operator {
    ($value:expr) => {
        $crate::scanner::Token::new($crate::scanner::TokenKind::operator($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! identifier {
    ($value:expr) => {
        $crate::scanner::Token::new($crate::scanner::TokenKind::identifier($crate::data::Str::from($value)), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! punctuation {
    ($value:expr) => {
        $crate::scanner::Token::new($crate::scanner::TokenKind::punctuation($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! literal {
    ($token:expr) => {
        $crate::parser::Element::new($crate::parser::ElementKind::literal($token), $crate::span!($token))
    };
}

#[macro_export]
macro_rules! delimited {
    ($value:expr) => {
        $crate::parser::Element::new($crate::parser::ElementKind::delimited($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! unary {
    ($value:expr) => {
        $crate::parser::Element::new($crate::parser::ElementKind::unary($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! binary {
    ($value:expr) => {
        $crate::parser::Element::new($crate::parser::ElementKind::binary($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! index {
    ($value:expr) => {
        $crate::parser::Element::new($crate::parser::ElementKind::index($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! invoke {
    ($value:expr) => {
        $crate::parser::Element::new($crate::parser::ElementKind::invoke($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! construct {
    ($value:expr) => {
        $crate::parser::Element::new($crate::parser::ElementKind::construct($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! symbolize {
    ($value:expr) => {
        $crate::parser::Element::new($crate::parser::ElementKind::symbolize($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! binding {
    ($value:expr) => {
        $crate::parser::Symbol::new($crate::parser::SymbolKind::binding($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! structure {
    ($value:expr) => {
        $crate::parser::Symbol::new($crate::parser::SymbolKind::structure($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! union {
    ($value:expr) => {
        $crate::parser::Symbol::new($crate::parser::SymbolKind::union($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! function {
    ($value:expr) => {
        $crate::parser::Symbol::new($crate::parser::SymbolKind::function($value), $crate::span!($value))
    };
}

#[macro_export]
macro_rules! module {
    ($value:expr) => {
        $crate::parser::Symbol::new($crate::parser::SymbolKind::module($value), $crate::span!($value))
    };
}