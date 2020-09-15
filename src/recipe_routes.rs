use actix_web::{HttpResponse, Responder, web};
use actix_web::web::{Json, Query};

use crate::{AppState, dao};
use crate::model::recipe::Recipe;
use crate::pagination::Pagination;

pub async fn add_one_recipe(data: web::Data<AppState>, recipe: Json<Recipe>) -> impl Responder {
    let recipe = recipe.into_inner();
    match dao::db_add_one_recipe(&data.database, recipe.clone()).await {
        Ok(bson) => {
            info!("Added new recipe: {:?}", recipe);
            HttpResponse::Ok().json(bson)
        }
        Err(err) => {
            error!("{}", err);
            HttpResponse::InternalServerError().body("")
        }
    }
}

pub async fn add_many_recipes(data: web::Data<AppState>, recipes: Json<Vec<Recipe>>) -> impl Responder {
    let recipes = recipes.into_inner();
    match dao::db_add_many_recipes(&data.database, recipes.clone()).await {
        Ok(bson) => {
            info!("Added new many recipes: {:?}", recipes);
            HttpResponse::Ok().json(bson)
        }
        Err(err) => {
            error!("{}", err);
            HttpResponse::InternalServerError().body("")
        }
    }
}


pub async fn get_recipes(params: Query<Pagination>, data: web::Data<AppState>) -> impl Responder {
    return if params.is_fully_set() {
        info!("get recipes with pagination: {:?}", params);
        HttpResponse::Ok().json(dao::db_get_all_recipes(&data.database, Some(params.0)).await)
    } else if params.is_fully_empty() {
        info!("get recipes no pagination");
        HttpResponse::Ok().json(dao::db_get_all_recipes(&data.database, None).await)
    } else {
        error!("get recipes with wrong pagination: {:?}", params);
        HttpResponse::BadRequest().body("")
    };
}


#[cfg(test)]
mod tests {
    use actix_web::{App, test, web};
    use bson::Bson;

    use crate::AppState;
    use crate::dao::dao_tests::{clean_up, init_test_database};
    use crate::recipe_routes::{add_many_recipes, add_one_recipe, get_recipes};

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
            .route("/recipes", web::get().to(get_recipes))
            .route("/addManyRecipes", web::post().to(add_many_recipes))).await;

        let req = test::TestRequest::get().uri("/recipes").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success(), "{}", resp.status());


        let payload = create_many_recipes();
        let payload = payload.as_array().unwrap().clone();
        let payload: Vec<Bson> = (0..50).into_iter().map(|_| payload.get(0).unwrap().clone()).collect();
        let payload = Bson::Array(payload);

        let req = test::TestRequest::post()
            .set_json(&payload).uri("/addManyRecipes").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success(), "{}", resp.status());


        let req = test::TestRequest::get().uri("/recipes").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success(), "{}", resp.status());

        clean_up(db).await;
    }
}
