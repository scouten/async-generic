use async_generic::async_generic;

#[async_generic]
trait Trait {
    // ERROR: async_generic can not be applied to traits
    fn do_stuff() {}
}

fn main() {}
