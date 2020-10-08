use actix_web::{Either, HttpRequest, HttpResponse, Responder, web};
use actix_web::web::{Json, Query};
use bson::oid::ObjectId;

use crate::dao::{Dao, DaoError};
use crate::model::recipe::Recipe;
use crate::pagination::Pagination;

pub struct RecipeRoutes {}

impl RecipeRoutes {
    pub async fn update_one_recipe_without_image(req: HttpRequest, database: web::Data<Dao>, recipe: Json<Recipe>) -> impl Responder {
        let id = match extract_id_from_req(req) {
            Some(id) => id,
            None => return HttpResponse::BadRequest()
        };

        match database.update_recipe_ignore_image(id, recipe.into_inner()).await {
            Ok(_) => HttpResponse::Ok(),
            Err(DaoError::DocumentNotFound) => HttpResponse::NotFound(),
            Err(DaoError::DatabaseError(_)) => HttpResponse::InternalServerError(),
            Err(DaoError::RecipeFormatError(_)) => HttpResponse::InternalServerError(),
        }
    }

    pub async fn add_one_recipe(database: web::Data<Dao>, recipe: Json<Recipe>) -> Either<impl Responder, impl Responder> {
        match database.insert_recipe(recipe.into_inner()).await {
            Ok(bson) => Either::A(HttpResponse::Ok().json(bson)),
            Err(DaoError::DocumentNotFound) => Either::B(HttpResponse::NotFound()),
            Err(DaoError::DatabaseError(_)) => Either::B(HttpResponse::InternalServerError()),
            Err(DaoError::RecipeFormatError(_)) => Either::B(HttpResponse::InternalServerError()),
        }
    }

    pub async fn delete_one_recipe(req: HttpRequest, database: web::Data<Dao>) -> impl Responder {
        let id = match extract_id_from_req(req) {
            Some(id) => id,
            None => return HttpResponse::BadRequest()
        };

        match database.delete_one_recipe(id).await {
            Ok(_) => HttpResponse::Ok(),
            Err(DaoError::DocumentNotFound) => HttpResponse::NotFound(),
            Err(DaoError::DatabaseError(_)) => HttpResponse::InternalServerError(),
            Err(DaoError::RecipeFormatError(_)) => HttpResponse::InternalServerError(),
        }

    }

    pub async fn add_many_recipes(database: web::Data<Dao>, recipes: Json<Vec<Recipe>>) -> Either<impl Responder, impl Responder> {
        match database.add_many_recipes(recipes.into_inner()).await {
            Ok(bson) => Either::A(HttpResponse::Ok().json(bson)),
            Err(DaoError::DocumentNotFound) =>  Either::B(HttpResponse::NotFound()),
            Err(DaoError::DatabaseError(_)) => Either::B(HttpResponse::InternalServerError()),
            Err(DaoError::RecipeFormatError(_)) =>  Either::B(HttpResponse::InternalServerError()),
        }
    }

    pub async fn get_one_recipe_without_image(req: HttpRequest, database: web::Data<Dao>) -> Either<impl Responder, impl Responder> {
        let id = match extract_id_from_req(req) {
            Some(id) => id,
            None => return Either::B(HttpResponse::BadRequest())
        };

        match database.get_one_recipe_without_image(id).await {
            Ok(recipe) => Either::A(HttpResponse::Ok().json(recipe)),
            Err(DaoError::DocumentNotFound) =>  Either::B(HttpResponse::NotFound()),
            Err(DaoError::DatabaseError(_)) => Either::B(HttpResponse::InternalServerError()),
            Err(DaoError::RecipeFormatError(_)) =>  Either::B(HttpResponse::InternalServerError()),
        }
    }

    pub async fn get_one_recipe_image(req: HttpRequest, database: web::Data<Dao>) -> Either<impl Responder, impl Responder> {
        let id = match extract_id_from_req(req) {
            Some(id) => id,
            None => return Either::A(HttpResponse::BadRequest())
        };

        match database.get_one_recipe_image(id).await {
            Ok(image) => Either::B(HttpResponse::Ok().body(image)),
            Err(DaoError::DocumentNotFound) => Either::A(HttpResponse::NotFound()),
            Err(DaoError::DatabaseError(_)) => Either::A(HttpResponse::InternalServerError()),
            Err(DaoError::RecipeFormatError(_)) => Either::A(HttpResponse::InternalServerError()),
        }
    }

    pub async fn update_one_recipe_image(req: HttpRequest, database: web::Data<Dao>, image: String) -> impl Responder {
        let id = match extract_id_from_req(req) {
            Some(id) => id,
            None => return HttpResponse::BadRequest()
        };

        match database.update_one_recipe_image(id, Some(image)).await {
            Ok(_) => HttpResponse::Ok(),
            Err(DaoError::DocumentNotFound) => HttpResponse::NotFound(),
            Err(DaoError::DatabaseError(_)) => HttpResponse::InternalServerError(),
            Err(DaoError::RecipeFormatError(_)) => HttpResponse::InternalServerError(),
        }
    }

    pub async fn delete_one_recipe_image(req: HttpRequest, database: web::Data<Dao>) -> impl Responder {
        let id = match extract_id_from_req(req) {
            Some(id) => id,
            None => return HttpResponse::BadRequest()
        };

        match database.update_one_recipe_image(id, None).await {
            Ok(_) => HttpResponse::Ok(),
            Err(DaoError::DocumentNotFound) => HttpResponse::NotFound(),
            Err(DaoError::DatabaseError(_)) => HttpResponse::InternalServerError(),
            Err(DaoError::RecipeFormatError(_)) => HttpResponse::InternalServerError(),
        }
    }

    pub async fn get_many_recipes(params: Query<Pagination>, database: web::Data<Dao>) -> Either<impl Responder, impl Responder> {
        let result = if params.0.is_fully_set() {
            database.get_many_recipes(Some(params.0)).await
        } else if params.is_fully_empty() {
            database.get_many_recipes(None).await
        } else {
            return Either::B(HttpResponse::BadRequest());
        };

        match result {
            Ok(recipes) => Either::A(HttpResponse::Ok().json(recipes)),
            Err(DaoError::DatabaseError(_)) => Either::B(HttpResponse::InternalServerError()),
            Err(DaoError::DocumentNotFound) => Either::B(HttpResponse::NotFound()),
            Err(DaoError::RecipeFormatError(_)) => Either::B(HttpResponse::InternalServerError()),
        }
    }
}


fn extract_id_from_req(req: HttpRequest) -> Option<ObjectId> {
    match req.match_info().get("id") {
        Some(id) => match ObjectId::with_string(&id) {
            Ok(oid) => return Some(oid),
            _ => error!("Error provided id is no Object id")
        }
        None => error!("Error getting id param from HTTP request={:#?}", req)
    }
    return None;
}


#[cfg(test)]
mod tests {
    use actix_web::{App, test, web};
    use actix_web::http::StatusCode;
    use bson::Bson;
    use serial_test::serial;

    use crate::dao::dao_tests::{before, cleanup_after};
    use crate::recipe_routes::RecipeRoutes;

    fn create_many_recipes() -> Bson {
        let vector = vec!(create_one_recipe_no_ingredients(),
                          create_one_recipe_with_ingredients(),
                          create_one_recipe_with_ingredients()
        );
        return Bson::Array(vector);
    }

    fn create_one_recipe_no_ingredients() -> Bson {
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

    fn create_one_recipe_with_image() -> Bson {
        let bson = create_one_recipe_no_ingredients();
        let mut doc = bson.as_document().unwrap().to_owned();
        doc.insert("image", "image".to_string()).unwrap()
    }

    fn create_one_recipe_with_ingredients() -> Bson {
        bson!(
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
        })
    }

    #[actix_rt::test]
    #[serial]
    async fn test_add_single_recipe() {
        let dao = before().await;

        let mut app = test::init_service(App::new()
            .data(dao.clone())
            .route("/addOneRecipe", web::post().to(RecipeRoutes::add_one_recipe))).await;

        let req = test::TestRequest::post().uri("/addOneRecipe").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());

        let payload = create_many_recipes();
        let req = test::TestRequest::post()
            .set_json(&payload).uri("/addOneRecipe").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error(), "{}", resp.status());

        let payload = create_one_recipe_no_ingredients();
        let req = test::TestRequest::post()
            .set_json(&payload).uri("/addOneRecipe").to_request();
        let resp = test::call_service(&mut app, req).await;
        println!("{:#?}", resp);
        assert!(resp.status().is_success(), "{}", resp.status());

        let payload = create_one_recipe_with_ingredients();
        let req = test::TestRequest::post()
            .set_json(&payload).uri("/addOneRecipe").to_request();
        let resp = test::call_service(&mut app, req).await;
        println!("{:#?}", resp);
        assert!(resp.status().is_success(), "{}", resp.status());

        let payload = create_one_recipe_with_image();
        let req = test::TestRequest::post()
            .set_json(&payload).uri("/addOneRecipe").to_request();
        let resp = test::call_service(&mut app, req).await;
        println!("{:#?}", resp);
        assert!(resp.status().is_success(), "{}", resp.status());

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn test_delete_single_recipe() {
        let dao = before().await;

        let mut app = test::init_service(App::new()
            .data(dao.clone())
            .route("/deleteOneRecipe/{id}", web::delete().to(RecipeRoutes::delete_one_recipe))
            .route("/addOneRecipe", web::post().to(RecipeRoutes::add_one_recipe))).await;

        let payload = create_one_recipe_with_ingredients();

        let req = test::TestRequest::delete()
            .set_json(&payload).uri("/deleteOneRecipe").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let req = test::TestRequest::delete()
            .set_json(&payload).uri("/deleteOneRecipe/hello").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let req = test::TestRequest::delete()
            .set_json(&payload).uri("/deleteOneRecipe/5f7333360051027600b01a36").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let req = test::TestRequest::post()
            .set_json(&payload).uri("/addOneRecipe").to_request();
        let resp = test::call_service(&mut app, req).await;

        let body: Bson = test::read_body_json(resp).await;
        let inserted_id = body.as_object_id().unwrap().to_string();

        let path = format!("/deleteOneRecipe/{}", inserted_id);
        let req = test::TestRequest::delete().set_json(&payload).uri(&path).to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn test_add_many_recipes() {
        let dao = before().await;

        let mut app = test::init_service(App::new()
            .data(dao.clone())
            .route("/addManyRecipes", web::post().to(RecipeRoutes::add_many_recipes))).await;

        let req = test::TestRequest::post().uri("/addManyRecipes").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error());

        let payload = create_one_recipe_no_ingredients();
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

        cleanup_after(dao).await;
    }


    #[actix_rt::test]
    #[serial]
    async fn test_get_many_recipes() {
        let dao = before().await;

        let mut app = test::init_service(App::new()
            .data(dao.clone())
            .route("/recipes", web::get().to(RecipeRoutes::get_many_recipes))
            .route("/addManyRecipes", web::post().to(RecipeRoutes::add_many_recipes))).await;

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

        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    #[serial]
    async fn test_get_one_recipe() {
        let dao = before().await;

        let mut app = test::init_service(App::new()
            .data(dao.clone())
            .route("/recipes/{id}", web::get().to(RecipeRoutes::get_one_recipe_without_image))
            .route("/recipes/{id}", web::post().to(RecipeRoutes::add_one_recipe))).await;

        let req = test::TestRequest::get().uri("/recipes/hello").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error(), "{}", resp.status());

        let payload = create_one_recipe_no_ingredients().as_document().unwrap().clone();

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


        cleanup_after(dao).await;
    }

    #[actix_rt::test]
    async fn test_update_one_recipe() {
        let dao = before().await;

        let mut app = test::init_service(App::new()
            .data(dao.clone())
            .route("/recipes/{id}", web::get().to(RecipeRoutes::get_one_recipe_without_image))
            .route("/recipes/{id}", web::post().to(RecipeRoutes::add_one_recipe))
            .route("/recipes/{id}", web::put().to(RecipeRoutes::update_one_recipe_without_image))).await;

        let mut payload = create_one_recipe_no_ingredients().as_document().unwrap().clone();

        let req = test::TestRequest::post()
            .set_json(&payload).uri("/recipes/new").to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success(), "{}", resp.status());

        //     todo get body from resp and extract id.

        payload.insert("difficulty", "Medium");
        let id = "5f7333360051027600b01a36".to_string();
        let url = format!("/recipes/{}", id);

        let req = test::TestRequest::put().set_json(&payload).uri(&url).to_request();

        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success(), "{}", resp.status());
        // todo check if recipe was updated


        cleanup_after(dao).await;
    }
}
