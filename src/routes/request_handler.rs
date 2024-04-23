use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Copy, Clone)]
pub enum Mode {
    Production,
    Presentation,
}

impl From<String> for Mode {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "production" => Mode::Production,
            "presentation" => Mode::Presentation,
            _ => Mode::Presentation,
        }
    }
}
