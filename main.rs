use derive_prompt::*;

#[derive(Debug, FromPrompt)]
struct Newtype(bool);

#[derive(Debug, FromPrompt)]
struct Wrapper(u8, String);

#[derive(Debug, FromPrompt)]
#[allow(dead_code)]
pub struct Command {
    executable: String,
    iterations: u64,
    precision: f64,
    shouting: bool,
    ids: Vec<u8>,
    newtype: Newtype,
    wrapper: Wrapper,
}

#[derive(Debug, FromPrompt)]
pub struct A1(u8);

#[derive(Debug, FromPrompt)]
pub struct B1(u8, u8);

#[derive(Debug, FromPrompt)]
pub struct C1(u8, u8, String);

#[derive(Debug, FromPrompt)]
pub enum Choice {
    A(A1),
    B(B1, C1),
}

// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run
fn main() {
    let command = interactive::<Choice>().unwrap();
    println!("{command:?}")
}
