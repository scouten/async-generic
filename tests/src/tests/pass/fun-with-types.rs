use async_generic::async_generic;

#[async_generic(async_signature(thing: &AsyncThing))]
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
        "sync".to_owned()
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
    let st = SyncThing {};
    let at = AsyncThing {};

    println!("sync => {}", do_stuff(&st));
    println!("async => {}", do_stuff_async(&at).await);
}
