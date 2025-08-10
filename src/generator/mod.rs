use {
    inkwell::{
        context::Context,
        builder::Builder,
        module::Module,
        types::BasicTypeEnum,
        values::FunctionValue,
        AddressSpace,
    },
};

pub struct Generator {
    context: Context,
}

impl Generator {
    pub fn new() -> Self {
        let context = Context::create();
        Self { context }
    }

    pub fn print(&self) {
        let module = self.context.create_module("axo_test");
        let builder = self.context.create_builder();

        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
        let function = module.add_function("add", fn_type, None);
        function.set_call_conventions(0);
        let entry = self.context.append_basic_block(function, "entry");
        builder.position_at_end(entry);

        let a = function.get_nth_param(0).unwrap().into_int_value();
        let b = function.get_nth_param(1).unwrap().into_int_value();

        let sum = builder.build_int_add(a, b, "sum").unwrap();
        builder.build_return(Some(&sum)).unwrap();

        module.print_to_file("lab/add.bc").unwrap();
    }
}