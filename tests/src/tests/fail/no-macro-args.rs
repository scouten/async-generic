use async_generic::async_generic;

#[async_generic(Send)]
fn do_stuff() -> String {
    if _async {
        my_async_stuff().await
    } else {
        "not async".to_owned()
    }
}

async fn my_async_stuff() -> String {
    "async".to_owned()
}

#[async_std::main]
async fn main() {}
