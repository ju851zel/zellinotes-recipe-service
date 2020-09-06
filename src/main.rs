mod gateway;

use actix_web::{App, HttpServer, web};
use mongodb::Database;
use mongodb::error::Error;

use crate::recipe_routes::get_recipes;

mod mongo_connector;
mod recipe_routes;

#[derive(Clone)]
pub struct AppState {
    database: Database
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let database = match mongo_connector::start().await {
        Ok(db) => db,
        Err(err) => {
            panic!(err);
        }
    };

    let state = AppState { database };

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .route("/recipes", web::get().to(get_recipes))
    })
        .bind("127.0.0.1:8088")?
        .run()
        .await
}

