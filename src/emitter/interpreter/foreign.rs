use {
    crate::{
        data::Str,
        emitter::interpreter::value::Value,
    },
    std::sync::Arc,
};

pub type NativeFn<'a> = Arc<dyn Fn(&[Value<'a>]) -> Value<'a> + Send + Sync>;

#[derive(Clone)]
pub enum Foreign<'a> {
    Native(NativeFn<'a>),
}

impl<'a> Foreign<'a> {
    pub fn native<F>(f: F) -> Self
    where
        F: Fn(&[Value<'a>]) -> Value<'a> + Send + Sync + 'static,
    {
        Foreign::Native(Arc::new(f))
    }

    pub fn call(&self, args: &[Value<'a>]) -> Value<'a> {
        match self {
            Foreign::Native(f) => f(args),
        }
    }
}