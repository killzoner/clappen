#[test]
fn expandtest() {
    let t = trybuild::TestCases::new();
    t.pass("tests/**/*.pass.rs");
    t.compile_fail("tests/**/*.fail.rs");

    let args = &["--all-features"];
    macrotest::expand_args("tests/expand/**/*.rs", args);
}
