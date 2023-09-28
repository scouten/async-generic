use async_generic::async_generic;

#[async_generic]
fn do_stuff() -> String {
    if _async && true {
        // ERROR: _async must stand alone in expression
        unreachable!();
    } else {
        "not async".to_owned()
    }
}

async fn my_async_stuff() -> String {
    "async".to_owned()
}

fn main() {}
