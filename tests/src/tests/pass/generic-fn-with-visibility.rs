pub mod fun {
    use async_generic::async_generic;

    #[async_generic]
    pub fn do_stuff() -> String {
        if _async {
            my_async_stuff().await
        } else {
            "not async".to_owned()
        }
    }

    async fn my_async_stuff() -> String {
        "async".to_owned()
    }
}

mod test {
    pub async fn test() {
        println!("sync => {}", super::fun::do_stuff());
        println!("async => {}", super::fun::do_stuff_async().await);
    }
}

#[async_std::main]
async fn main() {
    test::test().await
}
