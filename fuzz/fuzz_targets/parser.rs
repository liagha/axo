#![no_main]

use {
    axo::{
        data::Str,
        parser::Parser,
        scanner::Scanner,
        tracker::Position,
    },
    libfuzzer_sys::fuzz_target,
};

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    let mut scanner = Scanner::new(Position::new(1), Str(source.as_bytes()));
    scanner.scan();

    let mut parser = Parser::new();
    parser.set_input(scanner.output);
    parser.parse();

    for element in &parser.output {
        assert!(element.span.start <= element.span.end);
    }
});
