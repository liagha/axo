#![no_main]

use {
    axo::{
        data::Str,
        scanner::Scanner,
        tracker::Position,
    },
    libfuzzer_sys::fuzz_target,
};

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    let mut scanner = Scanner::new(Position::new(1), Str(source.as_bytes()));
    scanner.scan();

    for token in &scanner.output {
        assert!(token.span.start <= token.span.end);
    }
});
