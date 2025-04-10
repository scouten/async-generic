use async_generic::async_generic;

#[async_generic(sync_cfg(target_family = "wasm"))]
fn do_stuff(thing: &AsyncThing) -> String {
    if _async {
        thing.do_stuff().await
    } else {
        thing.do_stuff()
    }
}

struct AsyncThing {}

impl AsyncThing {
    async fn do_stuff(&self) -> String {
        "async".to_owned()
    }
}

#[async_std::main]
async fn main() {
    let at = AsyncThing {};

    println!("async => {}", do_stuff_async(&at).await);
}
