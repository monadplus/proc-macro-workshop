use derive_prompt::*;

#[derive(Debug, Clone, FromPrompt)]
enum A {
    A {#[help(msg = "Write a u8")] a1: u8, a2: f32, a3:f64 },
    B(u8),
    C
}


// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run
fn main() { 
    let command = interactive::<A>().unwrap();
    println!("{command:?}")
}
