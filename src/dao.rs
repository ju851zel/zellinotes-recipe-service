use std::convert::TryFrom;

use bson::Document;
use bson::oid::ObjectId;
use futures_util::StreamExt;
use mongodb::{bson::{Bson, doc}, Client, options::FindOptions};
use mongodb::Database;
use mongodb::error::Error;
use mongodb::options::ClientOptions;

use crate::model::recipe::{Recipe, RecipeFormatError};
use crate::pagination::Pagination;
use crate::{LogExtensionOk, LogExtensionErr};

const RECIPE_COLLECTION: &str = "recipes";
const URL: &str = "mongodb://localhost:26666";
const APP_NAME: &str = "Zellinotes recipes";
const DATABASE: &str = "zellinotes_recipes";



type DaoError = String;

#[derive(Clone)]
pub struct Dao {
    pub(crate) database: Database
}

impl Dao {
    pub async fn new() -> Result<Database, Error> {
        get_db_handler().await
            .log_if_ok(|_| info!("Created database handler"))
            .log_if_err(|err| error!("Could not create database handler. Err={}", err))
    }

    pub async fn update_one_recipe(&self, id: String, recipe: Recipe) -> Result<Option<()>, String> {
        update_one_recipe(&self.database, id.clone(), recipe).await
            .log_if_ok(|_| info!("Updated recipe in db with id={:#?}", &id))
            .log_if_err(|err| error!("Could not update recipe with id={:#?}, Err={:#?}", &id, err))
    }
}

fn id_to_object_id(id: String) -> Result<ObjectId, DaoError> {
    ObjectId::with_string(&id)
        .map_err(|err| format!("Could not create object id from provided id"))
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


async fn update_one_recipe(db: &Database, id: String, recipe: Recipe) -> Result<Option<()>, DaoError> {
    let object_id = id_to_object_id(id.clone())?;
    let query = object_id_into_doc(object_id);
    let recipe = Document::from(recipe);

    return db.collection(RECIPE_COLLECTION)
        .update_one(query, recipe, None).await
        .map_err(|err| format!("Could not update recipe with id={:#?}, Err={:#?}", id, err))
        .map(|result| match result.modified_count {
            0 =>None,
            _ => Some(()),
        })

}

/// ignores recipe Id
pub async fn db_add_one_recipe(db: &Database, recipe: Recipe) -> Result<Bson, String> {
    return match db.collection(RECIPE_COLLECTION)
        .insert_one(recipe.clone().into(), None).await {
        Ok(result) => Ok(result.inserted_id),
        Err(err) => Err(format!("Error inserting recipe:{:?}. Err: {:?}", recipe, err)),
    };
}

pub async fn db_add_many_recipes(db: &Database, recipes: Vec<Recipe>) -> Result<Bson, String> {
    return match db.collection(RECIPE_COLLECTION).insert_many(
        recipes.clone()
            .into_iter()
            .map(|r| r.into())
            .collect::<Vec<Document>>(), None).await {
        Ok(result) => Ok(Bson::from(result.inserted_ids.values().map(|b: &Bson| b.to_owned()).collect::<Vec<Bson>>())),
        Err(err) => Err(format!("Error inserting many recipes:{:?}. Err: {:?}", recipes, err)),
    };
}


pub async fn db_get_one_recipe(db: &Database, id_as_string: String) -> Result<Option<Recipe>, String> {
    let id = match ObjectId::with_string(&id_as_string) {
        Ok(id) => id,
        _ => return Ok(None),
    };

    let filter = Some(doc! { "_id": id});
    return match db.collection(RECIPE_COLLECTION).find_one(filter, None).await {
        Ok(optional_doc) => match optional_doc {
            Some(document) => Recipe::try_from(document).map(|r| Some(r)).map_err(|e| e.error),
            None => Ok(None),
        },
        Err(_) => Err(format!("Could not get document with id={} in db", id_as_string))
    };
}


pub async fn db_get_all_recipes(db: &Database, pagination: Option<Pagination>) -> Result<Vec<Recipe>, String> {
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
    use mongodb::{Client, Database};
    use mongodb::error::Error;
    use mongodb::options::ClientOptions;

    use crate::{dao, init_logger};
    use crate::model::difficulty::Difficulty;
    use crate::model::recipe::Recipe;
    use crate::pagination::Pagination;

    const TEST_URL: &str = "mongodb://localhost:26666";
    const TEST_APP_NAME: &str = "Zellinotes development recipes";
    const TEST_DATABASE: &str = "test_zellinotes_development_recipes";

    pub fn create_recipe() -> Recipe {
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
            let mut x = create_recipe();
            x.title = i.to_string();
            x.created = Utc::now().with_nanosecond(0).unwrap() + Duration::days(1);
            x
        }).collect()
    }

    pub async fn init_test_database() -> Result<Database, Error> {
        let mut client_options = ClientOptions::parse(TEST_URL).await?;
        client_options.app_name = Some(TEST_APP_NAME.to_string());
        let client = Client::with_options(client_options)?;
        let db = client.database(TEST_DATABASE);
        clean_up(db).await;
        let db = client.database(TEST_DATABASE);
        return Ok(db);
    }

    pub async fn clean_up(db: Database) {
        db.drop(None).await.unwrap();
    }

    #[actix_rt::test]
    async fn add_single_recipe_test() {
        let db = init_test_database().await.unwrap();
        let recipe = create_recipe();

        let result = dao::db_add_one_recipe(&db, recipe).await;
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap());

        clean_up(db).await;
    }

    #[actix_rt::test]
    async fn update_one_recipe_test() {
        init_logger();
        let db = init_test_database().await.unwrap();
        let mut recipe = create_recipe();

        let result = dao::db_add_one_recipe(&db, recipe.clone()).await;
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap());
        let recipe_id = result.unwrap().as_object_id().unwrap().to_string();
        println!("{}", recipe_id);
        recipe.title = "new".to_string();

        let result = dao::db_update_one_recipe(&db, recipe_id.clone(), recipe).await;
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap());

        clean_up(db).await;
    }

    #[actix_rt::test]
    async fn add_many_recipes_test() {
        let db = init_test_database().await.unwrap();
        let recipes = create_many_recipes(50);

        let result = dao::db_add_many_recipes(&db, recipes.clone()).await;
        assert_eq!(result.is_ok(), true, "{}", result.err().unwrap());
        let added_recipes: Vec<Bson> = result.unwrap().as_array().unwrap().to_owned();
        let added_recipes: Vec<ObjectId> = added_recipes.into_iter().map(|e| e.as_object_id().unwrap().to_owned()).collect();
        let len_before = recipes.len();
        let len_after = added_recipes.len();
        assert_eq!(len_after, len_before, "recipes-added={}, recipes-read={}", len_before, len_after);

        clean_up(db).await;
    }

    #[actix_rt::test]
    async fn get_all_recipes() {
        let db = init_test_database().await.unwrap();
        let recipes = create_many_recipes(50);

        let result = dao::db_add_many_recipes(&db, recipes.clone()).await;
        assert_eq!(result.clone().is_ok(), true, "{}", result.err().unwrap());
        assert_eq!(result.clone().unwrap().as_array().unwrap().len(), recipes.clone().len(), "{}", result.err().unwrap());

        let read_recipes = dao::db_get_all_recipes(&db, None).await.unwrap();
        assert_eq!(recipes.len(), read_recipes.len(), "recipes-wanted-to-add={}, recipes-read-after-add={}", recipes.len(), read_recipes.len());

        clean_up(db).await;
    }

    #[actix_rt::test]
    async fn get_paged_recipes() -> Result<(), ()> {
        let db = init_test_database().await.unwrap();

        get_paged_recipes_test(&db, create_many_recipes(20), 1, 10, 1).await;
        get_paged_recipes_test(&db, create_many_recipes(20), 2, 5, 1).await;
        get_paged_recipes_test(&db, create_many_recipes(20), 3, 5, 1).await;
        get_paged_recipes_test(&db, create_many_recipes(20), 2, 20, 1).await;

        clean_up(db).await;
        Ok(())
    }

    #[actix_rt::test]
    async fn get_one_recipes() {
        let db = init_test_database().await.unwrap();
        let recipe = create_recipe();

        let result = dao::db_add_one_recipe(&db, recipe.clone()).await.unwrap();
        let inserted_oid = result.as_object_id().unwrap().to_string();


        let doc_with_wrong_id_not_found = dao::db_get_one_recipe(&db, "not an object id".to_string())
            .await.unwrap().is_none();
        assert_eq!(doc_with_wrong_id_not_found, true);

        let recipe_found = dao::db_get_one_recipe(&db, inserted_oid.clone())
            .await.unwrap();
        assert_eq!(recipe_found.is_some(), true);
        assert_eq!(recipe_found.unwrap()._id, inserted_oid);

        clean_up(db).await;
    }


    async fn get_paged_recipes_test(db: &Database, mut recipes_to_insert: Vec<Recipe>, page: usize, items: usize, sorting: i32) {
        let result = dao::db_add_many_recipes(&db, recipes_to_insert.clone()).await;
        assert_eq!(result.clone().is_ok(), true, "{}", result.err().unwrap());
        assert_eq!(result.clone().unwrap().as_array().unwrap().len(), recipes_to_insert.clone().len(), "{}", result.clone().err().unwrap());

        let read_recipes = dao::db_get_all_recipes(&db, Some(Pagination {
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
