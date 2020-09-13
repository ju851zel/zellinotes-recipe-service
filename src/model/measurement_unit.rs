use std::convert::TryFrom;
use std::fmt;
use std::fmt::Formatter;

use bson::Bson;
use futures_util::core_reexport::fmt::Display;
use serde::Deserialize;
use serde::Serialize;

use crate::model::recipe::RecipeFormatError;

const STR_KILOGRAMM: &str = "Kilogramm";
const STR_GRAMM: &str = "Gramm";
const STR_MILLILITER: &str = "Milliliter";
const STR_LITER: &str = "Liter";
const STR_PIECE: &str = "Piece";
const STR_PACK: &str = "Pack";

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum MeasurementUnit {
    Kilogramm,
    Gramm,
    Milliliter,
    Liter,
    Piece,
    Pack,
}

impl From<MeasurementUnit> for Bson {
    fn from(unit: MeasurementUnit) -> Self {
        Bson::String(unit.to_string())
    }
}

impl TryFrom<&str> for MeasurementUnit {
    type Error = RecipeFormatError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            STR_KILOGRAMM => Ok(MeasurementUnit::Kilogramm),
            STR_GRAMM => Ok(MeasurementUnit::Gramm),
            STR_MILLILITER => Ok(MeasurementUnit::Milliliter),
            STR_LITER => Ok(MeasurementUnit::Liter),
            STR_PIECE => Ok(MeasurementUnit::Piece),
            STR_PACK => Ok(MeasurementUnit::Pack),
            _ => Err(format!("Could not create MeasurementUnit from string: {}", value).into())
        }
    }
}

impl fmt::Display for MeasurementUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}


#[cfg(test)]
mod measurement_unit_tests {
    use bson::Bson;

    use crate::model::measurement_unit::{MeasurementUnit,
                                         STR_KILOGRAMM,
                                         STR_PACK,
                                         STR_MILLILITER,
                                         STR_GRAMM,
                                         STR_LITER,
                                         STR_PIECE};

    #[test]
    fn measurement_unit_to_bson_test() {
        assert_eq!(Bson::from(MeasurementUnit::Kilogramm).as_str().unwrap(), STR_KILOGRAMM);
        assert_eq!(Bson::from(MeasurementUnit::Gramm).as_str().unwrap(), STR_GRAMM);
        assert_eq!(Bson::from(MeasurementUnit::Milliliter).as_str().unwrap(), STR_MILLILITER);
        assert_eq!(Bson::from(MeasurementUnit::Liter).as_str().unwrap(), STR_LITER);
        assert_eq!(Bson::from(MeasurementUnit::Piece).as_str().unwrap(), STR_PIECE);
        assert_eq!(Bson::from(MeasurementUnit::Pack).as_str().unwrap(), STR_PACK);
    }
}


