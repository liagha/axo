use crate::{
    analyzer::Analysis,
    data::Str,
    emitter::interpreter::{
        compiler::{Chunk, Compiler},
        error::InterpretError,
        machine::Machine,
        value::Value,
        Foreign,
    },
};

pub struct Engine<'a> {
    pub machine: Machine<'a>,
}

impl<'a> Engine<'a> {
    pub fn new() -> Self {
        let mut engine = Self {
            machine: Machine::new(),
        };
        engine.register_base();
        engine
    }

    fn register_base(&mut self) {
        self.machine.register(
            Str::from("print_integer"),
            Foreign::native(|args| {
                if let Some(Value::Integer(n)) = args.first() {
                    print!("{}", n);
                }
                Value::Void
            }),
        );

        self.machine.register(
            Str::from("print_float"),
            Foreign::native(|args| {
                if let Some(Value::Float(f)) = args.first() {
                    print!("{}", f);
                }
                Value::Void
            }),
        );

        self.machine.register(
            Str::from("print_boolean"),
            Foreign::native(|args| {
                if let Some(Value::Boolean(b)) = args.first() {
                    print!("{}", b);
                }
                Value::Void
            }),
        );

        self.machine.register(
            Str::from("print_character"),
            Foreign::native(|args| {
                if let Some(Value::Character(c)) = args.first() {
                    print!("{}", c);
                }
                Value::Void
            }),
        );

        self.machine.register(
            Str::from("print_string"),
            Foreign::native(|args| {
                if let Some(Value::String(s)) = args.first() {
                    print!("{}", s);
                }
                Value::Void
            }),
        );

        self.machine.register(
            Str::from("println_integer"),
            Foreign::native(|args| {
                if let Some(Value::Integer(n)) = args.first() {
                    println!("{}", n);
                }
                Value::Void
            }),
        );

        self.machine.register(
            Str::from("println_float"),
            Foreign::native(|args| {
                if let Some(Value::Float(f)) = args.first() {
                    println!("{}", f);
                }
                Value::Void
            }),
        );

        self.machine.register(
            Str::from("println_boolean"),
            Foreign::native(|args| {
                if let Some(Value::Boolean(b)) = args.first() {
                    println!("{}", b);
                }
                Value::Void
            }),
        );

        self.machine.register(
            Str::from("println_character"),
            Foreign::native(|args| {
                if let Some(Value::Character(c)) = args.first() {
                    println!("{}", c);
                }
                Value::Void
            }),
        );

        self.machine.register(
            Str::from("println_string"),
            Foreign::native(|args| {
                if let Some(Value::String(s)) = args.first() {
                    println!("{}", s);
                }
                Value::Void
            }),
        );

        self.machine.register(
            Str::from("print_newline"),
            Foreign::native(|_| {
                println!();
                Value::Void
            }),
        );
    }

    pub fn register(&mut self, name: Str<'a>, foreign: Foreign<'a>) {
        self.machine.register(name, foreign);
    }

    pub fn execute(
        &mut self,
        analyses: Vec<Analysis<'a>>,
    ) -> Result<Value<'a>, InterpretError<'a>> {
        self.machine.load(&analyses)?;

        let mut compiler = Compiler::new();
        let mut chunk = Chunk::new();
        compiler.compile(&analyses, &mut chunk)?;

        self.machine.run(&chunk)
    }

    pub fn process(
        &mut self,
        analyses: Vec<Analysis<'a>>,
    ) -> Result<Value<'a>, InterpretError<'a>> {
        self.machine.load(&analyses)?;

        let non_functions: Vec<_> = analyses
            .into_iter()
            .filter(|a| !matches!(&a.kind, crate::analyzer::AnalysisKind::Function(_)))
            .collect();

        let mut compiler = Compiler::new();
        let mut chunk = Chunk::new();
        compiler.compile(&non_functions, &mut chunk)?;

        self.machine.run(&chunk)
    }
}
