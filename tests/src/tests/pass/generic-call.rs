use async_generic::async_generic;

#[async_generic]
fn do_stuff() -> String {
    #[call_generic]
    do_nested_stuff()
}

#[async_generic]
fn do_nested_stuff() -> String {
    if _async {
        my_async_nested_stuff().await
    } else {
        "not async".to_owned()
    }
}

async fn my_async_nested_stuff() -> String {
    "async".to_owned()
}

struct Do;

impl Do {
    #[async_generic]
    fn stuff(&self) -> String {
        #[call_generic]
        self.nested_stuff()
    }

    #[async_generic]
    fn nested_stuff(&self) -> String {
        if _async {
            my_async_nested_stuff().await
        } else {
            "not async".to_owned()
        }
    }
}

#[async_std::main]
async fn main() {
    println!("sync => {}", do_stuff());
    println!("async => {}", do_stuff_async().await);
    println!("sync method => {}", Do.stuff());
    println!("async method => {}", Do.stuff_async().await);
}
