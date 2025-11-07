#![deny(warnings)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg, doc_cfg_hide))]

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Error, ItemFn, Token,
};

use crate::desugar_if_async::DesugarIfAsync;

mod desugar_if_async;

fn convert_sync_async(
    input: &mut Item,
    is_async: bool,
    async_signature: Option<Args>,
) -> TokenStream2 {
    let item = &mut input.0;

    if is_async {
        item.sig.asyncness = Some(Token![async](Span::call_site()));
        item.sig.ident = Ident::new(&format!("{}_async", item.sig.ident), Span::call_site());
    }

    if let Some(async_signature) = async_signature {
        item.sig.inputs = async_signature.inputs;

        if let Some(generics) = async_signature.generics {
            item.sig.generics = generics;
        }

        if let Some(output) = async_signature.output {
            item.sig.output = output;
        }
    };

    let tokens = quote!(#item);

    DesugarIfAsync { is_async }.desugar_if_async(tokens)
}

#[proc_macro_attribute]
pub fn async_generic(args: TokenStream, input: TokenStream) -> TokenStream {
    let async_signature = if args.is_empty() {
        None
    } else {
        Some(parse_macro_input!(args as Args))
    };

    let input_clone = input.clone();
    let mut item = parse_macro_input!(input_clone as Item);
    let sync_tokens = convert_sync_async(&mut item, false, None);

    let mut item = parse_macro_input!(input as Item);
    let async_tokens = convert_sync_async(&mut item, true, async_signature);

    let mut tokens = sync_tokens;
    tokens.extend(async_tokens);
    tokens.into()
}

struct Args {
    generics: Option<syn::Generics>,
    inputs: syn::punctuated::Punctuated<syn::FnArg, Token![,]>,
    output: Option<syn::ReturnType>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let async_signature: Ident = input.parse()?;
        if async_signature != "async_signature" {
            return Err(Error::new(
                Span::call_site(),
                "async_generic can only take a async_signature argument",
            ));
        }

        let mut generics: Option<syn::Generics> = if input.peek(Token![<]) {
            Some(input.parse()?)
        } else {
            None
        };

        let args;
        let _paren: syn::token::Paren = parenthesized!(args in input);
        let inputs = args.parse_terminated(syn::FnArg::parse, Token![,])?;

        let output = if input.peek(Token![->]) {
            Some(input.parse()?)
        } else {
            None
        };

        if input.peek(Token![where]) {
            if let Some(generics) = &mut generics {
                generics.where_clause = Some(input.parse()?);
            } else {
                generics = Some(syn::Generics {
                    where_clause: Some(input.parse()?),
                    ..Default::default()
                });
            }
        }

        Ok(Self {
            generics,
            inputs,
            output,
        })
    }
}

struct Item(ItemFn);

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        if let Ok(mut item) = input.parse::<ItemFn>() {
            item.attrs = attrs;
            if item.sig.asyncness.is_some() {
                return Err(Error::new(
                    Span::call_site(),
                    "an async_generic function should not be declared as async",
                ));
            }
            Ok(Item(item))
        } else {
            Err(Error::new(
                Span::call_site(),
                "async_generic can only be used with functions",
            ))
        }
    }
}
