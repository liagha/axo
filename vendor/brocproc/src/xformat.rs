use quote::quote;
use crate::parse::MacroArg;

pub fn xformat_args(args: &Vec<MacroArg>) -> proc_macro::TokenStream {
    fn xformat_arg(arg: &MacroArg) -> proc_macro::TokenStream {
        match &arg {
            MacroArg::Simple(expr) => quote! {
                format!("{}", #expr)
            }.into(),
            MacroArg::KeyValue(expr, color) => quote! {
                format!(
                    "{}",
                    broccli::ColoredText {
                        content: #expr,
                        color: #color,
                    }
                )
            }.into(),
            MacroArg::Group(inner_args) => {
                let inner_tokens: proc_macro2::TokenStream = xformat_args(inner_args).into();
                quote! { #inner_tokens }.into()
            }
            MacroArg::Block(block_args) => {
                let inner_tokens: proc_macro2::TokenStream = xformat_block(block_args, 1).into();
                quote! { #inner_tokens }.into()
            }
        }
    }

    if args.is_empty() {
        return quote! { "" }.into();
    }

    if args.len() == 1 {
        return xformat_arg(&args[0]);
    }

    let mut iter = args.iter();
    let (format_string, format_color) = match iter.next().unwrap() {
        MacroArg::Simple(expr) => (quote! { #expr }, quote! { broccli::Color::Transparent }),
        MacroArg::KeyValue(expr, color) => (quote! { #expr }, quote! { #color }),
        MacroArg::Group(inner_args) => {
            let inner_tokens: proc_macro2::TokenStream = xformat_args(inner_args).into();
            (
                quote! { #inner_tokens },
                quote! { broccli::Color::Transparent },
            )
        }
        MacroArg::Block(block_args) => {
            let inner_tokens: proc_macro2::TokenStream = xformat_block(block_args, 1).into();
            (
                quote! { #inner_tokens },
                quote! { broccli::Color::Transparent },
            )
        }
    };

    let format_args = iter
        .map(|arg| xformat_arg(arg).into())
        .collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
        format!(
            "{}",
            broccli::TextStyle::term_colorize(&format!(#format_string, #(#format_args),*), #format_color)
        )
    }.into()
}

pub fn xformat_block(block: &Vec<MacroArg>, depth: u16) -> proc_macro::TokenStream {
    let depth = depth as usize;
    let mut format_string = String::new();

    let (tl, tr, bl, br, v, h, vr, vl, vh, hu, hd, branches) =
        TreeStyle::Thick.style(true);

    let output = block
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            if i != block.len() - 1 {
                format_string.push_str("{}\n");
            } else {
                format_string.push_str("{}");
            }

            let prefix = {
                let tree = if depth == 0 {
                    if i == 0 {
                        tl.to_string()
                    } else if i == block.len() - 1 {
                        bl.to_string()
                    } else {
                        vr.to_string()
                    }
                } else {
                    let root = if i == 0 {
                        tl.to_string()
                    } else if i == block.len() - 1 {
                        bl.to_string()
                    } else {
                        vr.to_string()
                    };

                    let branch = if i == 0 {
                        vr.to_string()
                    } else if i == block.len() - 1 {
                        bl.to_string()
                    } else {
                        vr.to_string()
                    };

                    if branches {
                        format!("{}  {}{}", v, branch, h.repeat(depth * 2 + 1))
                    } else {
                        let horizontal = h.repeat(depth * 2 + 1);
                        format!("{}{}", vr, horizontal)
                    }
                };

                if tree.is_empty() {
                    "".to_string()
                } else {
                    format!("{}{} ", tree, h)
                }
            };

            match arg {
                MacroArg::Simple(expr) => {
                    quote! {
                        format!("{}{}", #prefix, #expr)
                    }
                }
                MacroArg::KeyValue(expr, color) => {
                    quote! {
                        format!(
                            "{}{}",
                            broccli::TextStyle::term_colorize(&#prefix, #color),
                            broccli::ColoredText {
                                content: #expr,
                                color: #color,
                            }
                        )
                    }
                }
                MacroArg::Group(args) => {
                    let inner_args: proc_macro2::TokenStream = xformat_args(args).into();

                    quote! {
                        format!("{}{}", #prefix, #inner_args)
                    }
                }
                MacroArg::Block(block_args) => {
                    let inner_block = xformat_block(block_args, (depth + 1) as u16);
                    let inner_block: proc_macro2::TokenStream = inner_block.into();

                    quote! {
                        #inner_block
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        format!(#format_string, #(#output),*)
    }.into()
}

pub enum TreeStyle {
    Simple,
    Indent,
    Thick,
    Line,
}

impl TreeStyle {
    pub fn style(&self, branches: bool) -> (&str, &str, &str, &str, &str, &str, &str, &str, &str, &str, &str, bool) {
        match self {
            TreeStyle::Simple => {
                ("+", "+", "+", "+", "|", "-", "+", "+", "+", "+", "+", branches)
            }
            TreeStyle::Indent => {
                (" ", " ", " ", " ", " ", " ", " ", " ", " ", " ", " ", false)
            }
            TreeStyle::Thick => {
                ("╔", "╗", "╚", "╝", "║", "═", "╠", "╣", "╬", "╩", "╦", branches)
            }
            TreeStyle::Line => {
                ("┌", "┐", "└", "┘", "│", "─", "├", "┤", "┼", "┴", "┬", branches)
            }
        }
    }
}