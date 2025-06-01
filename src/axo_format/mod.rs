use {
    crate::{
        format, Color, TextStyle, 
        
        axo_lexer::{
            Token, TokenKind, 
            PunctuationKind, 
        },
    }
};

pub fn indent(string: &String) -> String {
    string.lines()
        .map(|line| format!("    {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_vec<Item: format::Display>(vector: &Vec<Item>) -> String {
    vector.iter().map(|form| form.to_string()).collect::<Vec<_>>().join(", ")
}

pub fn print_usage(program: &str) {
    println!("Usage: {} [OPTIONS] <file.axo>", program);
    println!("Options:");
    println!("  -v, --verbose   Enable verbose output");
    println!("  -t, --tokens    Show lexer tokens");
    println!("  -a, --ast       Show parsed AST");
    println!("  --time          Show execution time reports");
    println!("  -h, --help      Show this help message");
}

pub fn format_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .enumerate()
        //.filter(|(_, token)|
        //    token.kind != TokenKind::Punctuation(PunctuationKind::Space))
        .map(|(i, token)| {
            let token_str = match token.kind {
                TokenKind::Punctuation(PunctuationKind::Newline) => format!(
                    "↓ {:?} | {:#?} ↓\n",
                    token,
                    token.span
                )
                    .term_colorize(Color::Green)
                    .to_string(),
                TokenKind::Punctuation(_) => format!(
                    "{:?} | {:#?}",
                    token,
                    token.span
                )
                    .term_colorize(Color::Green)
                    .to_string(),
                TokenKind::Operator(_) => format!(
                    "{:?} | {:#?}",
                    token,
                    token.span
                )
                    .term_colorize(Color::Orange)
                    .to_string(),
                _ => format!("{:?} | {:#?}", token, token.span),
            };
            if i < tokens.len() - 1
                && !matches!(token.kind, TokenKind::Punctuation(PunctuationKind::Newline))
            {
                format!("{}, ", token_str)
            } else {
                token_str
            }
        })
        .collect()
}
