use colored::Colorize;
use inquire::{length, Confirm, Text};

pub use derive_prompt_impl::*;
pub use inquire::{error::InquireResult, CustomType, Select};

pub fn interactive<T: Prompt>() -> InquireResult<T> {
    <T as Prompt>::prompt("".to_string(), None)
}

pub trait Prompt: Sized {
    fn prompt(name: String, help: Option<String>) -> InquireResult<Self>;
}

macro_rules! primitive_type {
    ($($ty:ty),+ , $name:literal , $show_range:literal) => {
        $(impl Prompt for $ty {
            fn prompt(name: String, help: Option<String>) -> InquireResult<Self> {
                let help = help.unwrap_or_else(|| {
                    if $show_range {
                        format!(
                            concat!("Expecting a ", $name, " on the range [{},{}]"),
                            <$ty>::MIN,
                            <$ty>::MAX
                        )
                    } else {
                        concat!("Expecting a ", $name).to_string()
                    }
                });
                CustomType::<$ty>::new(&name)
                    .with_placeholder(concat!("<", $name, ">"))
                    .with_help_message(&help)
                    .prompt()
            }
        })*
    };
}

primitive_type!(i8, i16, i32, i64, i128, "INTEGER", true);
primitive_type!(u8, u16, u32, u64, u128, "NATURAL", true);
primitive_type!(f32, f64, "DECIMAL", false);

impl Prompt for bool {
    fn prompt(name: String, _help: Option<String>) -> InquireResult<Self> {
        Confirm::new(&name).with_default(bool::default()).prompt()
    }
}

impl Prompt for String {
    fn prompt(name: String, help: Option<String>) -> InquireResult<Self> {
        let help = help.unwrap_or_else(|| format!("Expecting a text"));
        Text::new(&name)
            .with_help_message(&help)
            .with_placeholder("<TEXT>")
            .prompt()
    }
}

impl Prompt for char {
    fn prompt(name: String, help: Option<String>) -> InquireResult<Self> {
        let help = help.unwrap_or_else(|| format!("Expecting a character"));
        Text::new(&name)
            .with_help_message(&help)
            .with_placeholder("<CHAR>")
            .with_validator(length!(1))
            .prompt()
            .map(|str| str.chars().next().unwrap())
    }
}

impl<T: Prompt> Prompt for Vec<T> {
    fn prompt(name: String, help: Option<String>) -> InquireResult<Self> {
        let mut result = Vec::new();
        println!("{}: {}", name, "<Vec>".blue().dimmed());
        let prompt_fn = |amount: usize| {
            let msg = format!("Add one more? ({} elements)", amount);
            Confirm::new(&msg).with_default(bool::default()).prompt()
        };
        let mut more = prompt_fn(result.len())?;
        while more {
            let value = T::prompt(name.clone(), help.clone())?;
            result.push(value);
            more = prompt_fn(result.len())?;
        }
        Ok(result)
    }
}

// HashMaps ?
