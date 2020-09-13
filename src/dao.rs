use std::convert::TryFrom;

use bson::Document;
use futures_util::StreamExt;
use mongodb::{bson::{Bson, doc}, Client, options::FindOptions};
use mongodb::Database;
use mongodb::error::Error;
use mongodb::options::ClientOptions;

use crate::model::recipe::Recipe;
use crate::pagination::Pagination;

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

pub async fn db_add_one_recipe(db: &Database, recipe: Recipe) -> Option<Bson> {
    return match db.collection(RECIPE_COLLECTION)
        .insert_one(recipe.into(), None).await {
        Ok(result) => Some(result.inserted_id),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    };
}

pub async fn db_add_many_recipes(db: &Database, recipes: Vec<Recipe>) -> Option<Bson> {
    return match db.collection(RECIPE_COLLECTION)
        .insert_many(recipes.into_iter().map(|r| r.into()).collect::<Vec<Document>>(), None).await {
        Ok(result) => {
            let vek = result.inserted_ids.values().map(|b: &Bson| b.to_owned()).collect::<Vec<Bson>>();
            Some(Bson::from(vek))
        }
        Err(err) => {
            println!("{:?}", err);
            None
        }
    };
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


#[cfg(test)]
mod tests {
    use actix_web::{App, test, web};
    use bson::Bson;
    use mongodb::{Client, Database};
    use mongodb::error::Error;
    use mongodb::options::ClientOptions;

    use crate::AppState;
    use crate::recipe_routes::{add_many_recipes, add_one_recipe, get_recipes};

    const TEST_URL: &str = "mongodb://localhost:26666";
    const TEST_APP_NAME: &str = "Zellinotes development recipes";
    const TEST_DATABASE: &str = "test_zellinotes_development_recipes";

    async fn init_test_database() -> Result<Database, Error> {
        let mut client_options = ClientOptions::parse(TEST_URL).await?;
        client_options.app_name = Some(TEST_APP_NAME.to_string());
        let client = Client::with_options(client_options)?;
        return Ok(client.database(TEST_DATABASE));
    }

    async fn clean_up(db: Database) {
        db.drop(None).await;
    }


    fn create_many_recipes() -> Bson {
        let vector = vec!(bson!(
        {
            "cookingTimeInMinutes": 12,
            "created": "2020-09-11T12:21:21+00:00",
            "lastModified": "2020-09-11T12:21:21+00:00",
            "ingredients": [
                {
                    "id": "0",
                    "amount": 200,
                    "title" : "Wheat",
                    "measurementUnit": "Kilogramm"
                },
                {
                    "id": "1",
                    "amount": 3000,
                    "title" : "Milk",
                    "measurementUnit": "Milliliter"
                }
            ],
            "version": 1,
            "difficulty": "Easy",
            "description": "",
            "title": "Spaghetti",
            "tags": [],
            "image": null,
            "instructions": [],
            "defaultServings": 2
        }),
                          bson!({
            "cookingTimeInMinutes": 12,
            "created": "2020-09-11T12:21:21+00:00",
            "lastModified": "2020-09-11T12:21:21+00:00",
            "ingredients": [],
            "version": 1,
            "difficulty": "Easy",
            "description": "",
            "title": "Spaghetti",
            "tags": [],
            "image": null,
            "instructions": [],
            "defaultServings": 2
        }),
                          bson!({
            "cookingTimeInMinutes": 12,
            "created": "2020-09-11T12:21:21+00:00",
            "lastModified": "2020-09-11T12:21:21+00:00",
            "ingredients": [],
            "version": 1,
            "difficulty": "Easy",
            "description": "",
            "title": "Spaghetti",
            "tags": [],
            "image": null,
            "instructions": [],
            "defaultServings": 2
        }));
        return Bson::Array(vector);
    }

    fn create_one_recipe() -> Bson {
        bson!(
        {
            "cookingTimeInMinutes": 12,
            "created": "2020-09-11T12:21:21+00:00",
            "lastModified": "2020-09-11T12:21:21+00:00",
            "ingredients": [],
            "version": 1,
            "difficulty": "Easy",
            "description": "",
            "title": "Spaghetti",
            "tags": [],
            "image": null,
            "instructions": [],
            "defaultServings": 2
        })
    }

    #[actix_rt::test]
    async fn test_add_single_recipe() {
        let db = init_test_database().await.unwrap();

        let mut app = test::init_service(App::new()
            .data(AppState { database: db.clone() })
            .route("/addOneRecipe", web::post().to(add_one_recipe))).await;

        let req = test::TestRequest::post().uri("/addOneRecipe").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());

        let payload = create_many_recipes();
        let req = test::TestRequest::post()
            .set_json(&payload).uri("/addOneRecipe").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error(), "{}", resp.status());

        let payload = create_one_recipe();
        let req = test::TestRequest::post()
            .set_json(&payload).uri("/addOneRecipe").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success(), "{}", resp.status());

        clean_up(db).await;
    }

    #[actix_rt::test]
    async fn test_add_many_recipes() {
        let db = init_test_database().await.unwrap();

        let mut app = test::init_service(App::new()
            .data(AppState { database: db.clone() })
            .route("/addManyRecipes", web::post().to(add_many_recipes))).await;

        let req = test::TestRequest::post().uri("/addManyRecipes").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());

        let payload = create_one_recipe();
        let req = test::TestRequest::post()
            .set_json(&payload)
            .uri("/addManyRecipes").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());

        let payload = create_many_recipes();
        let req = test::TestRequest::post()
            .set_json(&payload).uri("/addManyRecipes").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success(), "{}", resp.status());

        clean_up(db).await;
    }

    #[actix_rt::test]
    async fn test_get_many_recipes() {
        let db = init_test_database().await.unwrap();

        let mut app = test::init_service(App::new()
            .data(AppState { database: db.clone() })
            .route("/recipes", web::get().to(get_recipes))).await;

        let req = test::TestRequest::get().uri("/recipes").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success(), "{}", resp.status());

        clean_up(db).await;
    }
}










