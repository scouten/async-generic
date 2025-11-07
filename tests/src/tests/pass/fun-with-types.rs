use async_generic::async_generic;
use std::future::Future;

#[async_generic(async_signature(thing: &AsyncThing))]
fn do_stuff(thing: &SyncThing) -> String {
    if _async {
        thing.do_stuff().await
    } else {
        thing.do_stuff()
    }
}

#[async_generic(async_signature(thing: &AsyncThing) -> i64)]
fn do_other_stuff(thing: &SyncThing) -> i32 {
    if _async {
        thing.do_other_stuff().await
    } else {
        thing.do_other_stuff()
    }
}

#[async_generic(async_signature<G: AsyncGeneric>(thing: &AsyncThing, g: G) -> G::AsyncOutput)]
fn do_generic_stuff<G: SyncGeneric>(thing: &SyncThing, g: G) -> G::SyncOutput {
    if _async {
        thing.do_generic_stuff(g).await
    } else {
        thing.do_generic_stuff(g)
    }
}

trait SyncGeneric {
    type SyncOutput;

    fn get(&self) -> Self::SyncOutput;
}

trait AsyncGeneric {
    type AsyncOutput;

    fn get(&self) -> impl Future<Output = Self::AsyncOutput>;
}

impl SyncGeneric for i32 {
    type SyncOutput = u32;

    fn get(&self) -> Self::SyncOutput {
        *self as u32
    }
}

impl AsyncGeneric for i64 {
    type AsyncOutput = u64;

    fn get(&self) -> impl Future<Output = Self::AsyncOutput> {
        async { *self as u64 }
    }
}

struct SyncThing {}

impl SyncThing {
    fn do_stuff(&self) -> String {
        "sync".to_owned()
    }
    fn do_other_stuff(&self) -> i32 {
        42
    }
    fn do_generic_stuff<G: SyncGeneric>(&self, g: G) -> G::SyncOutput {
        g.get()
    }
}

struct AsyncThing {}

impl AsyncThing {
    async fn do_stuff(&self) -> String {
        "async".to_owned()
    }
    async fn do_other_stuff(&self) -> i64 {
        24
    }
    async fn do_generic_stuff<G: AsyncGeneric>(&self, g: G) -> G::AsyncOutput {
        g.get().await
    }
}

#[async_std::main]
async fn main() {
    let st = SyncThing {};
    let at = AsyncThing {};

    println!("sync => {}", do_stuff(&st));
    println!("async => {}", do_stuff_async(&at).await);

    let _s: i32 = do_other_stuff(&st);
    let _a: i64 = do_other_stuff_async(&at).await;

    let _sg: u32 = do_generic_stuff(&st, 42_i32);
    let _ag: u64 = do_generic_stuff_async(&at, 24_i64).await;
}
