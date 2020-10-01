use std::convert::TryFrom;
use std::sync::Arc;

use bson::Document;
use bson::oid::ObjectId;
use futures_util::StreamExt;
use mongodb::{bson::{Bson, doc}, Client, options::FindOptions};
use mongodb::Database;
use mongodb::error::Error;
use mongodb::options::ClientOptions;

use crate::{LogExtensionErr, LogExtensionOk};
use crate::model::recipe::{Recipe, RecipeFormatError};
use crate::pagination::Pagination;
use mongodb::results::DeleteResult;

const RECIPE_COLLECTION: &str = "recipes";
const URL: &str = "mongodb://localhost:26666";
const APP_NAME: &str = "Zellinotes recipes";
const DATABASE: &str = "zellinotes_recipes";


type DaoError = String;

#[derive(Clone)]
pub struct Dao {
    pub database: Database
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

    pub async fn update_one_recipe(&self, id: String, recipe: Recipe) -> Option<Option<()>> {
        update_one_recipe(&self.database, id.clone(), recipe).await
            .log_if_ok(|_| info!("Updated recipe in db with id={:#?}", &id))
            .log_if_err(|err| error!("Could not update recipe with id={:#?}, Err={:#?}", &id, err))
            .ok()
    }

    pub async fn add_one_recipe(&self, recipe: Recipe) -> Option<Bson> {
        add_one_recipe(&self.database, recipe.clone()).await
            .log_if_ok(|id| info!("Added recipe in db. id={:?}", id))
            .log_if_err(|err| error!("Could not add recipe={:#?}, Err={:#?}", recipe, err))
            .ok()
    }

    pub async fn add_many_recipes(&self, recipes: Vec<Recipe>) -> Option<Bson> {
        add_many_recipes(&self.database, recipes.clone()).await
            .log_if_ok(|id| info!("Added multiple recipes in db. ids={:#?}", id))
            .log_if_err(|err| error!("Could not add multiple recipes={:#?}, Err={:#?}", recipes, err))
            .ok()
    }

    pub async fn get_one_recipe(&self, id: String) -> Option<Option<Recipe>> {
        get_one_recipe(&self.database, id.clone()).await
            .log_if_ok(|id| info!("Got one recipe from db. id={:#?}", id))
            .log_if_err(|err| error!("{} id={:#?}", err, id))
            .ok()
    }

    pub async fn delete_one_recipe(&self, id: String) -> Option<Option<()>> {
        delete_one_recipe(&self.database, id.clone()).await
            .log_if_ok(|_| info!("Deleted one recipe from db. id={:#?}", id))
            .log_if_err(|err| error!("{} id={:#?}", err, id))
            .ok()
    }

    pub async fn get_many_recipes(&self, pagination: Option<Pagination>) -> Option<Vec<Recipe>> {
        get_many_recipes(&self.database, pagination).await
            .log_if_ok(|recipes| info!("Get many recipes from db. ids={:#?}", recipes))
            .log_if_err(|err| error!("{}", err))
            .ok()
    }
}

fn id_to_object_id(id: String) -> Result<ObjectId, String> {
    ObjectId::with_string(&id).map_err(|_| format!("Could not parse id={:#?} into Object Id", id))
}

fn object_id_into_doc(id: ObjectId) -> Document {
    doc! {"_id": Bson::ObjectId(id)}
}

async fn get_db_handler() -> Result<Database, Error> {
    let mut client_options = ClientOptions::parse(URL).await?;
    client_options.app_name = Some(APP_NAME.to_string());
    let client = Client::with_options(client_options)?;
    return Ok(client.database(DATABASE));
}

/// If worked returns Option if Updated or not
async fn update_one_recipe(db: &Database, id: String, recipe: Recipe) -> Result<Option<()>, String> {
    let object_id = id_to_object_id(id.clone())?;
    let query = object_id_into_doc(object_id);
    let recipe = Document::from(recipe);

    return db.collection(RECIPE_COLLECTION)
        .update_one(query, recipe, None).await
        .map_err(|e| format!("{:#?}", e.kind))
        .map(|result| match result.modified_count {
            0 => None,
            _ => Some(()),
        });
}

/// ignores recipe Id
async fn add_one_recipe(db: &Database, recipe: Recipe) -> Result<Bson, Error> {
    return match db.collection(RECIPE_COLLECTION)
        .insert_one(recipe.clone().into(), None).await {
        Ok(result) => Ok(result.inserted_id),
        Err(err) => Err(err),
    };
}

async fn add_many_recipes(db: &Database, recipes: Vec<Recipe>) -> Result<Bson, Error> {
    return match db.collection(RECIPE_COLLECTION).insert_many(
        recipes.clone()
            .into_iter()
            .map(|r| r.into())
            .collect::<Vec<Document>>(), None).await {
        Ok(result) => Ok(Bson::from(result.inserted_ids.values().map(|b: &Bson| b.to_owned()).collect::<Vec<Bson>>())),
        Err(err) => Err(err),
    };
}


pub async fn get_one_recipe(db: &Database, id: String) -> Result<Option<Recipe>, String> {
    let object_id = id_to_object_id(id.clone())
        .map_err(|e| format!("id={:#?} not parsable to object id", id.clone()))?;
    let query = object_id_into_doc(object_id);

    return match db.collection(RECIPE_COLLECTION).find_one(query, None).await {
        Ok(optional_doc) => match optional_doc {
            Some(document) => match Recipe::try_from(document) {
                Ok(recipe) => { Ok(Some(recipe)) }
                Err(err) => Err(err.error)
            }
            None => Ok(None),
        },
        Err(err) => Err(format!("{:#?}", err))
    };
}

pub async fn delete_one_recipe(db: &Database, id: String) -> Result<Option<()>, String> {
    let object_id = id_to_object_id(id.clone())
        .map_err(|e| format!("id={:#?} not parsable to object id", id.clone()))?;
    let query = object_id_into_doc(object_id);

    return match db.collection(RECIPE_COLLECTION).delete_one(query, None).await {
        Ok(delete_result) => {
            match delete_result.deleted_count {
                1 => Ok(Some(())),
                _ => Ok(None),
            }
        },
        Err(err) => Err(format!("{:#?}", err))
    };
}



pub async fn get_many_recipes(db: &Database, pagination: Option<Pagination>) -> Result<Vec<Recipe>, String> {
    let mut find_options = FindOptions::default();
    let mut skip = 0;
    let mut take = usize::MAX;
    if pagination.is_some() {
        skip = (pagination.unwrap().page.unwrap() - 1) * pagination.unwrap().items.unwrap();
        take = pagination.unwrap().items.unwrap();
        find_options.sort = Some(doc! { "created": Bson::Int32(pagination.unwrap().sorting.unwrap()as i32) });
    }

    return match db.collection(RECIPE_COLLECTION).find(None, find_options).await {
        Ok(cursor) => {
            let (correct_recipes, wrong_recipes): (Vec<_>, Vec<_>) =
                cursor
                    .skip(skip)
                    .take(take)
                    .collect::<Vec<Result<Document, Error>>>().await
                    .into_iter()
                    .partition(Result::is_ok);

            for doc in wrong_recipes {
                error!("Error reading recipe document from db: {:?}", doc.err().unwrap())
            }

            let (correct_recipes,
                broken_recipes) = docs_to_recipes(correct_recipes);

            for recipe in broken_recipes {
                error!("Error converting recipe document to recipe: {:?}", recipe.err().unwrap())
            }

            Ok(correct_recipes.into_iter().map(|r| r.unwrap()).collect())
        }
        Err(_) => Err(format!("Could not get all recipes from db"))
    };
}

fn docs_to_recipes(correct_recipes: Vec<Result<Document, Error>>) -> (Vec<Result<Recipe, RecipeFormatError>>, Vec<Result<Recipe, RecipeFormatError>>) {
    correct_recipes.into_iter()
        .map(|x| Recipe::try_from(x.unwrap()))
        .partition(Result::is_ok)
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
    use simplelog::{CombinedLogger, Config, TerminalMode, TermLogger};

    use crate::{dao, init_logger};
    use crate::dao::Dao;
    use crate::model::difficulty::Difficulty;
    use crate::model::recipe::Recipe;
    use crate::pagination::Pagination;

    const TEST_URL: &str = "mongodb://localhost:26666";
    const TEST_APP_NAME: &str = "Zellinotes development recipes";
    const TEST_DATABASE: &str = "test_zellinotes_development_recipes";

    pub fn create_one_recipe() -> Recipe {
        Recipe {
            _id: "".to_string(),
            cooking_time_in_minutes: 10,
            created: Utc::now().with_nanosecond(0).unwrap(),
            last_modified: Utc::now().with_nanosecond(0).unwrap(),
            ingredients: vec![],
            version: 1,
            difficulty: Difficulty::Easy,
            description: "".to_string(),
            title: "".to_string(),
            tags: vec![],
            image: None,
            instructions: vec![],
            default_servings: 1,
        }
    }

    pub fn create_many_recipes(amount: i32) -> Vec<Recipe> {
        (0..amount).into_iter().map(|i| {
            let mut x = create_one_recipe();
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
                                 TerminalMode::Mixed).unwrap_or_else(|e| ());
    }

    pub async fn cleanup_after(dao: Dao) {
        dao.database.drop(None).await.unwrap();
    }

    #[actix_rt::test]
    #[serial]
    async fn add_single_recipe_test() {
        let dao = before().await;
        let recipe = create_one_recipe();

        let result = dao.add_one_recipe(recipe).await;
        assert_eq!(result.is_some(), true);
        result.unwrap().as_object_id().unwrap().timestamp().date();

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn update_one_recipe_test() {
        let dao = before().await;
        let mut recipe = create_one_recipe();
        let result = dao.add_one_recipe(recipe.clone()).await.unwrap();
        let recipe_id = result.as_object_id().unwrap().to_string();
        recipe.title = "new".to_string();

        let result = dao.update_one_recipe(recipe_id.clone(), recipe.clone()).await;
        assert_eq!(result.unwrap().is_some(), true);

        let result = dao.update_one_recipe("5f7d1be300f9ff0e0049f573".to_string(), recipe.clone()).await;
        assert_eq!(result.unwrap().is_none(), true);

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn delete_one_recipe_test() {
        let dao = before().await;
        let mut recipe = create_one_recipe();
        let result = dao.add_one_recipe(recipe.clone()).await.unwrap();
        let recipe_id = result.as_object_id().unwrap().to_string();

        let result = dao.delete_one_recipe(recipe_id.clone()).await;
        assert_eq!(result.unwrap().is_some(), true);

        let result = dao.delete_one_recipe("hello".to_string()).await;
        assert_eq!(result.is_none(), true);

        let result = dao.delete_one_recipe("5f7d1be300f9ff0e0049f573".to_string()).await;
        assert_eq!(result.unwrap().is_some(), true);

        cleanup_after(dao).await;
    }


    #[actix_rt::test]
    #[serial]
    async fn add_many_recipes_test() {
        let dao = before().await;
        let recipes = create_many_recipes(50);
        let amount_of_recipes = recipes.clone().len();

        let result = dao.add_many_recipes(recipes).await;
        assert_eq!(result.is_some(), true);

        let added_recipes: Vec<Bson> = result.unwrap().as_array().unwrap().to_owned();
        let added_recipes: Vec<ObjectId> = added_recipes.into_iter().map(|e| e.as_object_id().unwrap().to_owned()).collect();
        assert_eq!(amount_of_recipes, added_recipes.len(), "recipes-added={}, recipes-read={}", amount_of_recipes, added_recipes.len());

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn get_all_recipes() {
        let dao = before().await;
        let recipes = create_many_recipes(50);
        let amount_of_recipes = recipes.clone().len();
        let result = dao.add_many_recipes(recipes.clone()).await;

        assert_eq!(result.clone().is_some(), true);
        assert_eq!(result.clone().unwrap().as_array().unwrap().len(), recipes.clone().len());

        let read_recipes = dao.get_many_recipes(None).await.unwrap();
        assert_eq!(amount_of_recipes, read_recipes.len());

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn get_paged_recipes_1() -> Result<(), ()> {
        let dao = before().await;
        get_paged_recipes_test(&dao, create_many_recipes(20), 1, 5, 1).await;
        cleanup_after(dao).await;
        Ok(())
    }

    #[actix_rt::test]
    #[serial]
    async fn get_paged_recipes_2() -> Result<(), ()> {
        let dao = before().await;
        get_paged_recipes_test(&dao, create_many_recipes(20), 2, 5, 1).await;
        cleanup_after(dao).await;
        Ok(())
    }

    #[actix_rt::test]
    #[serial]
    async fn get_paged_recipes_3() -> Result<(), ()> {
        let dao = before().await;
        get_paged_recipes_test(&dao, create_many_recipes(20), 3, 5, 1).await;
        cleanup_after(dao).await;
        Ok(())
    }

    #[actix_rt::test]
    #[serial]
    async fn get_paged_recipes_4() -> Result<(), ()> {
        let dao = before().await;
        get_paged_recipes_test(&dao, create_many_recipes(20), 2, 20, 1).await;
        cleanup_after(dao).await;
        Ok(())
    }

    #[actix_rt::test]
    #[serial]
    async fn get_one_recipes() {
        let dao = before().await;
        let recipe = create_one_recipe();
        let result = dao.add_one_recipe(recipe).await.unwrap();

        let inserted_oid = result.as_object_id().unwrap().to_string();

        let doc_with_wrong_id_not_found = dao.get_one_recipe("5f73167e00d1c93600f9bf73".to_string())
            .await.unwrap().is_none();
        assert_eq!(doc_with_wrong_id_not_found, true);

        let recipe_found = dao.get_one_recipe(inserted_oid.clone())
            .await.unwrap();
        assert_eq!(recipe_found.is_some(), true);
        assert_eq!(recipe_found.unwrap()._id, inserted_oid);

        cleanup_after(dao).await;
    }


    async fn get_paged_recipes_test(dao: &Dao, mut recipes_to_insert: Vec<Recipe>, page: usize, items: usize, sorting: i32) {
        let result = dao.add_many_recipes(recipes_to_insert.clone()).await;
        assert_eq!(result.clone().is_some(), true);
        assert_eq!(result.clone().unwrap().as_array().unwrap().len(), recipes_to_insert.clone().len());

        let read_recipes = dao.get_many_recipes(Some(Pagination {
            page: Some(page),
            items: Some(items),
            sorting: Some(sorting),
        })).await.unwrap();
        let read_recipes: Vec<Recipe> = read_recipes.into_iter().map(|mut r| {
            r._id = "".to_string();
            r
        }).collect();

        recipes_to_insert.sort_by(|l, r| l.created.cmp(&r.created));

        let recipes_to_insert: Vec<Recipe> = recipes_to_insert.into_iter().skip((page - 1) * items).take(items).collect();

        assert_eq!(read_recipes, recipes_to_insert);
        println!("{:#?}", read_recipes);
    }
}
