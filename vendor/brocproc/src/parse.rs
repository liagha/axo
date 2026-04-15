use std::fmt::Display;
use quote::{quote, ToTokens};
use syn::{braced, parenthesized, Expr, Token};
use syn::parse::{Parse, ParseStream};
use syn::token::{Brace, FatArrow, Paren};

#[derive(Clone)]
pub enum MacroArg {
    Simple(Expr),
    KeyValue(Expr, Expr),
    Group(Vec<MacroArg>),
    Block(Vec<MacroArg>),
}

impl Display for MacroArg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MacroArg::Simple(expr) => {
                write!(f, "{}", quote!(#expr).to_string())
            }
            MacroArg::KeyValue(key, value) => {
                write!(f, "{} => color: {}", quote!(#key).to_string(), quote!(#value).to_string())
            }
            MacroArg::Group(args) => {
                let args_str = args.iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "({})", args_str)
            }
            MacroArg::Block(args) => {
                let args_str = args.iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{ {} }}", args_str)
            }
        }
    }
}

pub struct MacroArgs {
    pub args: Vec<MacroArg>,
}

impl ToTokens for MacroArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            MacroArg::Simple(expr) => expr.to_tokens(tokens),
            MacroArg::KeyValue(key, value) => {
                quote! { (#key, #value) }.to_tokens(tokens)
            }
            MacroArg::Group(args) => {
                let expanded_args = args.iter().map(|arg| quote! { #arg });
                quote! { { #(#expanded_args)* } }.to_tokens(tokens)
            }
            MacroArg::Block(args) => {
                let expanded_args = args.iter().map(|arg| quote! { #arg });
                quote! { { #(#expanded_args)* } }.to_tokens(tokens)
            }
        }
    }
}

impl Parse for MacroArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Brace) {
            let content;
            braced!(content in input);

            let nested_args = content.parse::<MacroArgs>()?;
            Ok(MacroArg::Block(nested_args.args))
        } else if input.peek(Paren) {
            let content;
            parenthesized!(content in input);

            let nested_args = content.parse::<MacroArgs>()?;
            Ok(MacroArg::Group(nested_args.args))
        } else {
            let expr = input.parse::<Expr>()?;

            if input.peek(FatArrow) {
                input.parse::<FatArrow>()?;
                let value = input.parse::<Expr>()?;
                Ok(MacroArg::KeyValue(expr, value))
            } else {
                Ok(MacroArg::Simple(expr))
            }
        }
    }
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Vec::new();

        while !input.is_empty() {
            args.push(input.parse()?);

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(MacroArgs { args })
    }
}