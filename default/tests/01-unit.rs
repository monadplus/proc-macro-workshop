#[derive(Debug, PartialEq, Eq, derive_default::Default)]
enum Letters {
    #[default]
    A, 
    B, 
    C,
}

fn main() {
    assert_eq!(Letters::default(), Letters::A);
}
