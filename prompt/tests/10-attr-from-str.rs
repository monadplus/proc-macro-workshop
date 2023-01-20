use derive_prompt::*;
use std::str::FromStr;

#[derive(Debug, Clone, FromPrompt)]
#[from_str]
struct B {
    b1: u8,
}

impl FromStr for B {
    type Err = <u8 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let b1 = s.parse()?;
        Ok(B { b1 })
    }
}

impl std::fmt::Display for B {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B {{b1: {}}}", self.b1)
    }
}

#[derive(Debug, Clone, FromPrompt)]
#[from_str]
enum C {
    C1,
    C2,
}

impl FromStr for C {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C1" => Ok(C::C1),
            "C2" => Ok(C::C2),
            _ => Err("C1 or C2".to_string()),
        }
    }
}

impl std::fmt::Display for C {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            C::C1 => write!(f, "C::C1"),
            C::C2 => write!(f, "C::C2"),
        }
    }
}

fn main() {}
