use std::convert::TryFrom;

use bson::{Bson, Document};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serializer};
use serde::Serialize;

use crate::model::difficulty::Difficulty;
use crate::model::ingredients::Ingredient;
use serde::de::Error;

const JSON_ATTR_ID: &str = "_id";
const JSON_ATTR_COOKING_TIME: &str = "cookingTimeInMinutes";
const JSON_ATTR_CREATED: &str = "created";
const JSON_ATTR_LAST_MODIFIED: &str = "last_modified";
const JSON_ATTR_INGREDIENTS: &str = "ingredients";
const JSON_ATTR_VERSION: &str = "version";
const JSON_ATTR_DIFFICULTY: &str = "difficulty";
const JSON_ATTR_DESCRIPTION: &str = "description";
const JSON_ATTR_TITLE: &str = "title";
const JSON_ATTR_TAGS: &str = "tags";
const JSON_ATTR_IMAGE: &str = "image";
const JSON_ATTR_INSTRUCTIONS: &str = "instructions";
const JSON_ATTR_DEFAULT_SERVINGS: &str = "defaultServings";

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Recipe {
    #[serde(skip_deserializing)]
    #[serde(rename = "id")]
    #[serde(serialize_with = "serialize_object_id")]
    pub _id: ObjectId,
    #[serde(rename = "cookingTimeInMinutes")]
    pub cooking_time_in_minutes: u32,
    pub created: DateTime<Utc>,
    #[serde(rename = "lastModified")]
    pub last_modified: DateTime<Utc>,
    pub ingredients: Vec<Ingredient>,
    pub version: u32,
    pub difficulty: Difficulty,
    pub description: String,
    pub title: String,
    pub tags: Vec<String>,
    #[serde(rename = "image")]
    #[serde(serialize_with = "serialize_image_oid")]
    #[serde(deserialize_with = "deserialize_image_oid")]
    pub image_oid: Option<ObjectId>,
    pub instructions: Vec<String>,
    #[serde(rename = "defaultServings")]
    pub default_servings: u32,
}


fn serialize_object_id<S>(oid: &ObjectId, ser: S) -> Result<S::Ok, S::Error> where S: Serializer {
    oid.to_string().serialize(ser)
}


fn serialize_image_oid<S>(oid: &Option<ObjectId>, ser: S) -> Result<S::Ok, S::Error> where S: Serializer {
    match oid {
        Some(oid) => oid.to_string().serialize(ser),
        None => Bson::Null.serialize(ser)
    }
}

fn deserialize_image_oid<'de, D>(des: D) -> Result<Option<ObjectId>, D::Error> where D: Deserializer<'de> {
    let bson = Bson::deserialize(des)?;
    match bson {
        Bson::Null => {Ok(None)},
        Bson::String(s) => match ObjectId::with_string(&s) {
            Ok(oid) => {Ok(Some(oid))},
            Err(err) =>  Err(D::Error::custom("")),
        }
        _ => Err(D::Error::custom(""))
    }
}


#[derive(Debug, Serialize)]
pub struct RecipeFormatError { pub error: String }

impl From<&str> for RecipeFormatError {
    fn from(error: &str) -> Self { Self { error: error.to_string() } }
}

impl From<String> for RecipeFormatError {
    fn from(error: String) -> Self { Self { error } }
}

impl TryFrom<Document> for Recipe {
    type Error = RecipeFormatError;

    fn try_from(doc: Document) -> Result<Self, Self::Error> {
        return Ok(Recipe {
            _id: Recipe::extract_id(&doc)?,
            cooking_time_in_minutes: Recipe::extract_cooking_time(&doc)?,
            created: Recipe::extract_created(&doc)?,
            last_modified: Recipe::extract_last_modified(&doc)?,
            ingredients: Recipe::extract_ingredients(&doc)?,
            version: Recipe::extract_version(&doc)?,
            difficulty: Recipe::extract_difficulty(&doc)?,
            description: Recipe::extract_description(&doc)?,
            title: Recipe::extract_title(&doc)?,
            tags: Recipe::extract_tags(&doc)?,
            image_oid: Recipe::extract_image(&doc)?,
            instructions: Recipe::extract_instructions(&doc)?,
            default_servings: Recipe::extract_default_servings(&doc)?,
        });
    }
}


impl From<Recipe> for Document {
    fn from(recipe: Recipe) -> Self {
        let mut doc = Document::new();
        doc.insert(JSON_ATTR_COOKING_TIME, recipe.cooking_time_in_minutes);
        doc.insert(JSON_ATTR_CREATED, recipe.created);
        doc.insert(JSON_ATTR_LAST_MODIFIED, recipe.last_modified);
        doc.insert(JSON_ATTR_INGREDIENTS, recipe.ingredients);
        doc.insert(JSON_ATTR_VERSION, recipe.version);
        doc.insert(JSON_ATTR_DIFFICULTY, recipe.difficulty);
        doc.insert(JSON_ATTR_DESCRIPTION, recipe.description);
        doc.insert(JSON_ATTR_TITLE, recipe.title);
        doc.insert(JSON_ATTR_TAGS, recipe.tags);
        doc.insert(JSON_ATTR_IMAGE, recipe.image_oid.map_or_else(|| Bson::Null, |oid| Bson::ObjectId(oid)));
        doc.insert(JSON_ATTR_INSTRUCTIONS, recipe.instructions);
        doc.insert(JSON_ATTR_DEFAULT_SERVINGS, recipe.default_servings);
        doc
    }
}

impl Recipe {
    fn extract_difficulty(doc: &Document) -> Result<Difficulty, RecipeFormatError> {
        doc.get_str(JSON_ATTR_DIFFICULTY)
            .map(Difficulty::try_from)
            .map_err(|_| RecipeFormatError::from("Error getting difficulty from document"))?
    }

    fn extract_description(doc: &Document) -> Result<String, RecipeFormatError> {
        doc.get_str(JSON_ATTR_DESCRIPTION)
            .map(String::from)
            .map_err(|_| RecipeFormatError::from("Error getting description from document"))
    }


    fn extract_tags(doc: &Document) -> Result<Vec<String>, RecipeFormatError> {
        doc.get_array(JSON_ATTR_TAGS)
            .map_err(|_| RecipeFormatError::from("Error getting tag from document"))
            .map(|tags| {
                tags.into_iter()
                    .map(|f| f.as_str().map(String::from))
                    .collect::<Option<Vec<String>>>()
                    .ok_or_else(|| RecipeFormatError::from("Error getting tag from document"))
            })?
    }

    fn extract_image(doc: &Document) -> Result<Option<ObjectId>, RecipeFormatError> {
        match doc.get(JSON_ATTR_IMAGE) {
            Some(Bson::Null) => Ok(None),
            Some(Bson::ObjectId(oid)) => Ok(Some(oid.to_owned())),
            _ => Err(RecipeFormatError::from("Error getting image from document"))
        }
    }

    fn extract_instructions(doc: &Document) -> Result<Vec<String>, RecipeFormatError> {
        doc.get_array(JSON_ATTR_INSTRUCTIONS)
            .map_err(|_| RecipeFormatError::from("Error getting instructions from document"))
            .map(|instructions| instructions.into_iter()
                .map(|instruction| instruction.as_str().map(String::from))
                .collect::<Option<Vec<String>>>()
                .ok_or_else(|| RecipeFormatError::from("Error getting instructions from document"))
            )?
    }

    fn extract_default_servings(doc: &Document) -> Result<u32, RecipeFormatError> {
        doc.get_i32(JSON_ATTR_DEFAULT_SERVINGS)
            .map(|x| if x < 1 { 1 } else { x as u32 })
            .map_err(|_| RecipeFormatError::from("Error getting default_servings from document"))
    }

    fn extract_title(doc: &Document) -> Result<String, RecipeFormatError> {
        doc.get_str(JSON_ATTR_TITLE)
            .map(String::from)
            .map_err(|_| RecipeFormatError::from("Error getting title from document"))
    }

    fn extract_version(doc: &Document) -> Result<u32, RecipeFormatError> {
        doc.get_i32(JSON_ATTR_VERSION)
            .map(|x| x as u32)
            .map_err(|_| RecipeFormatError::from("Error getting version from document"))
    }

    fn extract_created(doc: &Document) -> Result<DateTime<Utc>, RecipeFormatError> {
        doc.get_datetime(JSON_ATTR_CREATED)
            .map(|x| x.to_owned())
            .map_err(|_| RecipeFormatError::from("Error getting created from document"))
    }

    fn extract_ingredients(doc: &Document) -> Result<Vec<Ingredient>, RecipeFormatError> {
        doc.get_array(JSON_ATTR_INGREDIENTS)
            .map_err(|_| RecipeFormatError::from("Error getting ingredients from document"))
            .map(|ingredients| ingredients.into_iter()
                .map(|ing| Ingredient::try_from(ing.clone())
                    .map_err(|_| RecipeFormatError::from("")))
                .collect::<Result<Vec<Ingredient>, RecipeFormatError>>()
            )?
    }

    fn extract_last_modified(doc: &Document) -> Result<DateTime<Utc>, RecipeFormatError> {
        doc.get_datetime(JSON_ATTR_LAST_MODIFIED)
            .map(|x| x.to_owned())
            .map_err(|_| RecipeFormatError::from("Error getting last modified from document"))
    }

    fn extract_cooking_time(doc: &Document) -> Result<u32, RecipeFormatError> {
        doc.get_i32(JSON_ATTR_COOKING_TIME)
            .map(|x| if x < 0 { 0 } else { x as u32 })
            .map_err(|_| RecipeFormatError::from("Error getting cooking timefrom document"))
    }

    fn extract_id(doc: &Document) -> Result<ObjectId, RecipeFormatError> {
        doc.get_object_id(JSON_ATTR_ID)
            .map(|x| x.to_owned())
            .map_err(|_| RecipeFormatError::from("Error getting  Object Id document"))
    }
}


#[cfg(test)]
mod convert_tests {
    use std::convert::{TryFrom, TryInto};
    use std::time::SystemTime;

    use bson::{Bson, Document};
    use bson::oid::ObjectId;
    use chrono::DateTime;

    use crate::model::difficulty::Difficulty;
    use crate::model::ingredients::Ingredient;
    use crate::model::measurement_unit::MeasurementUnit;
    use crate::model::recipe::{JSON_ATTR_COOKING_TIME,
                               JSON_ATTR_CREATED,
                               JSON_ATTR_DEFAULT_SERVINGS,
                               JSON_ATTR_DESCRIPTION,
                               JSON_ATTR_DIFFICULTY,
                               JSON_ATTR_ID,
                               JSON_ATTR_IMAGE,
                               JSON_ATTR_INGREDIENTS,
                               JSON_ATTR_INSTRUCTIONS,
                               JSON_ATTR_LAST_MODIFIED,
                               JSON_ATTR_TAGS,
                               JSON_ATTR_TITLE,
                               JSON_ATTR_VERSION,
                               Recipe,
                               RecipeFormatError};

    #[test]
    fn from_str_to_recipe_format_error_works() {
        let error = "error message";
        let rfe = RecipeFormatError::from(error);
        assert_eq!(rfe.error, error.to_string());
    }

    fn create_basic_recipe_doc() -> Document {
        let mut doc = Document::new();
        doc.insert(JSON_ATTR_ID, ObjectId::new());
        doc.insert(JSON_ATTR_COOKING_TIME, 10);
        doc.insert(JSON_ATTR_CREATED, DateTime::from(SystemTime::now()));
        doc.insert(JSON_ATTR_LAST_MODIFIED, DateTime::from(SystemTime::now()));
        doc.insert(JSON_ATTR_INGREDIENTS, vec![
            Ingredient::new("0", 100, "Cheese",
                            MeasurementUnit::Kilogramm),
            Ingredient::new("1", 200, "Bread",
                            MeasurementUnit::Piece)]);
        doc.insert(JSON_ATTR_VERSION, 1);
        doc.insert(JSON_ATTR_DIFFICULTY, Difficulty::Easy);
        doc.insert(JSON_ATTR_DESCRIPTION, "Recipe desciption");
        doc.insert(JSON_ATTR_TITLE, "Recipe title");
        doc.insert(JSON_ATTR_TAGS, vec!["vegan", "fast"]);
        doc.insert(JSON_ATTR_IMAGE, Bson::Null);
        doc.insert(JSON_ATTR_INSTRUCTIONS, vec!["do it", "do that", "do this"]);
        doc.insert(JSON_ATTR_DEFAULT_SERVINGS, 2);
        return doc;
    }

    #[test]
    fn basic_recipe_from_document() {
        let doc = create_basic_recipe_doc();
        let result = Recipe::try_from(doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);
    }

    #[test]
    fn basic_recipe_from_document_with_ignoring_value() {
        let mut doc = create_basic_recipe_doc();
        doc.insert("ignore", "value");
        let result = Recipe::try_from(doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);
    }

    #[test]
    fn basic_recipe_from_document_no_ingredients() {
        let mut doc = create_basic_recipe_doc();
        doc.insert(JSON_ATTR_INGREDIENTS, Vec::<Ingredient>::new());
        let result = Recipe::try_from(doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);
    }

    #[test]
    fn basic_recipe_from_document_with_image() {
        let mut doc = create_basic_recipe_doc();
        let image = Some("/9j/4AAQSkZJRgABAQAAAQABAAD/2Q==".to_string());
        let image = Recipe::image_insert(&image).unwrap();
        doc.insert(JSON_ATTR_IMAGE, image);
        let result = Recipe::try_from(doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);
    }

    #[test]
    fn basic_recipe_from_document_no_tags() {
        let mut doc = create_basic_recipe_doc();
        doc.insert(JSON_ATTR_TAGS, Vec::<String>::new());
        let result = Recipe::try_from(doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);
    }

    #[test]
    fn document_from_recipe() {
        let recipe: Recipe = create_basic_recipe_doc().try_into().unwrap();
        let result = Document::try_from(recipe).unwrap();
        assert_eq!(result.is_empty(), false);
    }
}

#[cfg(test)]
mod recipe_tests {
    use std::time::SystemTime;

    use bson::{Bson, Document};
    use bson::oid::ObjectId;
    use chrono::DateTime;

    use crate::model::difficulty::Difficulty;
    use crate::model::ingredients::Ingredient;
    use crate::model::measurement_unit::MeasurementUnit;
    use crate::model::recipe::{JSON_ATTR_COOKING_TIME, JSON_ATTR_CREATED, JSON_ATTR_DEFAULT_SERVINGS, JSON_ATTR_DESCRIPTION, JSON_ATTR_DIFFICULTY, JSON_ATTR_ID, JSON_ATTR_IMAGE, JSON_ATTR_INGREDIENTS, JSON_ATTR_INSTRUCTIONS, JSON_ATTR_LAST_MODIFIED, JSON_ATTR_TAGS, JSON_ATTR_TITLE, JSON_ATTR_VERSION, Recipe};

    #[test]
    fn extract_difficulty_test() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_DIFFICULTY, Difficulty::Easy);
        let result = Recipe::extract_difficulty(&doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);

        doc.insert(JSON_ATTR_DIFFICULTY, Difficulty::Medium);
        let result = Recipe::extract_difficulty(&doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);

        doc.insert(JSON_ATTR_DIFFICULTY, Difficulty::Hard);
        let result = Recipe::extract_difficulty(&doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);

        doc.insert(JSON_ATTR_DIFFICULTY, "Easy");
        let result = Recipe::extract_difficulty(&doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);

        doc.insert(JSON_ATTR_DIFFICULTY, "Super Hard");
        let result = Recipe::extract_difficulty(&doc);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_description_test() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_DESCRIPTION, "Recipe description");
        let result = Recipe::extract_description(&doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);

        doc.insert(JSON_ATTR_DESCRIPTION, "");
        let result = Recipe::extract_description(&doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);

        doc.insert(JSON_ATTR_DESCRIPTION, Bson::Null);
        let result = Recipe::extract_description(&doc);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_tags_test() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_TAGS, Vec::<String>::new());
        let result = Recipe::extract_tags(&doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);
        assert_eq!(result.unwrap(), Vec::<String>::new());

        doc.insert(JSON_ATTR_TAGS, vec!["vegan", "test", "hello"]);
        let result = Recipe::extract_tags(&doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);
        assert_eq!(result.unwrap(), vec!["vegan", "test", "hello"]);

        doc.insert(JSON_ATTR_TAGS, vec![Bson::Null]);
        let result = Recipe::extract_tags(&doc);
        assert_eq!(result.is_err(), true, "{}", result.err().unwrap().error);

        doc.insert(JSON_ATTR_TAGS, vec![Bson::Null, Bson::String("vegan".to_string())]);
        let result = Recipe::extract_tags(&doc);
        assert_eq!(result.is_err(), true, "{}", result.err().unwrap().error);
    }

    #[test]
    fn extract_image() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_IMAGE, Bson::Null);
        let result = Recipe::extract_image(&doc);
        assert_eq!(result.unwrap().is_none(), true);

        doc.insert(JSON_ATTR_IMAGE, "image");
        let result = Recipe::extract_image(&doc);
        assert_eq!(result.unwrap().is_some(), true);

        doc.insert(JSON_ATTR_IMAGE, "");
        let result = Recipe::extract_image(&doc);
        assert_eq!(result.unwrap().is_some(), true);

        doc.remove(JSON_ATTR_IMAGE);
        let result = Recipe::extract_image(&doc);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_instructions() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_INSTRUCTIONS, Vec::<String>::new());
        let result = Recipe::extract_instructions(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_INSTRUCTIONS, vec!["one", "two", "three"]);
        let result = Recipe::extract_instructions(&doc);
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap().error);

        doc.insert(JSON_ATTR_INSTRUCTIONS, Bson::Null);
        let result = Recipe::extract_instructions(&doc);
        assert_eq!(result.is_err(), true);

        doc.insert(JSON_ATTR_INSTRUCTIONS, String::new());
        let result = Recipe::extract_instructions(&doc);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_default_servings() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_DEFAULT_SERVINGS, 1);
        let result = Recipe::extract_default_servings(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_DEFAULT_SERVINGS, 0);
        let result = Recipe::extract_default_servings(&doc);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), 1);

        doc.insert(JSON_ATTR_DEFAULT_SERVINGS, 2);
        let result = Recipe::extract_default_servings(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_DEFAULT_SERVINGS, 3);
        let result = Recipe::extract_default_servings(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_DEFAULT_SERVINGS, 4);
        let result = Recipe::extract_default_servings(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_DEFAULT_SERVINGS, 5);
        let result = Recipe::extract_default_servings(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_DEFAULT_SERVINGS, -1);
        let result = Recipe::extract_default_servings(&doc);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn extract_title() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_TITLE, "Title");
        let result = Recipe::extract_title(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_TITLE, Bson::Null);
        let result = Recipe::extract_title(&doc);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_version() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_VERSION, 1);
        let result = Recipe::extract_version(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_VERSION, Bson::Null);
        let result = Recipe::extract_version(&doc);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_created() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_CREATED, DateTime::from(SystemTime::now()));
        let result = Recipe::extract_created(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_CREATED, Bson::Null);
        let result = Recipe::extract_created(&doc);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_ingredients() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_INGREDIENTS, Vec::<Ingredient>::new());
        let result = Recipe::extract_ingredients(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_INGREDIENTS, vec![
            Ingredient::new("0", 100, "Cheese",
                            MeasurementUnit::Kilogramm)]);
        let result = Recipe::extract_ingredients(&doc);
        assert_eq!(result.is_ok(), true);

        let mut ing = Document::new();
        ing.insert("id", "0");
        ing.insert("amount", "100");
        doc.insert(JSON_ATTR_INGREDIENTS, vec![
            ing,
            Ingredient::new("0",
                            100,
                            "Cheese",
                            MeasurementUnit::Kilogramm).into()
        ]);
        let result = Recipe::extract_ingredients(&doc);
        assert_eq!(result.is_err(), true);
    }


    #[test]
    fn extract_last_modified() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_LAST_MODIFIED, DateTime::from(SystemTime::now()));
        let result = Recipe::extract_last_modified(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_LAST_MODIFIED, Bson::Null);
        let result = Recipe::extract_last_modified(&doc);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn extract_cooking_time() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_COOKING_TIME, 0);
        let result = Recipe::extract_cooking_time(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_COOKING_TIME, 5);
        let result = Recipe::extract_cooking_time(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_COOKING_TIME, 300);
        let result = Recipe::extract_cooking_time(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_COOKING_TIME, 305);
        let result = Recipe::extract_cooking_time(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_COOKING_TIME, -1);
        let result = Recipe::extract_cooking_time(&doc);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn extract_id() {
        let mut doc = Document::new();

        doc.insert(JSON_ATTR_ID, ObjectId::new());
        let result = Recipe::extract_id(&doc);
        assert_eq!(result.is_ok(), true);

        doc.insert(JSON_ATTR_ID, Bson::Null);
        let result = Recipe::extract_id(&doc);
        assert_eq!(result.is_err(), true);
    }
}
