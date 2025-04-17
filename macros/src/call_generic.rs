use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_quote,
    spanned::Spanned,
    visit::{self, Visit},
    visit_mut::{self, VisitMut},
    Attribute, Error, Expr, ExprCall, ExprMethodCall,
};

const ATTRIBUTE_NAME: &str = "call_generic";

pub(super) fn parse_quote_call_generic(input: TokenStream) -> TokenStream {
    let mut syntax_tree = syn::parse::<syn::File>(input.into()).unwrap();
    GenericCallVisitor.visit_file_mut(&mut syntax_tree);
    let mut visitor = OrphanAttributeVisitor::default();
    visitor.visit_file(&syntax_tree);
    if let Some(error_span) = visitor.error_span {
        return Error::new(
            error_span,
            format!("{ATTRIBUTE_NAME} must be used on a method call or a function call."),
        )
        .into_compile_error();
    }
    quote! {#syntax_tree}
}

/// Finds an orphan attribute that hasn't been handled by [GenericCallVisitor].
#[derive(Default)]
struct OrphanAttributeVisitor {
    error_span: Option<Span>,
}

impl Visit<'_> for OrphanAttributeVisitor {
    fn visit_attribute(&mut self, node: &Attribute) {
        if node
            .path()
            .segments
            .last()
            .map(|path| path.ident.to_string())
            == Some(ATTRIBUTE_NAME.to_string())
        {
            self.error_span = Some(node.span());
        }
        visit::visit_attribute(self, node);
    }
}

/// Finds and replaces a method call or function call annotated with `#[call_generic]`.
/// This enables `#[call_generic]` to be used as a shorthand for
/// ```rust,ignore
/// if _sync {
///     do_stuff();
/// } else {
///     do_stuff_async().await;
/// }
/// ```
struct GenericCallVisitor;

impl VisitMut for GenericCallVisitor {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        let async_call = match node {
            Expr::MethodCall(expr) => construct_async_method_call(expr),
            Expr::Call(expr) => construct_async_function_call(expr),
            _ => None,
        };

        if let Some(async_call) = async_call {
            let original_call = node.clone();
            *node = parse_quote! {
                if _async {
                    #async_call.await
                } else {
                    #original_call
               }
            };
            return;
        }

        // Delegate to the default impl to visit nested expressions.
        visit_mut::visit_expr_mut(self, node);
    }
}

fn construct_async_method_call(expr: &mut ExprMethodCall) -> Option<Expr> {
    find_and_remove_generic_call_attr(&mut expr.attrs)?;
    let mut async_expr = expr.clone();
    async_expr.method = format_ident!("{}_async", async_expr.method);
    Some(Expr::MethodCall(async_expr))
}

fn construct_async_function_call(expr: &mut ExprCall) -> Option<Expr> {
    find_and_remove_generic_call_attr(&mut expr.attrs)?;
    let mut async_expr = expr.clone();
    let mut func = *async_expr.func;
    let Expr::Path(ref mut path_expr) = &mut func else {
        return None;
    };
    let last_segment = path_expr.path.segments.last_mut()?;
    last_segment.ident = format_ident!("{}_async", last_segment.ident);
    async_expr.func = Box::new(func);
    Some(Expr::Call(async_expr))
}

fn find_and_remove_generic_call_attr(attributes: &mut Vec<Attribute>) -> Option<()> {
    let length_before_removal = attributes.len();
    attributes.retain(|attr| {
        let Some(last_segment) = attr.path().segments.last() else {
            return true;
        };
        last_segment.ident != ATTRIBUTE_NAME
    });
    (length_before_removal > attributes.len()).then_some(())
}
