use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote,
    visit_mut::{self, VisitMut},
    Block, Expr, ExprBlock, File,
};

pub struct DesugarIfAsync {
    pub is_async: bool,
}

impl DesugarIfAsync {
    pub fn desugar_if_async(&mut self, item: TokenStream) -> TokenStream {
        let mut syntax_tree: File = syn::parse(item.into()).unwrap();
        self.visit_file_mut(&mut syntax_tree);
        quote!(#syntax_tree)
    }

    fn rewrite_if_async(
        &self,
        node: &mut Expr,
        then_branch: Block,
        else_branch: Option<Expr>,
        expr_is_async: bool,
    ) {
        if expr_is_async == self.is_async {
            *node = Expr::Block(ExprBlock {
                attrs: vec![],
                label: None,
                block: then_branch,
            });
        } else if let Some(else_expr) = else_branch {
            *node = else_expr;
        } else {
            *node = parse_quote! {{}};
        }
    }
}

impl VisitMut for DesugarIfAsync {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        visit_mut::visit_expr_mut(self, node);

        if let Expr::If(expr_if) = &node {
            if let Expr::Path(ref var) = expr_if.cond.as_ref() {
                if let Some(first_segment) = var.path.segments.first() {
                    if var.path.segments.len() == 1 {
                        let name = first_segment.ident.to_string();
                        let then_branch = expr_if.then_branch.clone();
                        let else_branch = expr_if.else_branch.as_ref().map(|eb| *eb.1.clone());
                        match name.as_str() {
                            "_async" => {
                                self.rewrite_if_async(node, then_branch, else_branch, true);
                            }
                            "_sync" => {
                                self.rewrite_if_async(node, then_branch, else_branch, false);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
