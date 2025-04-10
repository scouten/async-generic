#![deny(warnings)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg, doc_cfg_hide))]

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Attribute, Error, ItemFn, Token,
};

use crate::arguments::AsyncGenericAttributeArgs;
use crate::desugar_if_async::DesugarIfAsync;

mod arguments;
mod desugar_if_async;

fn convert_sync_async(
    input: &mut Item,
    is_async: bool,
    args: &AsyncGenericAttributeArgs,
) -> TokenStream2 {
    let item = &mut input.0;

    if is_async {
        item.sig.asyncness = Some(Token![async](Span::call_site()));
        item.sig.ident = Ident::new(&format!("{}_async", item.sig.ident), Span::call_site());
    }

    let cfg_attr = match (is_async, args.sync_cfg.as_ref(), args.async_cfg.as_ref()) {
        (false, Some(sync_cfg), _) => quote! { #[cfg(#sync_cfg)] },
        (true, _, Some(async_cfg)) => quote! { #[cfg(#async_cfg)]  },
        _ => Default::default(),
    };
    let mut tokens = quote!(#cfg_attr #item);

    if is_async {
        if let Some(alt_sig) = args.async_signature.as_ref() {
            let mut found_fn = false;
            let mut found_args = false;

            let old_tokens = tokens.into_iter().map(|token| match &token {
                TokenTree2::Ident(i) => {
                    found_fn = found_fn || &i.to_string() == "fn";
                    token
                }
                TokenTree2::Group(g) => {
                    if found_fn
                        && !found_args
                        && g.delimiter() == proc_macro2::Delimiter::Parenthesis
                    {
                        found_args = true;
                        return TokenTree2::Group(proc_macro2::Group::new(
                            proc_macro2::Delimiter::Parenthesis,
                            alt_sig.to_owned().into(),
                        ));
                    }
                    token
                }
                _ => token,
            });

            tokens = TokenStream2::from_iter(old_tokens);
        }
    }

    DesugarIfAsync { is_async }.desugar_if_async(tokens)
}

/// Produce both a sync and an async version of this function.
///
/// The async version of this function has `_async` appended to its name.
///
/// ## Arguments
///
/// This macro optionally accepts certain arguments, as follows.
///
/// ### `async_signature(<params>)`
///
/// Calling this function with an `async_signature` argument replaces its parameter list with the specified parameters.
///
/// For example:
///
/// ```rust
/// # use async_generic::async_generic;
/// # struct SyncThing;
/// # struct AsyncThing;
/// #[async_generic(async_signature(thing: &AsyncThing))]
/// fn do_stuff(thing: &SyncThing) -> String {
///     todo!()
/// }
/// ```
///
/// Expands to these functions:
///
/// ```rust
/// # struct SyncThing;
/// # struct AsyncThing;
/// fn do_stuff(thing: &SyncThing) -> String {
///     todo!()
/// }
/// async fn do_stuff_async(thing: &AsyncThing) -> String {
///     todo!()
/// }
/// ```
///
/// ### `sync_cfg(<condition>)`
///
/// Calling this function with a `sync_cfg` argument adds a conditional compilation marker to the sync version of the emitted function.
///
/// For example:
///
/// ```rust,ignore
/// #[async_generic(sync_cfg(any(test, feature = "sync")))]
/// fn do_stuff(thing: &Thing) {
///     todo!()
/// }
/// ```
///
/// Expands to these functions
///
/// ```rust,ignore
/// #[cfg(any(test, feature = "sync"))]
/// fn do_stuff(thing: &Thing) -> String {
///     todo!()
/// }
/// async fn do_stuff_async(thing: &Thing) -> String {
///     todo!()
/// }
/// ```
///
/// ### `async_cfg(<condition>)`
///
/// Calling this function with an `async_cfg` argument adds a conditional compilation marker to the asycn version of the emitted function.
///
/// For examples:
///
/// ```rust,ignore
/// #[async_generic(async_cfg(feature = "async"))]
/// fn do_stuff(thing: &Thing) {
///     todo!()
/// }
/// ```
///
/// Expands to these functions
///
/// ```rust,ignore
/// fn do_stuff(thing: &Thing) -> String {
///     todo!()
/// }
/// #[cfg(any(test, feature = "async"))]
/// async fn do_stuff_async(thing: &Thing) -> String {
///     todo!()
/// }
/// ```
#[proc_macro_attribute]
pub fn async_generic(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AsyncGenericAttributeArgs);

    let input_clone = input.clone();
    let mut item = parse_macro_input!(input_clone as Item);
    let sync_tokens = convert_sync_async(&mut item, false, &args);

    let mut item = parse_macro_input!(input as Item);
    let async_tokens = convert_sync_async(&mut item, true, &args);

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
