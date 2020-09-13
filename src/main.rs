mod model;

#[macro_use]
extern crate bson;
extern crate mongodb;

mod dao;
mod pagination;

use actix_web::{App, HttpServer, web};
use mongodb::Database;

mod recipe_routes;

#[derive(Clone)]
pub struct AppState {
    database: Database
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let database = match dao::init_database().await {
        Ok(db) => db,
        Err(err) => panic!(err)
    };

    let state = AppState { database };

    let addr = "127.0.0.1:8088";

    println!("Running on: {}", addr);

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(
                web::resource("/recipes")
                    .name("recipes")
                    .route(web::get().to(recipe_routes::get_recipes))
                    .route(web::post().to(recipe_routes::add_one_recipe))
            )
        // .route("/recipes", web::get().to(get_recipes))
    })
        .bind(addr)?
        .run()
        .await
}

