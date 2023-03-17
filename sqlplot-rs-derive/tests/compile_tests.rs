#[test]
fn macro_def_test() {
    trybuild::TestCases::new().pass("tests/test_code/macro_def_test.rs");
}

#[test]
fn trait_exists_test() {
    trybuild::TestCases::new().pass("tests/test_code/trait_exists_test.rs");
}

#[test]
fn skip_compile_test() {
    trybuild::TestCases::new().pass("tests/test_code/skip_compile_test.rs");
}

#[test]
fn invalid_rename_test() {
    let tc = trybuild::TestCases::new();
    tc.compile_fail("tests/test_code/invalid_rename_special_char_test.rs");
    tc.compile_fail("tests/test_code/invalid_rename_non_ascii_test.rs");
}