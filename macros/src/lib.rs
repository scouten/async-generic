#![deny(warnings)]
#![doc = include_str!("../../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg, doc_cfg_hide))]

use proc_macro::{TokenStream, TokenTree};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Error, ItemFn, Token,
};

use crate::desugar_if_async::DesugarIfAsync;

mod desugar_if_async;

fn convert_sync_async(
    input: &mut Item,
    is_async: bool,
    alt_sig: Option<TokenStream>,
) -> TokenStream2 {
    let item = &mut input.0;

    if is_async {
        item.sig.asyncness = Some(Token![async](Span::call_site()));
        item.sig.ident = Ident::new(&format!("{}_async", item.sig.ident), Span::call_site());
    }

    let tokens = quote!(#item);

    let tokens = if let Some(alt_sig) = alt_sig {
        let mut found_fn = false;
        let mut found_args = false;

        let old_tokens = tokens.into_iter().map(|token| match &token {
            TokenTree2::Ident(i) => {
                found_fn = found_fn || &i.to_string() == "fn";
                token
            }
            TokenTree2::Group(g) => {
                if found_fn && !found_args && g.delimiter() == proc_macro2::Delimiter::Parenthesis {
                    found_args = true;
                    return TokenTree2::Group(proc_macro2::Group::new(
                        proc_macro2::Delimiter::Parenthesis,
                        alt_sig.clone().into(),
                    ));
                }
                token
            }
            _ => token,
        });

        TokenStream2::from_iter(old_tokens)
    } else {
        tokens
    };

    let mut dia = DesugarIfAsync { is_async };
    dia.desugar_if_async(tokens)
}

#[proc_macro_attribute]
pub fn async_generic(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut async_signature: Option<TokenStream> = None;

    if !args.to_string().is_empty() {
        let mut atokens = args.into_iter();
        loop {
            if let Some(TokenTree::Ident(i)) = atokens.next() {
                if i.to_string() != *"async_signature" {
                    break;
                }
            } else {
                break;
            }

            if let Some(TokenTree::Group(g)) = atokens.next() {
                if atokens.next().is_none() && g.delimiter() == proc_macro::Delimiter::Parenthesis {
                    async_signature = Some(g.stream());
                }
            }
        }

        if async_signature.is_none() {
            return syn::Error::new(
                Span::call_site(),
                "async_generic can only take a async_signature argument",
            )
            .to_compile_error()
            .into();
        }
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
