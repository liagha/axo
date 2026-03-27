pub struct Command {
    pub program: String,
    pub arguments: Vec<String>,
    pub dir: Option<String>,
}

impl Command {
    #[inline]
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            arguments: Vec::new(),
            dir: None,
        }
    }

    #[inline]
    pub fn arg(mut self, argument: impl Into<String>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    #[inline]
    pub fn args(mut self, arguments: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.arguments.extend(arguments.into_iter().map(Into::into));
        self
    }

    #[inline]
    pub fn current_dir(mut self, dir: impl Into<String>) -> Self {
        self.dir = Some(dir.into());
        self
    }
}