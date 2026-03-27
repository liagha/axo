pub struct Command {
    pub program: String,
    pub arguments: Vec<String>,
    pub directory: Option<String>,
}

impl Command {
    #[inline]
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            arguments: Vec::new(),
            directory: None,
        }
    }

    #[inline]
    pub fn argument(mut self, argument: impl Into<String>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    #[inline]
    pub fn arguments(mut self, arguments: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.arguments.extend(arguments.into_iter().map(Into::into));
        self
    }

    #[inline]
    pub fn current_directory(mut self, dir: impl Into<String>) -> Self {
        self.directory = Some(dir.into());
        self
    }
}
