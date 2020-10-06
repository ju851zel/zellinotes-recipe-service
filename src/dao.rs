use std::convert::TryFrom;

use bson::Document;
use bson::document::ValueAccessError;
use bson::oid::ObjectId;
use futures_util::StreamExt;
use mongodb::{bson::{Bson, doc}, Client, options::FindOptions};
use mongodb::Database;
use mongodb::error::Error;
use mongodb::options::{ClientOptions, FindOneOptions, UpdateModifications};

use crate::{LogExtensionErr, LogExtensionOk};
use crate::model::recipe::{Recipe, RecipeFormatError};
use crate::pagination::Pagination;

const RECIPE_COLLECTION: &str = "recipes";
const URL: &str = "mongodb://localhost:26666";
const APP_NAME: &str = "Zellinotes recipes";
const DATABASE: &str = "zellinotes_recipes";

type ImageBase64String = String;

#[derive(Clone)]
pub struct Dao {
    pub database: Database
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DaoError {
    DatabaseError(String),
    DocumentNotFound,
    RecipeFormatError(String),
}


impl From<Error> for DaoError {
    fn from(error: Error) -> Self {
        DaoError::DatabaseError(format!("{:#?}", error))
    }
}

impl From<ValueAccessError> for DaoError {
    fn from(error: ValueAccessError) -> Self {
        DaoError::DatabaseError(format!("{:#?}", error))
    }
}

impl From<RecipeFormatError> for DaoError {
    fn from(error: RecipeFormatError) -> Self {
        DaoError::RecipeFormatError(format!("{:#?}", error.error))
    }
}

impl Dao {
    pub async fn new() -> Option<Self> {
        match get_db_handler().await
            .log_if_ok(|_| info!("Created database handler"))
            .log_if_err(|err| error!("Could not create database handler. Err={}", err))
            .ok() {
            Some(database) => Some(Self { database }),
            None => None
        }
    }

    /// add recipe as it is, but ignores id
    pub async fn add_one_recipe(&self, recipe: Recipe) -> Result<Bson, DaoError> {
        match self.database.collection(RECIPE_COLLECTION).insert_one(recipe.clone().into(), None).await {
            Ok(result) => {
                info!("Added recipe in db. id={:?}", result.inserted_id);
                Ok(result.inserted_id)
            }
            Err(err) => {
                error!("Could not add recipe={:#?}, Err={:#?}", recipe, err);
                Err(DaoError::from(err))
            }
        }
    }

    pub async fn update_one_recipe_ignore_image(&self, id: ObjectId, recipe: Recipe) -> Result<(), DaoError> {
        let query = object_id_into_doc(id.clone());

        let mut recipe = Document::from(recipe);
        recipe.remove("image");
        let update = UpdateModifications::Document(
            doc! { "$set" : recipe}
        );

        match self.database.collection(RECIPE_COLLECTION)
            .update_one(query, update, None).await {
            Ok(result) => match result.modified_count {
                0 => {
                    info!("Not Updated recipe, doc not found with id={:#?}", &id);
                    Err(DaoError::DocumentNotFound)
                }
                _ => {
                    info!("Updated recipe in db with id={:#?}", &id);
                    Ok(())
                }
            }
            Err(err) => {
                error!("Could not update recipe with id={:#?}, Err={:#?}", &id, err);
                Err(DaoError::from(err))
            }
        }
    }

    pub async fn add_many_recipes(&self, recipes: Vec<Recipe>) -> Result<Bson, DaoError> {
        match self.database.collection(RECIPE_COLLECTION).insert_many(
            recipes.clone().into_iter().map(|r| r.into()).collect::<Vec<Document>>(), None).await {
            Ok(result) => {
                info!("Added multiple recipes in db. ids={:#?}", result.inserted_ids);
                Ok(Bson::from(result.inserted_ids.values().map(|b: &Bson| b.to_owned()).collect::<Vec<Bson>>()))
            }
            Err(err) => {
                error!("Could not add multiple recipes={:#?}, Err={:#?}", recipes, err);
                Err(DaoError::from(err))
            }
        }
    }

    pub async fn get_one_recipe_without_image(&self, id: ObjectId) -> Result<Recipe, DaoError> {
        let filter = object_id_into_doc(id.clone());

        let options = Dao::recipe_only_image_find_options();

        let result = self.database
            .collection(RECIPE_COLLECTION)
            .find_one(filter, options).await
            .map_err(|err| DaoError::from(err))?
            .map(|doc| Recipe::try_from(doc));

        match result {
            Some(Ok(recipe)) => {
                info!("Got one recipe from db. id={:#?}", id.clone());
                Ok(recipe)
            }
            Some(Err(error)) => {
                error!("Got one recipe, but could not format id={:#?}, error={:#?}", id.clone(), error);
                Err(DaoError::RecipeFormatError(id.clone().to_hex()))
            }
            None => {
                error!("get recipe without image, recipe Not found: id={:#?}", id);
                Err(DaoError::DocumentNotFound)
            }
        }
    }

    pub async fn get_one_recipe_image(&self, id: ObjectId) -> Result<ImageBase64String, DaoError> {
        let filter = object_id_into_doc(id.clone());

        let options = Dao::recipe_without_image_find_options();

        let image: Option<Document> = self.database
            .collection(RECIPE_COLLECTION)
            .find_one(filter, options)
            .await
            .map_err(|err| DaoError::from(err))?;

        match image {
            Some(image) => {
                match image.get_str("image") {
                    Ok(image) => {
                        info!("Got one recipe from db. id={:?}", id.clone());
                        Ok(image.to_string())
                    }
                    Err(_) => {
                        error!("Image not found, or not string id={:#?}", id.clone());
                        Err(DaoError::DocumentNotFound)
                    }
                }
            }
            None => {
                error!("Image not found id={:#?}", id);
                Err(DaoError::DocumentNotFound)
            }
        }
    }

    pub async fn update_one_recipe_image(&self, id: ObjectId, image: Option<String>) -> Result<(), DaoError> {
        let query = object_id_into_doc(id.clone());

        let update = match image {
            Some(image) => UpdateModifications::Document(
                doc! { "$set" : { "image" : image} }
            ),
            None => UpdateModifications::Document(
                doc! { "$set" : { "image" : Bson::Null} }
            )
        };

        match self.database.collection(RECIPE_COLLECTION)
            .update_one(query, update, None).await {
            Ok(result) => match result.modified_count {
                0 => {
                    info!("Not Updated image, doc not found with id={:#?}", &id);
                    Err(DaoError::DocumentNotFound)
                }
                _ => {
                    info!("Updated recipe image in db with id={:#?}", &id);
                    Ok(())
                }
            }
            Err(err) => {
                error!("Could not update recipe image with id={:#?}, Err={:#?}", &id, err);
                Err(DaoError::from(err))
            }
        }
    }


    fn recipe_without_image_find_options() -> Option<FindOneOptions> {
        let mut options = FindOneOptions::default();
        options.projection = Some(db_projection_only_image());
        let options = Some(options);
        options
    }

    fn recipe_only_image_find_options() -> Option<FindOneOptions> {
        let mut options = FindOneOptions::default();
        options.projection = Some(Recipe::default_projection_no_image());
        let options = Some(options);
        options
    }

    pub async fn delete_one_recipe(&self, id: ObjectId) -> Result<(), DaoError> {
        let query = object_id_into_doc(id.clone());

        match self.database.collection(RECIPE_COLLECTION).delete_one(query, None).await {
            Ok(delete_result) => match delete_result.deleted_count {
                1 => {
                    info!("Deleted one recipe from db. id={:#?}", &id);
                    Ok(())
                }
                _ => {
                    error!("Deleted no recipe from db. id={:#?}", &id);
                    Err(DaoError::DocumentNotFound)
                }
            }
            Err(err) => {
                error!("Error receiving recipe from db. id={:#?}, err={:#?}", &id, &err);
                Err(DaoError::from(err))
            }
        }
    }

    pub async fn get_many_recipes(&self, pagination: Option<Pagination>) -> Result<Vec<Recipe>, DaoError> {
        get_many_recipes(&self.database, pagination).await
            .log_if_ok(|recipes| info!("Get many recipes from db. ids={:#?}", recipes))
            .log_if_err(|err| error!("{:#?}", err))
    }
}

fn object_id_into_doc(id: ObjectId) -> Document {
    doc! {"_id": Bson::ObjectId(id)}
}

fn db_projection_only_image() -> Document {
    doc! {"image": 1, "_id": 0}
}

async fn get_db_handler() -> Result<Database, Error> {
    let mut client_options = ClientOptions::parse(URL).await?;
    client_options.app_name = Some(APP_NAME.to_string());
    let client = Client::with_options(client_options)?;
    return Ok(client.database(DATABASE));
}


pub async fn get_many_recipes(db: &Database, pagination: Option<Pagination>) -> Result<Vec<Recipe>, DaoError> {
    let mut find_options = FindOptions::default();
    let mut skip = 0;
    let mut take = usize::MAX;
    if pagination.is_some() {
        skip = (pagination.unwrap().page.unwrap() - 1) * pagination.unwrap().items.unwrap();
        take = pagination.unwrap().items.unwrap();
        find_options.sort = Some(doc! { "created": Bson::Int32(pagination.unwrap().sorting.unwrap() as i32) });
        find_options.projection = Some(Recipe::default_projection_no_image());
    }

    match db.collection(RECIPE_COLLECTION).find(None, find_options).await {
        Ok(cursor) => {
            let recipes = cursor
                .skip(skip)
                .take(take)
                .collect::<Vec<Result<Document, Error>>>()
                .await
                .into_iter()
                .collect::<Result<Vec<Document>, Error>>()
                .map_err(|err| {
                    DaoError::DatabaseError(format!("{:#?}", err))
                })?;

            let recipes = recipes
                .into_iter()
                .map(|recipe| Recipe::try_from(recipe))
                .collect::<Result<Vec<Recipe>, RecipeFormatError>>()
                .map_err(|err| {
                    DaoError::DatabaseError(format!("{:#?}", err))
                })?;

            Ok(recipes)
        }
        Err(err) => Err(DaoError::DatabaseError(format!("{:#?}", err)))
    }
}


#[cfg(test)]
pub mod dao_tests {
    use bson::Bson;
    use bson::oid::ObjectId;
    use chrono::{Duration, Timelike};
    use chrono::Utc;
    use log::LevelFilter;
    use mongodb::{Client, Database};
    use mongodb::error::Error;
    use mongodb::options::ClientOptions;
    use serial_test::serial;
    use simplelog::{Config, TerminalMode, TermLogger};

    use crate::dao::{Dao, DaoError};
    use crate::model::difficulty::Difficulty;
    use crate::model::recipe::Recipe;
    use crate::pagination::Pagination;

    const TEST_URL: &str = "mongodb://localhost:26666";
    const TEST_APP_NAME: &str = "Zellinotes development recipes";
    const TEST_DATABASE: &str = "test_zellinotes_development_recipes";

    pub fn create_one_recipe_without_image() -> Recipe {
        Recipe {
            _id: ObjectId::new(),
            cooking_time_in_minutes: 10,
            created: Utc::now().with_nanosecond(0).unwrap(),
            last_modified: Utc::now().with_nanosecond(0).unwrap(),
            ingredients: vec![],
            version: 1,
            difficulty: Difficulty::Easy,
            description: "".to_string(),
            title: "".to_string(),
            tags: vec![],
            image_base64: None,
            instructions: vec![],
            default_servings: 1,
        }
    }

    pub fn create_one_recipe_with_image() -> Recipe {
        let mut recipe = create_one_recipe_without_image();
        recipe.image_base64 = Some("image".to_string());
        recipe
    }

    pub fn create_many_recipes_without_images(amount: i32) -> Vec<Recipe> {
        (0..amount).into_iter().map(|i| {
            let mut x = create_one_recipe_without_image();
            x.title = i.to_string();
            x.created = Utc::now().with_nanosecond(0).unwrap() + Duration::days(1);
            x
        }).collect()
    }

    async fn init_test_database() -> Result<Database, Error> {
        let mut client_options = ClientOptions::parse(TEST_URL).await?;
        client_options.app_name = Some(TEST_APP_NAME.to_string());
        let client = Client::with_options(client_options)?;
        let db = client.database(TEST_DATABASE);
        return Ok(db);
    }

    pub async fn before() -> Dao {
        init_test_logger();
        let dao = Dao { database: init_test_database().await.unwrap() };
        cleanup_after(dao).await;
        Dao { database: init_test_database().await.unwrap() }
    }

    fn init_test_logger() {
        let _ = TermLogger::init(LevelFilter::Info,
                                 Config::default(),
                                 TerminalMode::Mixed).unwrap_or_else(|_| ());
    }

    pub async fn cleanup_after(dao: Dao) {
        dao.database.drop(None).await.unwrap();
    }

    #[actix_rt::test]
    #[serial]
    async fn add_one_recipe_without_image_test() {
        let dao = before().await;
        let recipe = create_one_recipe_without_image();

        let result = dao.add_one_recipe(recipe).await;
        assert!(result.is_ok());
        result.unwrap().as_object_id().unwrap().timestamp().date();

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn add_one_recipe_with_image_test() {
        let dao = before().await;
        let recipe = create_one_recipe_with_image();

        let result = dao.add_one_recipe(recipe).await;
        assert!(result.is_ok());
        result.unwrap().as_object_id().unwrap().timestamp().date();

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn update_one_recipe_ignore_image_test() {
        let dao = before().await;
        let mut recipe = create_one_recipe_with_image();

        let result = dao.add_one_recipe(recipe.clone()).await.unwrap();
        let recipe_id = result.as_object_id().unwrap().to_owned();
        recipe.title = "new".to_string();
        recipe.image_base64 = Some("new_image".to_string());

        let result = dao.update_one_recipe_ignore_image(recipe_id.clone(), recipe.clone()).await;
        assert!(result.is_ok());

        let result = dao.get_one_recipe_without_image(recipe_id.clone()).await;
        assert_eq!(result.clone().unwrap().title, "new".to_string());
        assert_eq!(result.unwrap().image_base64, None);

        let result = dao.get_one_recipe_image(recipe_id).await;
        assert_eq!(result.unwrap().as_str(), "image");

        let result = dao.update_one_recipe_ignore_image(ObjectId::new(), recipe.clone()).await;
        assert_eq!(result.err().unwrap(), DaoError::DocumentNotFound);

        cleanup_after(dao).await;
    }


    #[actix_rt::test]
    #[serial]
    async fn update_one_recipe_image_test() {
        let dao = before().await;
        let recipe = create_one_recipe_with_image();

        let result = dao.add_one_recipe(recipe.clone()).await.unwrap();
        let recipe_id = result.as_object_id().unwrap().to_owned();
        let result = dao.update_one_recipe_image(recipe_id.clone(), Some("new_image".to_string())).await;
        assert!(result.is_ok());

        let result = dao.get_one_recipe_image(recipe_id.clone()).await;
        assert_eq!(result.unwrap(), "new_image".to_string());

        let result = dao.update_one_recipe_image(recipe_id.clone(),None).await;
        assert!(result.is_ok());

        let result = dao.get_one_recipe_image(recipe_id.clone()).await;
        assert_eq!(result.err().unwrap(), DaoError::DocumentNotFound);

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn add_many_recipes_test() {
        let dao = before().await;
        let recipes = create_many_recipes_without_images(50);
        let inserted_ids: Vec<ObjectId> = recipes.clone().into_iter().map(|recipe| recipe._id).collect();

        let result = dao.add_many_recipes(recipes).await;
        assert!(result.is_ok());

        let added_recipes: Vec<Bson> = result.unwrap().as_array().unwrap().to_owned();
        let added_recipes: Vec<ObjectId> = added_recipes.into_iter().map(|e| e.as_object_id().unwrap().to_owned()).collect();
        assert_eq!(inserted_ids.len(), added_recipes.len()); //todo compare correct

        cleanup_after(dao).await;
    }


    #[actix_rt::test]
    #[serial]
    async fn get_one_recipe_without_image() {
        let dao = before().await;
        let recipe = create_one_recipe_with_image();
        let result = dao.add_one_recipe(recipe).await.unwrap();

        let inserted_oid = result.as_object_id().unwrap().to_owned();
        let recipe_found = dao
            .get_one_recipe_without_image(inserted_oid.clone())
            .await;

        assert_eq!(recipe_found.is_ok(), true);
        assert_eq!(recipe_found.clone().unwrap()._id, inserted_oid);
        assert_eq!(recipe_found.unwrap().image_base64, None);

        let doc_with_wrong_id_not_found = dao.get_one_recipe_without_image(ObjectId::new()).await;
        assert_eq!(doc_with_wrong_id_not_found.err().unwrap(), DaoError::DocumentNotFound);

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn get_one_recipe_image() {
        let dao = before().await;
        let recipe = create_one_recipe_with_image();
        let result = dao.add_one_recipe(recipe).await.unwrap();

        let inserted_oid = result.as_object_id().unwrap().to_owned();
        let image = dao
            .get_one_recipe_image(inserted_oid.clone())
            .await;

        assert_eq!(image.is_ok(), true);
        assert_eq!(image.unwrap(), "image");

        let doc_with_wrong_id_not_found = dao.get_one_recipe_without_image(ObjectId::new()).await;
        assert_eq!(doc_with_wrong_id_not_found.err().unwrap(), DaoError::DocumentNotFound);

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn delete_one_recipe_test() {
        let dao = before().await;
        let recipe = create_one_recipe_without_image();
        let result = dao.add_one_recipe(recipe.clone()).await.unwrap();
        let recipe_id = result.as_object_id().unwrap().to_owned();

        let result = dao.delete_one_recipe(recipe_id.clone()).await;
        assert!(result.is_ok());

        let result = dao.delete_one_recipe(ObjectId::new()).await;
        assert_eq!(result.err().unwrap(), DaoError::DocumentNotFound);

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn get_all_recipes() {
        let dao = before().await;
        let recipes = create_many_recipes_without_images(50);
        let amount_of_recipes = recipes.clone().len();
        let result = dao.add_many_recipes(recipes.clone()).await;

        assert!(result.clone().is_ok());
        assert_eq!(result.clone().unwrap().as_array().unwrap().len(), recipes.clone().len());

        let read_recipes = dao.get_many_recipes(None).await.unwrap();
        assert_eq!(amount_of_recipes, read_recipes.len());

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn get_paged_recipes_1() -> Result<(), ()> {
        let dao = before().await;
        get_paged_recipes_test(&dao, create_many_recipes_without_images(20), 1, 5, 1).await;
        cleanup_after(dao).await;
        Ok(())
    }

    #[actix_rt::test]
    #[serial]
    async fn get_paged_recipes_2() -> Result<(), ()> {
        let dao = before().await;
        get_paged_recipes_test(&dao, create_many_recipes_without_images(20), 2, 5, 1).await;
        cleanup_after(dao).await;
        Ok(())
    }

    #[actix_rt::test]
    #[serial]
    async fn get_paged_recipes_3() -> Result<(), ()> {
        let dao = before().await;
        get_paged_recipes_test(&dao, create_many_recipes_without_images(20), 3, 5, 1).await;
        cleanup_after(dao).await;
        Ok(())
    }

    #[actix_rt::test]
    #[serial]
    async fn get_paged_recipes_4() -> Result<(), ()> {
        let dao = before().await;
        get_paged_recipes_test(&dao, create_many_recipes_without_images(20), 2, 20, 1).await;
        cleanup_after(dao).await;
        Ok(())
    }


    async fn get_paged_recipes_test(dao: &Dao, mut recipes_to_insert: Vec<Recipe>, page: usize, items: usize, sorting: i32) {
        let result = dao.add_many_recipes(recipes_to_insert.clone()).await;
        assert!(result.clone().is_ok());
        assert_eq!(result.clone().unwrap().as_array().unwrap().len(), recipes_to_insert.clone().len());

        let read_recipes = dao.get_many_recipes(Some(Pagination {
            page: Some(page),
            items: Some(items),
            sorting: Some(sorting),
        })).await.unwrap();
        let read_recipes: Vec<Recipe> = read_recipes.into_iter().map(|mut r| {
            r._id = ObjectId::with_bytes([0; 12]);
            r
        }).collect();

        recipes_to_insert.sort_by(|l, r| l.created.cmp(&r.created));

        let recipes_to_insert: Vec<Recipe> = recipes_to_insert
            .into_iter()
            .map(|mut recipe| {
                recipe._id = ObjectId::with_bytes([0; 12]);
                recipe.image_base64 = None;
                recipe
            })
            .skip((page - 1) * items)
            .take(items)
            .collect();

        assert_eq!(read_recipes, recipes_to_insert);
        println!("{:#?}", read_recipes);
    }
}
