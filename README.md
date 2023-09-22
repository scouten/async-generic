# async-generic

Write code that can be both async and synchronous without duplicating it.

## Why solve this problem now?

This macro set is intended as an interim solution for the problem space that will eventually be covered by the Rust [Keyword Generic Initiative](https://blog.rust-lang.org/inside-rust/2022/07/27/keyword-generics.html).

As of this writing (September 2023), the official [status of that project](https://github.com/rust-lang/keyword-generics-initiative) is listed as still in the "Experimental" stage, so deployment in the language is still likely many months away if not longer.

So ... what do we do _now_ if we need async-generic code? We build our own, using proc macros. And that's exactly what this crate is.

I'll happily mark this crate as deprecated when keyword generics land officially in the language. Until then, hopefully it solves some problems for you, too!

## User's guide

The `async_generic` crate introduces a single proc macro also named `async_generic` which can be applied as an attribute to any function (either inside a struct or not).

The macro outputs _two_ versions of the function, one synchronous and one that's async. The functions are identical to each other, except as follows:

* When writing the async flavor of the function, the macro inserts the `async` modifier for you and renames the function (to avoid a name collision) by adding an `_async` suffix to the existing function name.
* The attribute macro _may_ contain an `async_signature` argument. If that exists, the async function's argument parameters are replaced. (See example below.)
* You can write `if _sync` or `if _async` blocks inside this block. The contents of these blocks will only appear in the corresponding sync or async flavors of the functions. You _may_ specify an `else` clause, which will only appear in the opposite flavor of the function. You may not combine `_sync` or `_async` with any other expression. (These aren't _really_ variables in the function scope, and they will cause "undefined identifier" errors if you try that.)

A simple example:

```rust
use async_generic::async_generic;

#[async_generic]
fn do_stuff() -> String {
    // Also: async fn do_stuff_async() -> String {...}
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
async fn main() {
    println!("sync => {}", do_stuff());
    println!("async => {}", do_stuff_async().await);
}
```

An example with different function arguments in the sync and async flavors:

```rust
use async_generic::async_generic;

#[async_generic(async_signature(thing: &AsyncThing))]
fn do_stuff(thing: &SyncThing) -> String {
    // Also: async fn do_stuff_async(thing: &AsyncThing) -> String {...}
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
```

## Why not use `maybe-async`?

This crate is loosely derived from the excellent work of the [`maybe-async`](https://crates.io/crates/maybe-async) crate, but is intended to solve a subtly different problem.

Use `maybe-async` when you know at compile-time whether each crate in your dependency tree will be used in an async or synchronous fashion. In that model, you can't have both at once.

Use `async-generic` when you wish to have both async and synchronous versions of an API at the same time and want to reuse most of the implementation.
