use crate::data::any::{Any, TypeId};
use crate::internal::hash::{DefaultHasher, Hash, Hasher};
use crate::checker::types::Typable;
use crate::data;
use crate::data::string::Str;
use crate::scanner::{Token, TokenKind};
use crate::tracker::{Location, Position, Span};

#[derive(Debug)]
pub struct Integer {
    pub value: data::Integer,
    pub size: data::Scale,
}

#[derive(Debug)]
pub struct Float {
    pub value: data::Float,
    pub size: data::Scale,
}

#[derive(Debug)]
pub struct Boolean {
    pub value: data::Boolean,
}

impl Typable for Integer {
    fn brand(&self) -> Option<Token<'static>> {
        let identifier = "Integer";
        let span = Span::point(Position::new(Location::Void));

        Some(Token::new(TokenKind::Identifier(Str::from(identifier)), span))
    }
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable> {
        Box::new(Self {
            value: self.value.clone(),
            size: self.size.clone(),
        })
    }

    fn dyn_eq(&self, other: &dyn Typable) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.value == other.value.clone()
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.value.hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl Typable for Float {
    fn brand(&self) -> Option<Token<'static>> {
        let identifier = "Float";
        let span = Span::point(Position::new(Location::Void));

        Some(Token::new(TokenKind::Identifier(Str::from(identifier)), span))
    }
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable> {
        Box::new(Self {
            value: self.value.clone(),
            size: self.size.clone(),
        })
    }

    fn dyn_eq(&self, other: &dyn Typable) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.value == other.value.clone()
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.value.hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl Typable for Boolean {
    fn brand(&self) -> Option<Token<'static>> {
        let identifier = "Boolean";
        let span = Span::point(Position::new(Location::Void));

        Some(Token::new(TokenKind::Identifier(Str::from(identifier)), span))
    }
    fn as_any(&self) -> &dyn Any where Self: 'static {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable> {
        Box::new(Self {
            value: self.value.clone(),
        })
    }

    fn dyn_eq(&self, other: &dyn Typable) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.value == other.value.clone()
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.value.hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}