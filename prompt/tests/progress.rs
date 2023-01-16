#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-named-structs.rs");
    t.pass("tests/02-unnamed-structs.rs");
}
