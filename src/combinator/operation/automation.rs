use {
    crate::{
        combinator::{Formable, Operator, Processor},
        data::memory::Rc,
    },
    std::{
        process::Command as Terminal,
        time::{Duration, SystemTime},
    },
};

#[derive(Clone)]
pub enum Trigger {
    Immediate,
    At(SystemTime),
}

#[derive(Clone)]
pub struct Command {
    pub program: String,
    pub arguments: Vec<String>,
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Command {
    #[inline(always)]
    pub const fn succeeded(&self) -> bool {
        self.code == 0
    }
}

pub type Run = Command;

pub type TaskHook<'a, 'source, Data, Value, Failure> = Rc<
    dyn Fn(
            &mut Operator<'a, Data, Value, Failure>,
            &mut Processor<'a, 'source, Data, Value, Failure>,
            &Run,
        ) + 'source,
>;

pub struct Task<'a, 'source, Data, Value, Failure>
where
    Data: Formable<'a>,
    Value: Formable<'a>,
    Failure: Formable<'a>,
{
    pub id: String,
    pub program: String,
    pub arguments: Vec<String>,
    pub trigger: Trigger,
    pub depends: Vec<String>,
    pub dir: Option<String>,
    pub on_success: TaskHook<'a, 'source, Data, Value, Failure>,
    pub on_failure: TaskHook<'a, 'source, Data, Value, Failure>,
}

impl<'a, 'source, Data, OutputType, Failure> Task<'a, 'source, Data, OutputType, Failure>
where
    Data: Formable<'a>,
    OutputType: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    pub fn command(id: impl Into<String>, program: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            program: program.into(),
            arguments: Vec::new(),
            trigger: Trigger::Immediate,
            depends: Vec::new(),
            dir: None,
            on_success: Rc::new(|_, _, _| {}),
            on_failure: Rc::new(|_, _, _| {}),
        }
    }

    #[inline]
    pub fn arg(mut self, argument: impl Into<String>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    #[inline]
    pub fn delay(mut self, duration: Duration) -> Self {
        self.trigger = Trigger::At(SystemTime::now() + duration);
        self
    }

    #[inline]
    pub fn wait(mut self, at: SystemTime) -> Self {
        self.trigger = Trigger::At(at);
        self
    }

    #[inline]
    pub fn depend(mut self, dependency: impl Into<String>) -> Self {
        self.depends.push(dependency.into());
        self
    }

    #[inline]
    pub fn dir(mut self, directory: impl Into<String>) -> Self {
        self.dir = Some(directory.into());
        self
    }

    #[inline]
    pub fn on_success<F>(mut self, action: F) -> Self
    where
        F: Fn(
                &mut Operator<'a, Data, OutputType, Failure>,
                &mut Processor<'a, 'source, Data, OutputType, Failure>,
                &Run,
            ) + 'source,
    {
        self.on_success = Rc::new(action);
        self
    }

    #[inline]
    pub fn on_failure<F>(mut self, action: F) -> Self
    where
        F: Fn(
                &mut Operator<'a, Data, OutputType, Failure>,
                &mut Processor<'a, 'source, Data, OutputType, Failure>,
                &Run,
            ) + 'source,
    {
        self.on_failure = Rc::new(action);
        self
    }

    pub fn ready(&self, now: SystemTime) -> bool {
        match self.trigger {
            Trigger::Immediate => true,
            Trigger::At(at) => at <= now,
        }
    }

    pub fn execute(
        &self,
        operator: &mut Operator<'a, Data, OutputType, Failure>,
        processor: &mut Processor<'a, 'source, Data, OutputType, Failure>,
    ) -> Run {
        let mut command = Terminal::new(&self.program);
        command.args(&self.arguments);
        if let Some(dir) = self.dir.as_deref() {
            command.current_dir(dir);
        }

        let result = match command.output() {
            Ok(outcome) => {
                let code = outcome.status.code().unwrap_or(-1);
                Run {
                    program: self.program.clone(),
                    arguments: self.arguments.clone(),
                    code,
                    stdout: String::from_utf8_lossy(&outcome.stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&outcome.stderr).into_owned(),
                }
            }
            Err(error) => Run {
                program: self.program.clone(),
                arguments: self.arguments.clone(),
                code: -1,
                stdout: String::new(),
                stderr: error.to_string(),
            },
        };

        if result.succeeded() {
            (self.on_success)(operator, processor, &result);
        } else {
            (self.on_failure)(operator, processor, &result);
        }

        result
    }
}

pub struct Workflow<'a, 'source, Data, OutputType, Failure>
where
    Data: Formable<'a>,
    OutputType: Formable<'a>,
    Failure: Formable<'a>,
{
    pub tasks: Vec<Task<'a, 'source, Data, OutputType, Failure>>,
    pub fail_fast: bool,
}

impl<'a, 'source, Data, OutputType, Failure> Workflow<'a, 'source, Data, OutputType, Failure>
where
    Data: Formable<'a>,
    OutputType: Formable<'a>,
    Failure: Formable<'a>,
{
    #[inline]
    pub fn new(tasks: Vec<Task<'a, 'source, Data, OutputType, Failure>>) -> Self {
        Self {
            tasks,
            fail_fast: true,
        }
    }

    #[inline]
    pub fn continue_on_fail(mut self) -> Self {
        self.fail_fast = false;
        self
    }
}
