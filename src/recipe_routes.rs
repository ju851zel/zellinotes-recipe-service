use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::dev::HttpResponseBuilder;
use actix_web::web::{Json, Query};

use crate::{dao, LogExtensionErr, LogExtensionOk, TakeDefined};
use crate::dao::Dao;
use crate::model::recipe::Recipe;
use crate::pagination::Pagination;

type RoutesError = String;

struct RecipeRoutes {}

impl RecipeRoutes {
    pub async fn update_one_recipe(req: HttpRequest, data: web::Data<Dao>, recipe: Json<Recipe>) -> impl Responder {
        update_one_recipe(req.clone(), data, recipe).await
            .log_if_ok(|updated| match updated {
                Ok(_) => info!("Updated recipe with id={:#?}",
                               extract_id_from_req(req.clone()).unwrap()),
                Err(_) => info!("Not found recipe with id={:#?}",
                                extract_id_from_req(req.clone()).unwrap()),
            })
            .log_if_err(|err| error!("Error updating recipe with id={:#?}, Err={:#?}",
                                     extract_id_from_req(req).unwrap(), err.1))
            .map(|result| result.take_defined())
            .map_err(|result| result.0)
            .take_defined()
    }
}

pub async fn update_one_recipe(req: HttpRequest, data: web::Data<Dao>, recipe: Json<Recipe>)
                               -> Result<Result<HttpResponseBuilder, HttpResponseBuilder>, (HttpResponseBuilder, RoutesError)> {
    let id = extract_id_from_req(req)
        .map_err(|err| (HttpResponse::BadRequest(), err))?;

    data.update_one_recipe(id, recipe.into_inner()).await
        .map_err(|err| (HttpResponse::InternalServerError(), err))
        .map(|updated| match updated {
            Some(_) => Ok(HttpResponse::Ok()),
            None => Err(HttpResponse::NotFound())
        })
}


fn extract_id_from_req(req: HttpRequest) -> Result<String, RoutesError> {
    req.match_info().get("id")
        .map(|id| id.to_string())
        .ok_or(format!("Error getting id param from HTTP request"))
}


pub async fn add_one_recipe(data: web::Data<Dao>, recipe: Json<Recipe>) -> impl Responder {
    let recipe = recipe.into_inner();
    match dao::db_add_one_recipe(&data.database, recipe.clone()).await {
        Ok(bson) => {
            info!("Added new recipe: {:?}", bson);
            HttpResponse::Ok().json(bson)
        }
        Err(err) => {
            error!("{}", err);
            HttpResponse::InternalServerError().body("")
        }
    }
}

pub async fn add_many_recipes(data: web::Data<Dao>, recipes: Json<Vec<Recipe>>) -> impl Responder {
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

pub async fn get_one_recipe(req: HttpRequest, data: web::Data<Dao>) -> impl Responder {
    match req.match_info().get("id") {
        Some(id) => {
            info!("get single recipe with id={}", id);
            match dao::db_get_one_recipe(&data.database, id.to_string()).await {
                Ok(recipe) => {
                    match recipe {
                        Some(recipe) => {
                            info!("get single recipe successful");
                            HttpResponse::Ok().json(recipe)
                        }
                        None => {
                            info!("recipe not found");
                            HttpResponse::NotFound().body("")
                        }
                    }
                }
                Err(err) => {
                    error!("{}", err);
                    HttpResponse::InternalServerError().body("")
                }
            }
        }
        _ => {
            error!("get one recipe no id provided: {:?}", req);
            HttpResponse::BadRequest().body("")
        }
    }
}


pub async fn get_many_recipes(params: Query<Pagination>, data: web::Data<Dao>) -> impl Responder {
    return if params.is_fully_set() {
        info!("get recipes with pagination: {:?}", params);
        match dao::db_get_all_recipes(&data.database, Some(params.0)).await {
            Ok(recipes) => {
                info!("success getting recipes with pagination");
                HttpResponse::Ok().json(recipes)
            }
            Err(err) => {
                error!("{}", err);
                HttpResponse::InternalServerError().body("")
            }
        }
    } else if params.is_fully_empty() {
        info!("get recipes no pagination");
        match dao::db_get_all_recipes(&data.database, None).await {
            Ok(recipes) => {
                info!("success getting all recipes ");
                HttpResponse::Ok().json(recipes)
            }
            Err(err) => {
                error!("{}", err);
                HttpResponse::InternalServerError().body("")
            }
        }
    } else {
        error!("get recipes with wrong pagination: {:?}", params);
        HttpResponse::BadRequest().body("")
    };
}


#[cfg(test)]
mod tests {
    use actix_web::{App, test, web};
    use bson::Bson;

    use crate::{Dao, init_logger};
    use crate::dao::dao_tests::{clean_up, init_test_database};
    use crate::recipe_routes::{add_many_recipes, add_one_recipe, get_many_recipes, get_one_recipe, update_one_recipe};

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

        init_logger();

        let mut app = test::init_service(App::new()
            .data(Dao { database: db.clone() })
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
        println!("{:#?}", resp);
        assert!(resp.status().is_success(), "{}", resp.status());

        clean_up(db).await;
    }

    #[actix_rt::test]
    async fn test_add_many_recipes() {
        let db = init_test_database().await.unwrap();

        let mut app = test::init_service(App::new()
            .data(Dao { database: db.clone() })
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
            .data(Dao { database: db.clone() })
            .route("/recipes", web::get().to(get_many_recipes))
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

    #[actix_rt::test]
    async fn test_get_one_recipe() {
        let db = init_test_database().await.unwrap();

        init_logger();
        let mut app = test::init_service(App::new()
            .data(Dao { database: db.clone() })
            .route("/recipes/{id}", web::get().to(get_one_recipe))
            .route("/recipes/{id}", web::post().to(add_one_recipe))).await;

        let req = test::TestRequest::get().uri("/recipes/hello").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error(), "{}", resp.status());

        let payload = create_one_recipe().as_document().unwrap().clone();

        let req = test::TestRequest::post()
            .set_json(&payload).uri("/recipes/new").to_request();

        let resp = test::call_service(&mut app, req).await;
        println!("{:#?}", resp);
        assert!(resp.status().is_success(), "{}", resp.status());

        let req = test::TestRequest::get().uri("/recipes/hello").to_request();
        let resp = test::call_service(&mut app, req).await;
        // let body = resp.response_mut().take_body().try_fold(|e| e);
        // let x = body.as_ref().unwrap().to_owned();
        // let x1 = std::str::from_utf8(x).unwrap();
        // println!("{:#?}", x);

        assert!(resp.status().is_client_error(), "{}", resp.status());


        clean_up(db).await;
    }

    #[actix_rt::test]
    async fn test_update_one_recipe() {
        let db = init_test_database().await.unwrap();

        let mut app = test::init_service(App::new()
            .data(Dao { database: db.clone() })
            .route("/recipes/{id}", web::get().to(get_one_recipe))
            .route("/recipes/{id}", web::post().to(add_one_recipe))
            .route("/recipes/{id}", web::put().to(update_one_recipe))).await;


        let mut payload = create_one_recipe().as_document().unwrap().clone();

        let req = test::TestRequest::post()
            .set_json(&payload).uri("/recipes/new").to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success(), "{}", resp.status());

        //todo get body from resp and extract id.

        payload.insert("difficulty", "Medium");
        let id = "id".to_string();
        let url = format!("/recipes/{}", id);

        let req = test::TestRequest::put().set_json(&payload).uri(&url).to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success(), "{}", resp.status());
        // todo check if recipe was updated


        clean_up(db).await;
    }
}
