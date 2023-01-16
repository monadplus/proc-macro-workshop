#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-named-structs.rs");
    t.pass("tests/02-unnamed-structs.rs");
    t.pass("tests/03-enums-newtype.rs");
    t.pass("tests/04-enums-unnamed.rs");
}
