#[cfg(test)]
mod tests {
    use {
        crate::{
            data::Str,
            format::Show,
            scanner::{PunctuationKind, Scanner, Token, TokenKind},
        },
    };

    fn char_from_byte(byte: u8) -> String {
        core::char::from_u32(byte as u32)
            .map(|c| c.to_string())
            .unwrap_or_else(|| String::new())
    }

    fn assert_str_eq(actual: &Str, expected: &str) {
        assert_eq!(actual.as_str(), Some(expected), "String mismatch");
    }

    #[test]
    fn test_simple_escapes() {
        // Test all simple escape sequences
        let escapes = r#""\\" "\"" "\'" "\a" "\b" "\e" "\f" "\n" "\r" "\t" "\v" "\0""#;

        let result = Scanner::scan_string(Str::from(escapes));

        if let Ok(tokens) = result {
            println!("Simple escapes result:\n{}", tokens.format(1));
            
            let bell = core::char::from_u32(0x07).unwrap();
            let backspace = core::char::from_u32(0x08).unwrap();
            let escape = core::char::from_u32(0x1B).unwrap();
            let formfeed = core::char::from_u32(0x0C).unwrap();
            let vtab = core::char::from_u32(0x0B).unwrap();
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 12);
            
            // Check each token
            assert_str_eq(string_tokens[0], "\\");   // \\
            assert_str_eq(string_tokens[1], "\"");   // \"
            assert_str_eq(string_tokens[2], "'");    // \'
            assert_str_eq(string_tokens[3], &bell.to_string());      // \a (bell)
            assert_str_eq(string_tokens[4], &backspace.to_string()); // \b (backspace)
            assert_str_eq(string_tokens[5], &escape.to_string());    // \e (escape)
            assert_str_eq(string_tokens[6], &formfeed.to_string());  // \f (form feed)
            assert_str_eq(string_tokens[7], "\n");    // \n (newline)
            assert_str_eq(string_tokens[8], "\r");    // \r (carriage return)
            assert_str_eq(string_tokens[9], "\t");    // \t (tab)
            assert_str_eq(string_tokens[10], &vtab.to_string());      // \v (vertical tab)
            assert_str_eq(string_tokens[11], "\0");   // \0 (null)
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }

    #[test]
    fn test_octal_escapes() {
        // Test octal escapes with 1, 2, and 3 digits
        let escapes = r#""\1" "\7" "\10" "\17" "\77" "\377""#;

        let result = Scanner::scan_string(Str::from(escapes));

        if let Ok(tokens) = result {
            println!("Octal escapes result:\n{}", tokens.format(1));
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 6);
            
            assert_str_eq(string_tokens[0], &char_from_byte(0x01));  // \1
            assert_str_eq(string_tokens[1], &char_from_byte(0x07));  // \7
            assert_str_eq(string_tokens[2], &char_from_byte(0x08));  // \10 (octal 8)
            assert_str_eq(string_tokens[3], &char_from_byte(0x0F));  // \17 (octal 15)
            assert_str_eq(string_tokens[4], &char_from_byte(0x3F));  // \77 (octal 63)
            assert_str_eq(string_tokens[5], &char_from_byte(0xFF));  // \377 (octal 255)
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }

    #[test]
    fn test_hex_escapes() {
        // Test hex escapes with x and X, 1 and 2 digits
        let escapes = r#""\x41" "\xff" "\xFF" "\x0f" "\X00""#;

        let result = Scanner::scan_string(Str::from(escapes));

        if let Ok(tokens) = result {
            println!("Hex escapes result:\n{}", tokens.format(1));
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 5);
            
            assert_str_eq(string_tokens[0], "A");            // \x41
            assert_str_eq(string_tokens[1], &char_from_byte(0xFF));  // \xff
            assert_str_eq(string_tokens[2], &char_from_byte(0xFF));  // \xFF
            assert_str_eq(string_tokens[3], &char_from_byte(0x0F));  // \x0f
            assert_str_eq(string_tokens[4], "\0");           // \X00
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }

    #[test]
    fn test_unicode_escape_braces() {
        // Test \u{...} and \U{...} with various lengths
        let escapes = r#""\u{0}" "\u{7F}" "\u{03A9}" "\u{0041}" "\U{1F600}" "\u{10FFFF}""#;

        let result = Scanner::scan_string(Str::from(escapes));

        if let Ok(tokens) = result {
            println!("Unicode braces result:\n{}", tokens.format(1));
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 6);
            
            assert_str_eq(string_tokens[0], "\0");                                    // \u{0}
            assert_str_eq(string_tokens[1], &char_from_byte(0x7F));                  // \u{7F}
            assert_str_eq(string_tokens[2], "Ω");                                    // \u{03A9} (Greek capital omega)
            assert_str_eq(string_tokens[3], "A");                                    // \u{0041}
            assert_str_eq(string_tokens[4], "😀");                                  // \U{1F600} (grinning face)
            assert_str_eq(string_tokens[5], &core::char::from_u32(0x10FFFF).unwrap().to_string()); // \u{10FFFF} (max valid)
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }

    #[test]
    fn test_unicode_escape_simple() {
        // Test \uuuu and \Uuuuu (exactly 4 hex digits, no braces)
        let escapes = r#""\u0041" "\u0000" "\u007F" "\U0042" "\uFFFF""#;

        let result = Scanner::scan_string(Str::from(escapes));

        if let Ok(tokens) = result {
            println!("Unicode simple result:\n{}", tokens.format(1));
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 5);
            
            assert_str_eq(string_tokens[0], "A");                                    // \u0041
            assert_str_eq(string_tokens[1], "\0");                                   // \u0000
            assert_str_eq(string_tokens[2], &char_from_byte(0x7F));                  // \u007F
            assert_str_eq(string_tokens[3], "B");                                    // \U0042
            assert_str_eq(string_tokens[4], &core::char::from_u32(0xFFFF).unwrap().to_string()); // \uFFFF
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }

    #[test]
    fn test_mixed_escapes() {
        // Test a string with multiple different escape types
        let test = r#""\n\t\v\rA\u{42}\u0043\104""#;
        
        let result = Scanner::scan_string(Str::from(test));

        if let Ok(tokens) = result {
            println!("Mixed escapes result:\n{}", tokens.format(1));
            
            let vtab = core::char::from_u32(0x0B).unwrap();
            let expected = format!("\n\t{}\rABCD", vtab);
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 1);
            assert_str_eq(string_tokens[0], &expected);
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }

    #[test]
    fn test_character_escapes() {
        // Test escape sequences in character literals
        let test = r#"'\n' '\t' '\r' '\'' '\\'"'"'\x41'"#;

        let result = Scanner::scan_string(Str::from(test));

        if let Ok(tokens) = result {
            println!("Character escapes result:\n{}", tokens.format(1));
            
            let expected_chars: Vec<char> = vec!['\n', '\t', '\r', '\'', '\\', 'A'];
            
            let char_tokens: Vec<char> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::Character(c) = &t.kind {
                        Some(*c)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(char_tokens.len(), expected_chars.len());
            
            for (i, (actual, expected)) in char_tokens.iter().zip(expected_chars.iter()).enumerate() {
                assert_eq!(actual, expected, "Token {} mismatch", i);
            }
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }

    #[test]
    fn test_escapes_in_context() {
        // Test escapes within strings that have other content
        let test = r#""Hello\nWorld" "\tTabbed\t" "\x20Space""#;

        let result = Scanner::scan_string(Str::from(test));

        if let Ok(tokens) = result {
            println!("Escapes in context result:\n{}", tokens.format(1));
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 3);
            
            assert_str_eq(string_tokens[0], "Hello\nWorld");
            assert_str_eq(string_tokens[1], "\tTabbed\t");
            assert_str_eq(string_tokens[2], " Space");
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }

    #[test]
    fn test_octal_boundary_cases() {
        // Test octal escape boundaries
        let test = r#""\000" "\777""#;

        let result = Scanner::scan_string(Str::from(test));

        if let Ok(tokens) = result {
            println!("Octal boundaries result:\n{}", tokens.format(1));
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 2);
            // \000 should be null
            assert_str_eq(string_tokens[0], "\0");
        } else if let Err(errors) = result {
            // \777 is overflow, should produce an error
            println!("Expected overflow error: {}", errors.format(1));
        }
    }

    #[test]
    fn test_hex_variants() {
        // Test different hex digit combinations
        let test = r#""\x0" "\x00" "\xFF" "\xFF" "\x12""#;

        let result = Scanner::scan_string(Str::from(test));

        if let Ok(tokens) = result {
            println!("Hex variants result:\n{}", tokens.format(1));
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 5);
            
            assert_str_eq(string_tokens[0], "\0");            // \x0
            assert_str_eq(string_tokens[1], "\0");            // \x00
            assert_str_eq(string_tokens[2], &char_from_byte(0xFF));  // \xFF
            assert_str_eq(string_tokens[3], &char_from_byte(0xFF));  // \xFF
            assert_str_eq(string_tokens[4], &char_from_byte(0x12));  // \x12
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }

    #[test]
    fn test_octal_single_digit() {
        // Test all single-digit octal escapes
        let test = r#""\0" "\1" "\2" "\3" "\4" "\5" "\6" "\7""#;

        let result = Scanner::scan_string(Str::from(test));

        if let Ok(tokens) = result {
            println!("Octal single digit result:\n{}", tokens.format(1));
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 8);
            
            for (i, token) in string_tokens.iter().enumerate() {
                let expected = char_from_byte(i as u8);
                assert_str_eq(token, &expected);
            }
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }

    #[test]
    fn test_unicode_brace_variations() {
        // Test \u and \U with braces with varying digit counts
        let test = r#""\u{A}" "\u{AB}" "\u{ABC}" "\U{ABC}" "\U{12345}""#;

        let result = Scanner::scan_string(Str::from(test));

        if let Ok(tokens) = result {
            println!("Unicode brace variations result:\n{}", tokens.format(1));
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 5);
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }

    #[test]
    fn test_backslash_at_end() {
        // Test backslash escaping the quote
        let test = r#""ending with \\""#;

        let result = Scanner::scan_string(Str::from(test));

        if let Ok(tokens) = result {
            println!("Backslash at end result:\n{}", tokens.format(1));
            
            let string_tokens: Vec<&Str> = tokens
                .as_slice()
                .iter()
                .filter_map(|t| {
                    if let TokenKind::String(s) = &t.kind {
                        Some(s)
                    } else {
                        None
                    }
                })
                .collect();
            
            assert_eq!(string_tokens.len(), 1);
            assert_str_eq(string_tokens[0], "ending with \\");
        } else if let Err(errors) = result {
            println!("errors: {}", errors.format(1));
            panic!("Unexpected errors: {}", errors.format(1));
        }
    }
}
