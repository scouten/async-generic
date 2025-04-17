use async_generic::async_generic;

#[async_generic]
fn do_stuff() -> String {
    #[call_generic]
    "not a function call or method call"
}

fn main() {}
