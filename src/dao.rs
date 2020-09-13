use std::convert::TryFrom;

use bson::Document;
use futures_util::StreamExt;
use mongodb::{bson::{Bson, doc}, options::FindOptions, Client};
use mongodb::Database;
use mongodb::error::Error;
use mongodb::options::ClientOptions;

use crate::pagination::Pagination;
use crate::model::recipe::Recipe;

const RECIPE_COLLECTION: &str = "recipes";
const URL: &str = "mongodb://localhost:26666";
const APP_NAME: &str = "Zellinotes recipes";
const DATABASE: &str = "zellinotes_recipes";

pub async fn init_database() -> Result<Database, Error> {
    let mut client_options = ClientOptions::parse(URL).await?;
    client_options.app_name = Some(APP_NAME.to_string());
    let client = Client::with_options(client_options)?;
    return Ok(client.database(DATABASE));
}

pub async fn db_get_all_recipes(db: &Database) -> Vec<Recipe> {
    return match db.collection(RECIPE_COLLECTION).find(None, None).await {
        Ok(cursor) => {
            let (correct_recipes, wrong_recipes): (Vec<_>, Vec<_>) = cursor
                .collect::<Vec<Result<Document, Error>>>().await.into_iter()
                .partition(Result::is_ok);

            for doc in wrong_recipes {
                println!("Wrong recipe document in db: {:?}", doc.err().unwrap())
            }

            let (correct_recipes, broken_recipes): (Vec<_>, Vec<_>) =
                correct_recipes.into_iter()
                    .map(|x| Recipe::try_from(x.unwrap()))
                    .partition(Result::is_ok);

            for recipe in broken_recipes {
                println!("Wrong recipe document in db: {:?}", recipe.err().unwrap())
            }

            let co: Vec<Recipe> = correct_recipes.into_iter().map(|r| r.unwrap()).collect();

            co
        }
        Err(_) => Vec::new()
    };
}


pub async fn db_add_recipe(db: &Database, recipe: Recipe) -> Bson {
    return match db.collection(RECIPE_COLLECTION)
        .insert_one(recipe.into(), None).await {
        Ok(result) => result.inserted_id,
        Err(err) => {
            println!("{:?}", err);
            Bson::Null
        }
    };
}

/// panics if pagination not fully set
pub async fn db_get_paged_recipes(db: &Database, pagination: Pagination) -> Vec<Recipe> {
    let mut find_options = FindOptions::default();
    find_options.sort = Some(doc! { "created": 1 });

    match db.collection(RECIPE_COLLECTION).find(None, find_options).await {
        Ok(cursor) => {
            let (correct_recipes, wrong_recipes): (Vec<_>, Vec<_>) = cursor
                .skip(pagination.page.unwrap() * pagination.items.unwrap()).take(pagination.items.unwrap())
                .collect::<Vec<Result<Document, Error>>>().await.into_iter()
                .partition(Result::is_ok);

            for doc in wrong_recipes {
                println!("Wrong recipe document in db: {:?}", doc.err().unwrap())
            }

            let (correct_recipes, broken_recipes): (Vec<_>, Vec<_>) =
                correct_recipes.into_iter()
                    .map(|x| Recipe::try_from(x.unwrap()))
                    .partition(Result::is_ok);

            for recipe in broken_recipes {
                println!("Wrong recipe document in db: {:?}", recipe.err().unwrap())
            }

            let co: Vec<Recipe> = correct_recipes.into_iter().map(|r| r.unwrap()).collect();

            co
        }
        Err(_) => Vec::new(),
    }
}







