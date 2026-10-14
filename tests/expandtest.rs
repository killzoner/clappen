#[test]
fn expandtest() {
    let t = trybuild::TestCases::new();
    t.pass("tests/**/*.pass.rs");
    t.compile_fail("tests/**/*.fail.rs");

    let args = &["--all-features"];
    // TODO: remove glob matching if https://github.com/eupn/macrotest/pull/141 gets merged
    macrotest::expand_args("tests/expand/**/*.pass.rs", args);
}

#[test]
fn expand_file_names() {
    let pass = glob::Pattern::new("*.pass.*").unwrap();
    let fail = glob::Pattern::new("*.fail.*").unwrap();
    for file in glob::glob("tests/expand/**/*.rs").unwrap() {
        let file = file.unwrap();
        assert!(
            pass.matches_path(&file) || fail.matches_path(&file),
            "{} must match *.pass.* or *.fail.*",
            file.display()
        );
    }
}
