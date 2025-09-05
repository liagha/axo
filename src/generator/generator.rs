use {
    super::{
        Backend,
        GenerateError,
    },
};

pub struct Generator<'generator, B: Backend<'generator>> {
    pub backend: B,
    pub errors: Vec<GenerateError<'generator>>,
}

impl<'generator, B: Backend<'generator>> Generator<'generator, B> {
    pub fn new(backend: B) -> Self {
        Self { backend, errors: Vec::new() }
    }
}