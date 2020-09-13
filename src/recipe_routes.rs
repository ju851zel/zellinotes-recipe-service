use actix_web::{HttpResponse, Responder, web};
use actix_web::web::{ Json, Query};

use crate::{AppState, dao};
use crate::pagination::Pagination;
use crate::model::recipe::Recipe;

pub async fn add_one_recipe(data: web::Data<AppState>, recipe: Json<Recipe>) -> impl Responder {
    match dao::db_add_one_recipe(&data.database, recipe.into_inner()).await {
        Some(bson) => HttpResponse::Ok().json(bson),
        None => HttpResponse::InternalServerError().body("")
    }
}

pub async fn add_many_recipes(data: web::Data<AppState>, recipe: Json<Vec<Recipe>>) -> impl Responder {
    match dao::db_add_many_recipes(&data.database, recipe.into_inner()).await {
        Some(bson) => HttpResponse::Ok().json(bson),
        None => HttpResponse::InternalServerError().body("")
    }
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
