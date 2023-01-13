use derive_prompt::*;

#[derive(Debug)]
struct Wrapper(u8);

#[derive(Debug, FromPrompt)]
#[allow(dead_code)]
pub struct Command {
    executable: String,
    iterations: u64,
    precision: f64,
    shouting: bool,
    ids: Vec<u8>,
    wrapper: Wrapper,
}

// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run
fn main() {
    let command = interactive::<Command>().unwrap();
    println!("{command:?}")
}
