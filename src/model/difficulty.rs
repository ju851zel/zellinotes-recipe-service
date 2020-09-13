use std::convert;
use std::fmt;

use bson::Bson;
use futures_util::core_reexport::fmt::Formatter;
use serde::Deserialize;
use serde::Serialize;

use crate::model::recipe::RecipeFormatError;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl convert::TryFrom<&str> for Difficulty {
    type Error = RecipeFormatError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Easy" => Ok(Difficulty::Easy),
            "Medium" => Ok(Difficulty::Medium),
            "Hard" => Ok(Difficulty::Hard),
            _ => Err(format!("Difficulty '{}' does not match one predefined value", value).into())
        }
    }
}

impl From<Difficulty> for Bson {
    fn from(difficulty: Difficulty) -> Self {
        Bson::String(difficulty.to_string())
    }
}


#[cfg(test)]
mod difficulty_tests {
    use crate::model::difficulty::Difficulty;
    use std::convert::{TryInto, TryFrom};
    use bson::Bson;

    #[test]
    fn from_string_to_difficulty_test() {
        assert_eq!(Difficulty::try_from("Easy").unwrap(), Difficulty::Easy);
        assert_eq!(Difficulty::try_from("Medium").unwrap(), Difficulty::Medium);
        assert_eq!(Difficulty::try_from("Hard").unwrap(), Difficulty::Hard);
    }
    #[test]
    fn from_difficulty_to_string_test() {
        assert_eq!(Bson::from(Difficulty::Easy),Bson::String("Easy".to_string()));
        assert_eq!(Bson::from(Difficulty::Medium),Bson::String("Medium".to_string()));
        assert_eq!(Bson::from(Difficulty::Hard),Bson::String("Hard".to_string()));
    }
}
