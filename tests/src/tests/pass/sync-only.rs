use async_generic::async_generic;

#[async_generic(async_cfg(target_family = "wasm"))]
fn do_stuff(thing: &SyncThing) -> String {
    if _async {
        thing.do_stuff().await
    } else {
        thing.do_stuff()
    }
}

struct SyncThing {}

impl SyncThing {
    fn do_stuff(&self) -> String {
        "async".to_owned()
    }
}

#[async_std::main]
async fn main() {
    let st = SyncThing {};

    println!("async => {}", do_stuff(&st));
}
