use std::convert::TryFrom;

use actix_web::{HttpResponse, Responder, web};
use actix_web::web::{Bytes, Json, Query};
use bson::Document;
use futures_util::StreamExt;
use mongodb::{Cursor, Database};
use mongodb::error::Error;
use mongodb::options::FindOptions;

use crate::{AppState, dao};
use crate::pagination::Pagination;
use crate::model::recipe::Recipe;

pub async fn add_recipe(data: web::Data<AppState>, recipe: Json<Recipe>) -> impl Responder {
    Json(dao::db_add_recipe(&data.database, recipe.into_inner()).await)
}

pub async fn get_recipes(params: Query<Pagination>, data: web::Data<AppState>) -> impl Responder {
    return if params.is_fully_set() {
        HttpResponse::Ok().json(dao::db_get_paged_recipes(&data.database, params.0).await)
    } else if params.is_fully_empty() {
        HttpResponse::Ok().json(dao::db_get_all_recipes(&data.database).await)
    } else {
        HttpResponse::BadRequest().body("")
    };
}


#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::time::SystemTime;

    use actix_web::{http};
    use actix_web::web::{Data, Json, Query};
    use bson::{Bson, Document};
    use mongodb::{Client, Database};
    use mongodb::error::Error;
    use mongodb::options::ClientOptions;

    use crate::AppState;
    use crate::recipe_routes::add_recipe;
    use crate::pagination::Pagination;
    use crate::model::recipe::Recipe;

    const RECIPE_COLLECTION: &str = "recipes";
    const TEST_URL: &str = "mongodb://localhost:26666";
    const TEST_APP_NAME: &str = "Zellinotes development recipes";
    const TEST_DATABASE: &str = "test_zellinotes_development_recipes";

    async fn init_test_database() -> Result<Database, Error> {
        let mut client_options = ClientOptions::parse(TEST_URL).await?;
        client_options.app_name = Some(TEST_APP_NAME.to_string());
        let client = Client::with_options(client_options)?;
        return Ok(client.database(TEST_DATABASE));
    }

    fn get_correct_recipe_bson() -> Bson {
        return bson!(
        {
            "_id": "ea",
            "cookingTimeInMinutes": 12,
            "created": "2020-09-11T12:21:21+00:00",
            "lastModified": "2020-09-11T12:21:21+00:00",
            "ingredients": [],
            "version": 1,
            "difficulty": "Easy",
            "description": "",
            "title": "SPaghetti",
            "tags": [],
            "image": null,
            "instructions": [],
            "defaultServings": 2
        });
    }

    #[actix_rt::test]
    async fn test_add_recipe() {
        let query: Query<Pagination> = Query::from_query("items=1&page=1&sorting=1").unwrap();
        let app_state = AppState { database: init_test_database().await.unwrap() };
        let recipe: Document = get_correct_recipe_bson().as_document().unwrap().to_owned();
        let recipe: Recipe = recipe.try_into().unwrap();
        let resp = add_recipe(Data::new(app_state), Json(recipe)).await;
        assert_eq!(http::StatusCode::BAD_REQUEST, http::StatusCode::BAD_REQUEST);
    }
}
