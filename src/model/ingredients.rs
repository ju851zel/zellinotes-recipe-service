use std::convert::{TryFrom};

use bson::{Bson, Document};
use serde::Deserialize;
use serde::Serialize;

use crate::model::measurement_unit::MeasurementUnit;
use crate::model::recipe::RecipeFormatError;

const JSON_ATTR_ID: &str = "id";
const JSON_ATTR_TITLE: &str = "title";
const JSON_ATTR_AMOUNT: &str = "amount";
const JSON_ATTR_MEASUREMENT_UNIT: &str = "measurementUnit";


#[derive(Serialize, Deserialize, Debug)]
pub struct Ingredient {
    pub id: String,
    pub amount: i32,
    pub title: String,
    #[serde(rename = "measurementUnit")]
    pub measurement_unit: MeasurementUnit,
}


impl TryFrom<Bson> for Ingredient {
    type Error = RecipeFormatError;

    fn try_from(bson: Bson) -> Result<Self, Self::Error> {
        let doc = bson.as_document()
            .ok_or_else(|| "Error getting ingredients from document")?;

        return Ok(Self {
            id: doc.get_str(JSON_ATTR_ID)
                .map(String::from)
                .map_err(|_| RecipeFormatError::from(
                    "Error getting id from ingredient from document"))?,
            amount: doc.get_i32(JSON_ATTR_AMOUNT)
                .map_err(|_| RecipeFormatError::from(
                    "Error getting amount from ingredient from document"))?,
            title: doc.get_str(JSON_ATTR_TITLE)
                .map(String::from)
                .map_err(|_| RecipeFormatError::from(
                    "Error getting title from ingredient from document"))?,
            measurement_unit: doc.get_str(JSON_ATTR_MEASUREMENT_UNIT)
                .map_err(|_| RecipeFormatError::from(
                    "Error getting measurement unit from ingredient from document"))
                .and_then(MeasurementUnit::try_from)
                .map_err(|_| RecipeFormatError::from(
                    "Error converting measurement unit from ingredient to enum"))?,
        });
    }
}


impl Ingredient {
    pub fn new(id: &str, amount: i32, title: &str, measurement_unit: MeasurementUnit) -> Self {
        return Self {
            id: id.to_string(),
            amount,
            title: title.to_string(),
            measurement_unit,
        };
    }
}

impl From<Ingredient> for Bson {
    fn from(ing: Ingredient) -> Self {
        let mut doc = Document::new();
        doc.insert(JSON_ATTR_ID, ing.id);
        doc.insert(JSON_ATTR_AMOUNT, ing.amount);
        doc.insert(JSON_ATTR_TITLE, ing.title);
        doc.insert(JSON_ATTR_MEASUREMENT_UNIT, ing.measurement_unit);
        Bson::Document(doc)
    }
}

impl From<Ingredient> for Document {
    fn from(ing: Ingredient) -> Self {
        let bson: Bson = ing.into();
        bson.as_document().unwrap().to_owned()
    }
}


#[cfg(test)]
mod ingredients_tests {
    use std::convert::TryFrom;

    use bson::{Bson, Document};

    use crate::model::ingredients::{Ingredient, JSON_ATTR_AMOUNT, JSON_ATTR_ID, JSON_ATTR_MEASUREMENT_UNIT, JSON_ATTR_TITLE};
    use crate::model::measurement_unit::MeasurementUnit;

    #[test]
    fn from_bson_to_ingredient_test() {
        let ingredient = Ingredient::try_from(Bson::Document(doc! {
            "id": "0",
            "amount": 1000,
            "title": "Bread",
            "measurementUnit": "Kilogramm"
        })).unwrap();
        assert_eq!(ingredient.title, "Bread");
        assert_eq!(ingredient.amount, 1000);
        assert_eq!(ingredient.measurement_unit, MeasurementUnit::Kilogramm);
        assert_eq!(ingredient.id, "0");
    }


    #[test]
    fn from_wrong_bson_to_ingredient_test() {
        let ingredient = Ingredient::try_from(Bson::Document(
            doc! { "id": null,"amount": 1000,"title": "Bread","measurementUnit": "Kilogramm" }));
        assert_eq!(ingredient.is_err(), true);

        let ingredient = Ingredient::try_from(Bson::Document(
            doc! { "id": null,"amount": 1000,"measurementUnit": "Kilogramm" }));
        assert_eq!(ingredient.is_err(), true);

        let ingredient = Ingredient::try_from(Bson::Document(
            doc! { "id": null,"amount": 1000, "title": "john" ,"measurementUnit": "Kilogramm" }));
        assert_eq!(ingredient.is_err(), true);

        let ingredient = Ingredient::try_from(Bson::Document(
            doc! { "id": null,"amount": 1000,"measurementUnit": "Kilogramm" }));
        assert_eq!(ingredient.is_err(), true);

        let ingredient = Ingredient::try_from(Bson::Document(
            doc! { "id": null,"amount": 1000 }));
        assert_eq!(ingredient.is_err(), true);

        let ingredient = Ingredient::try_from(Bson::Document(
            doc! { "id": null,"amount": 1000, "measurementUnit": "wrong" }));
        assert_eq!(ingredient.is_err(), true);
    }


    #[test]
    fn from_ingredient_to_bson_test() {
        let ingredient = Ingredient {
            id: "0".to_string(),
            amount: 200,
            title: "wheat".to_string(),
            measurement_unit: MeasurementUnit::Kilogramm,
        };
        let bson: Document = Bson::from(ingredient).as_document().unwrap().to_owned();

        assert_eq!(bson.get_str(JSON_ATTR_ID).unwrap(), "0");
        assert_eq!(bson.get_i32(JSON_ATTR_AMOUNT).unwrap(), 200);
        assert_eq!(bson.get_str(JSON_ATTR_TITLE).unwrap(), "wheat");
        assert_eq!(bson.get_str(JSON_ATTR_MEASUREMENT_UNIT).unwrap(), MeasurementUnit::Kilogramm.to_string());
    }
}
