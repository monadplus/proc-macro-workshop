#[derive(Debug, PartialEq, Eq, derive_default::Default)]
enum Either<L, R> {
    Left(L),
    #[default]
    Right(R),
}

fn main() {
    let either: Either<String, u8> = Either::default();
    assert_eq!(either, Either::Right(u8::default()));
}
