use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(strum::EnumIter)]
pub enum Color {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Violet,
}

#[derive(thiserror::Error, serde::Serialize, serde::Deserialize, Debug)]
pub enum ColorParseError {
    #[error("Invalid color")]
    InvalidColor { given_color: String },
}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "red" => Ok(Color::Red),
            "orange" => Ok(Color::Orange),
            "yellow" => Ok(Color::Yellow),
            "green" => Ok(Color::Green),
            "blue" => Ok(Color::Blue),
            "violet" => Ok(Color::Violet),
            _ => Err(ColorParseError::InvalidColor {
                given_color: s.to_string(),
            }),
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Color::Red => "red",
                Color::Orange => "orange",
                Color::Yellow => "yellow",
                Color::Green => "green",
                Color::Blue => "blue",
                Color::Violet => "violet",
            }
        )
    }
}
