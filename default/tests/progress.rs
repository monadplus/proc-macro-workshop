#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-unit.rs");
    t.pass("tests/02-unnamed.rs");
    // t.pass("tests/03-named.rs");
    // t.pass("tests/04-generics.rs");

    // Test: doesn't work for struct
    // Test: does't work for empty
    // Test: does't work when more than one default
}
