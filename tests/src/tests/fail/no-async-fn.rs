use async_generic::async_generic;

#[async_generic]
async fn do_some_stuff() -> bool {
    // ERROR: async should not be specified on an async_generic fn.
    true
}

fn main() {}
