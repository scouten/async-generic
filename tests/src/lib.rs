#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("src/tests/pass/fun-with-types.rs");
    t.pass("src/tests/pass/generic-fn.rs");
    t.pass("src/tests/pass/generic-fn-with-visibility.rs");
    t.pass("src/tests/pass/struct-method-generic.rs");

    t.compile_fail("src/tests/fail/misuse-of-underscore-async.rs");
    t.compile_fail("src/tests/fail/no-async-fn.rs");
    t.compile_fail("src/tests/fail/no-impl.rs");
    t.compile_fail("src/tests/fail/no-macro-args.rs");
    t.compile_fail("src/tests/fail/no-struct.rs");
    t.compile_fail("src/tests/fail/no-trait.rs");
}
