#[cfg(test)]
mod tests {
    use {
        crate::{
            data::{
                Str, Float,
            },
            format::Show,
            scanner::{OperatorKind, PunctuationKind, Scanner, Token, TokenKind},
        },
    };

    #[test]
    fn test() {
        let numbers =
            "0x10\n0XFF\n0xbeef_cafe\n0b1111\n0B101_010\n0o10\n0o777\n42\n1_000_000\n3.14\n1.0e3\n-5.5e-2\n1_234.56\n1.\n.5";

        let result = Scanner::scan_string(Str::from(numbers));

        if let Ok(tokens) = result {
            println!("{}", tokens.format(1));

            assert!(
                matches!(
                    tokens.as_slice(),
                    [
                        Token { kind: TokenKind::Integer(16), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Integer(255), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Integer(3203386110), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Integer(15), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Integer(42), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Integer(8), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Integer(511), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Integer(42), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Integer(1000000), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Float(Float(3.14)), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Float(Float(1000.0)), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Float(Float(-0.055)), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Float(Float(1234.56)), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Float(Float(1.0)), .. },
                        Token { kind: TokenKind::Punctuation(PunctuationKind::Newline), .. },
                        Token { kind: TokenKind::Float(Float(0.5)), .. },
                    ]
                )
            )
        }
    }
}
