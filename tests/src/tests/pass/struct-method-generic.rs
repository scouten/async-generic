use async_generic::async_generic;

struct Struct {}

impl Struct {
    #[async_generic]
    fn do_stuff(&self) -> String {
        if _async {
            self.my_async_stuff().await
        } else {
             "not async".to_owned()
        }
    }

    async fn my_async_stuff(&self) -> String {
        "async".to_owned()
    }
}

#[async_std::main]
async fn main() {
    let s = Struct {};
    println!("sync => {}", s.do_stuff());
    println!("async => {}", s.do_stuff_async().await);
}
