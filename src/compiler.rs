use crate::{format_tokens, indent, xprintln, Color, Lexer, Parser, Resolver, Timer, TIMERSOURCE};
use crate::tree::Tree;

pub trait Stage {
    fn entry(&mut self);
}

pub struct Compiler {
    pub resolver: Resolver,
    pub stages: Tree<Box<dyn Stage>>,
}

impl Stage for Lexer {
    fn entry(&mut self) {
        let lex_timer = Timer::new(TIMERSOURCE);

        let (tokens, errors) = self.lex();

        xprintln!("Tokens:\n{}", indent(&format_tokens(&*tokens)));

        xprintln!();

        for err in errors {
            let (msg, details) = err.format();
            xprintln!(
                "{}\n{}" => Color::Red,
                msg => Color::Orange,
                details
            );
        }

        xprintln!();

        /*
        let tokens = lexer.tokenize().unwrap_or_else(|err| {
            let (msg, details) = err.format();
            xprintln!(
                "{}\n{}" => Color::Red,
                msg => Color::Orange,
                details
            );
            if config.time_report {
                println!(
                    "Failed Compilation Took {} ns",
                    lex_timer.to_nanoseconds(lex_timer.elapsed().unwrap())
                );
            }
            process::exit(1);
        });
    
        if config.show_tokens || config.verbose {
            xprintln!("Tokens:\n{}", indent(&format_tokens(&tokens)));
            xprintln!();
        }
        */

        println!(
            "Lexing Took {} ns",
            lex_timer.to_nanoseconds(lex_timer.elapsed().unwrap())
        );
    }
}

impl Stage for Parser {
    fn entry(&mut self) {
        let parse_timer = Timer::new(TIMERSOURCE);

        let (test_elements, test_errors) = self.parse_program();

        let test_ast = test_elements
            .iter()
            .map(|element| format!("{:?}", element))
            .collect::<Vec<String>>()
            .join("\n");

        xprintln!("Test Elements:\n{}" => Color::Green, indent(&test_ast));
        xprintln!();

        for err in test_errors {
            let (msg, details) = err.format();
            xprintln!(
                "{}\n{}" => Color::Red,
                msg => Color::Orange,
                details
            );
        }

        /*
        parser.restore();

        let elements = parser.parse();

        if !parser.errors.is_empty() {
            for err in parser.errors {
                xprintln!("{}" => Color::Red, err);
            }
            if config.time_report {
                println!(
                    "Error Handling Took {} ns",
                    parse_timer.to_nanoseconds(parse_timer.elapsed().unwrap())
                );
            }
            process::exit(1);
        }

        if config.show_ast || config.verbose {
            let ast = elements
                .iter()
                .map(|element| format!("{:?}", element))
                .collect::<Vec<String>>()
                .join("\n");
            xprintln!("Elements:\n{}" => Color::Green, indent(&ast));
            xprintln!();
        }

        if config.time_report {
            println!(
                "Parsing Took {} ns",
                parse_timer.to_nanoseconds(parse_timer.elapsed().unwrap())
            );
        }
        */

    }
}