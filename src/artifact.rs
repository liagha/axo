use {
    crate::{
        thread::Arc,
        format::Debug,
        any::{Any, TypeId},
        hash::{Hash, Hasher},
        axo_cursor::{
            Spanned
        },
    },
};
use crate::axo_cursor::Span;

pub trait Asset: Spanned + Any + Send + Sync + Debug {
    fn dyn_hash(&self, state: &mut dyn Hasher);
    fn dyn_eq(&self, other: &dyn Asset) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl<T> Asset for T
where
    T: Spanned + Any + Send + Sync + Debug + Hash + PartialEq + 'static
{
    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state);
        
        TypeId::of::<T>().hash(&mut state);
    }

    fn dyn_eq(&self, other: &dyn Asset) -> bool {
        if let Some(other_concrete) = other.as_any().downcast_ref::<T>() {
            self == other_concrete
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct Artifact {
    inner: Arc<dyn Asset>,
}

impl Artifact {
    pub fn new<T: Spanned + Any + Send + Sync + Debug + Hash + PartialEq + 'static>(value: T) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.inner.as_any().downcast_ref()
    }
}

impl Spanned for Artifact {
    fn span(&self) -> Span {
        self.inner.span()
    }
}

impl Hash for Artifact {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.dyn_hash(state);
    }
}

impl PartialEq for Artifact {
    fn eq(&self, other: &Self) -> bool {
        self.inner.dyn_eq(&*other.inner)
    }
}

impl Eq for Artifact {}