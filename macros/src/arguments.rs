use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result},
    token, Error, Token,
};

const ASYNC_SIG: &str = "async_signature";
const SYNC_CFG: &str = "sync_cfg";
const ASYNC_CFG: &str = "async_cfg";
const VALID_ARGUMENTS: &[&str] = &[ASYNC_SIG, SYNC_CFG, ASYNC_CFG];

/// matches `ident(...)`
struct NamedParenGroup {
    name: Ident,
    _paren: token::Paren,
    contents: TokenStream,
}

impl Parse for NamedParenGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let name = input.parse::<Ident>()?;
        if !VALID_ARGUMENTS.contains(&name.to_string().as_str()) {
            return Err(Error::new(
                name.span(),
                format!(
                    "invalid argument for async_generic. valid arguments: {}",
                    VALID_ARGUMENTS.join(", ")
                ),
            ));
        }
        let _paren = parenthesized!(content in input);
        // For whatever reason, there isn't a convenient shorthand to consume the whole
        // rest of a `ParseStream` and store it as a `TokenStream`, so we have to do it manually:
        let mut contents = TokenStream2::default();
        content.step(|cursor| {
            let mut rest = *cursor;
            while let Some((tt, next)) = rest.token_tree() {
                contents.extend(std::iter::once(tt));
                rest = next;
            }
            Ok(((), rest))
        })?;
        Ok(Self {
            name,
            _paren,
            contents: contents.into(),
        })
    }
}

#[derive(Default)]
pub struct AsyncGenericAttributeArgs {
    pub async_signature: Option<TokenStream>,
    pub sync_cfg: Option<TokenStream>,
    pub async_cfg: Option<TokenStream>,
}

impl AsyncGenericAttributeArgs {
    /// Associate the argument with one of the legal attributes by name
    fn recognize(&mut self, ident: Ident) -> Result<&mut TokenStream> {
        let field = match ident.to_string().as_str() {
            ASYNC_SIG => &mut self.async_signature,
            SYNC_CFG => &mut self.sync_cfg,
            ASYNC_CFG => &mut self.async_cfg,
            _ => {
                return Err(syn::Error::new(
                    ident.span(),
                    "unrecognized async_generic argument",
                ))
            }
        };
        if field.is_some() {
            return Err(syn::Error::new(
                ident.span(),
                "duplicate async_generic argument",
            ));
        }
        Ok(field.insert(Default::default()))
    }
}

impl Parse for AsyncGenericAttributeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = Self::default();

        for named_group in input.parse_terminated(NamedParenGroup::parse, Token![,])? {
            let field = args.recognize(named_group.name)?;
            *field = named_group.contents;
        }

        Ok(args)
    }
}
