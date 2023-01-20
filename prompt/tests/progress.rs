#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-struct-named.rs");
    t.pass("tests/02-struct-unnamed.rs");
    t.pass("tests/03-struct-unit.rs");
    t.pass("tests/04-enum-newtype.rs");
    t.pass("tests/05-enum-unnamed.rs");
    t.pass("tests/06-enum-named.rs");
    t.pass("tests/07-enum-unit.rs");
    t.compile_fail("tests/08-enum-empty.rs");
    t.compile_fail("tests/09-trait-not-sat.rs");
    t.pass("tests/10-attr-from-str.rs");
    t.pass("tests/11-attr-help.rs");
}
