#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-unit.rs");
    t.pass("tests/02-unnamed.rs");
    t.pass("tests/03-named.rs");
    t.pass("tests/04-generics.rs");
    t.compile_fail("tests/05-struct.rs");
    t.compile_fail("tests/06-empty.rs");
    t.compile_fail("tests/07-more-than-one.rs");
    // t.pass("tests/XX-associated-type.rs");
}
