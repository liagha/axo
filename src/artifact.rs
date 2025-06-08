use {
    crate::{
        thread::Arc,
        format::Debug,
        any::{Any, TypeId},
        hash::{Hash, Hasher},
    },
};

pub trait Asset: Any + Send + Sync + Debug {
    fn dyn_hash(&self, state: &mut dyn Hasher);
    fn dyn_eq(&self, other: &dyn Asset) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl<T> Asset for T
where
    T: Any + Send + Sync + Debug + Hash + PartialEq + 'static
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

#[derive(Debug, Clone)]
pub struct Artifact {
    inner: Arc<dyn Asset>,
}

impl Artifact {
    pub fn new<T: Any + Send + Sync + Debug + Hash + PartialEq + 'static>(value: T) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.inner.as_any().downcast_ref()
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