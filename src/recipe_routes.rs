use actix_web::{Responder, HttpResponse, web, HttpRequest};
use crate::AppState;

pub async fn get_recipes(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
