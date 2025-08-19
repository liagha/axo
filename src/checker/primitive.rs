use crate::checker::Type;
use crate::data::any::{Any, TypeId};
use crate::internal::hash::{DefaultHasher, Hash, Hasher};
use crate::checker::types::Typable;
use crate::data;
use crate::data::{Boolean, Char, Float, Integer};
use crate::data::string::Str;
use crate::parser::{Element, Symbolic};
use crate::scanner::{Token, TokenKind};
use crate::schema::{Enumeration, Group, Structure};
use crate::tracker::{Location, Position, Span};

impl<'ty: 'static> Typable<'ty> for Integer {
    fn brand(&self) -> Option<Token<'ty>> {
        let identifier = "Integer";
        let span = Span::point(Position::new(Location::Void));

        Some(Token::new(TokenKind::Identifier(Str::from(identifier)), span))
    }

    fn as_any(&self) -> &dyn Any where Self: 'ty {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable<'ty>> {
        Box::new(self.clone())
    }

    fn dyn_eq(&self, other: &dyn Typable<'ty>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'ty: 'static> Typable<'ty> for Float {
    fn brand(&self) -> Option<Token<'ty>> {
        let identifier = "Float";
        let span = Span::point(Position::new(Location::Void));

        Some(Token::new(TokenKind::Identifier(Str::from(identifier)), span))
    }

    fn as_any(&self) -> &dyn Any where Self: 'ty {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable<'ty>> {
        Box::new(self.clone())
    }

    fn dyn_eq(&self, other: &dyn Typable<'ty>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'ty: 'static> Typable<'ty> for Boolean {
    fn brand(&self) -> Option<Token<'ty>> {
        let identifier = "Boolean";
        let span = Span::point(Position::new(Location::Void));

        Some(Token::new(TokenKind::Identifier(Str::from(identifier)), span))
    }

    fn as_any(&self) -> &dyn Any where Self: 'ty {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable<'ty>> {
        Box::new(self.clone())
    }

    fn dyn_eq(&self, other: &dyn Typable<'ty>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'ty: 'static> Typable<'ty> for Str<'ty> {
    fn brand(&self) -> Option<Token<'ty>> {
        let identifier = "String";
        let span = Span::point(Position::new(Location::Void));

        Some(Token::new(TokenKind::Identifier(Str::from(identifier)), span))
    }

    fn as_any(&self) -> &dyn Any where Self: 'ty {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable<'ty>> {
        Box::new(self.clone())
    }

    fn dyn_eq(&self, other: &dyn Typable<'ty>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'ty: 'static> Typable<'ty> for Char {
    fn brand(&self) -> Option<Token<'ty>> {
        let identifier = "Character";
        let span = Span::point(Position::new(Location::Void));

        Some(Token::new(TokenKind::Identifier(Str::from(identifier)), span))
    }

    fn as_any(&self) -> &dyn Any where Self: 'ty {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable<'ty>> {
        Box::new(self.clone())
    }

    fn dyn_eq(&self, other: &dyn Typable<'ty>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'ty: 'static> Typable<'ty> for Group<Type<'ty>> {
    fn brand(&self) -> Option<Token<'ty>> {
        let identifier = "Group";
        let span = Span::point(Position::new(Location::Void));

        Some(Token::new(TokenKind::Identifier(Str::from(identifier)), span))
    }

    fn as_any(&self) -> &dyn Any where Self: 'ty {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable<'ty>> {
        Box::new(self.clone())
    }

    fn dyn_eq(&self, other: &dyn Typable<'ty>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'ty: 'static> Typable<'ty> for Structure<Box<Element<'ty>>, Element<'ty>> {
    fn brand(&self) -> Option<Token<'ty>> {
        self.get_target().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'ty {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable<'ty>> {
        Box::new(self.clone())
    }

    fn dyn_eq(&self, other: &dyn Typable<'ty>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}

impl<'ty: 'static> Typable<'ty> for Enumeration<Box<Element<'ty>>, Element<'ty>> {
    fn brand(&self) -> Option<Token<'ty>> {
        self.get_target().brand()
    }

    fn as_any(&self) -> &dyn Any where Self: 'ty {
        self
    }

    fn dyn_clone(&self) -> Box<dyn Typable<'ty>> {
        Box::new(self.clone())
    }

    fn dyn_eq(&self, other: &dyn Typable<'ty>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&TypeId::of::<Self>(), &mut hasher);
        state.write_u64(hasher.finish());

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        state.write_u64(hasher.finish());
    }
}
