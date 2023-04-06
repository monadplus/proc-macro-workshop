// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run
#[derive(Debug, Default)]
enum Letters {
    #[default]
    A, 
    B, 
    C,
}

fn main() {
    let letter = Letters::default();
    println!("{letter:?}")
}
