use crate::{
    boolean, character,
    data::{Delimited, Function, Interface},
    delimited, function, identifier, integer, literal,
    parser::{Element, Symbol},
    punctuation,
    resolver::Resolver,
    scanner::PunctuationKind,
    string,
};

impl<'resolver> Resolver<'resolver> {
    pub fn builtin(target: &Element<'resolver>) -> Option<Symbol<'resolver>> {
        let name = target.target()?;

        match name.as_str()? {
            "Int8" => Some(Resolver::function("Int8", "Integer")),
            "Int16" => Some(Resolver::function("Int16", "Integer")),
            "Int32" => Some(Resolver::function("Int32", "Integer")),
            "Int64" => Some(Resolver::function("Int64", "Integer")),
            "Integer" => Some(Resolver::function("Integer", "Integer")),
            "UInt8" => Some(Resolver::function("UInt8", "Integer")),
            "UInt16" => Some(Resolver::function("UInt16", "Integer")),
            "UInt32" => Some(Resolver::function("UInt32", "Integer")),
            "UInt64" => Some(Resolver::function("UInt64", "Integer")),
            "Float32" => Some(Resolver::function("Float32", "Float")),
            "Float64" => Some(Resolver::function("Float64", "Float")),
            "Float" => Some(Resolver::function("Float", "Float")),
            "Boolean" => Some(Resolver::function("Boolean", "Boolean")),
            "Character" => Some(Resolver::function("Character", "Character")),
            "String" => Some(Resolver::function("String", "String")),
            "Void" => Some(Resolver::function("Void", "Void")),
            "if" => Some(Resolver::statement("if")),
            "while" => Some(Resolver::statement("while")),
            "break" => Some(Resolver::statement("break")),
            "continue" => Some(Resolver::statement("continue")),
            "return" => Some(Resolver::statement("return")),
            _ => None,
        }
    }

    fn statement(name: &'static str) -> Symbol<'resolver> {
        let target = literal!(string!(name));
        let body = delimited!(Delimited::new(
            punctuation!(PunctuationKind::LeftBrace),
            Vec::new(),
            None,
            punctuation!(PunctuationKind::RightBrace),
        ));

        function!(Function::new(
            target,
            Vec::new(),
            Some(body),
            None,
            Interface::Compiler,
            false,
            false,
        ))
    }

    fn function(name: &'static str, output: &'static str) -> Symbol<'resolver> {
        let target = literal!(identifier!(name));
        let annotation = literal!(string!(output));

        let body = match output {
            "Integer" => literal!(integer!(0)),
            "Float" => literal!(integer!(0)),
            "Boolean" => literal!(boolean!(false)),
            "Character" => literal!(character!('a')),
            "String" => literal!(string!("")),
            _ => delimited!(Delimited::new(
                punctuation!(PunctuationKind::LeftBrace),
                Vec::new(),
                None,
                punctuation!(PunctuationKind::RightBrace),
            )),
        };

        function!(Function::new(
            target,
            Vec::new(),
            Some(body),
            Some(annotation),
            Interface::Compiler,
            false,
            false,
        ))
    }
}
