use std::str::FromStr;

use derive_prompt::*;

#[derive(Debug, Clone, FromPrompt)]
#[from_str]
enum B {
    A,
    B,
}

impl FromStr for B {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(B::A),
            "B" => Ok(B::B),
            _ => Err("A or B".to_string()),
        }
    }
}

impl std::fmt::Display for B {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            B::A => write!(f, "B::A"),
            B::B => write!(f, "B::B"),
        }
    }
}

// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run
fn main() {
    let command = interactive::<B>().unwrap();
    println!("{command:?}")
}
