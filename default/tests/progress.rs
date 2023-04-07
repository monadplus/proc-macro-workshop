#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-unit.rs");
    t.pass("tests/02-unnamed.rs");
    t.pass("tests/03-named.rs");
    t.pass("tests/04-generics.rs");
    t.compile_fail("tests/05-struct.rs");
    // Test: does't work for empty
    // Test: does't work when more than one default
    // t.pass("tests/XX-associated-type.rs");
}
