#[macro_use]
extern crate bson;
#[macro_use]
extern crate log;
extern crate mongodb;
extern crate simplelog;


use std::fs::File;

use actix_web::{App, HttpServer, web};
use mongodb::Database;
use simplelog::{CombinedLogger, Config, LevelFilter, TerminalMode, TermLogger, WriteLogger};

mod model;

mod dao;
mod pagination;

mod recipe_routes;

#[derive(Clone)]
pub struct AppState {
    database: Database
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let state = AppState { database: dao::init_database().await.unwrap() };

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
                    .route(web::post().to(recipe_routes::add_many_recipes))
            )
    }).bind(addr)?.run().await
}


fn init_logger() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info,
                            Config::default(),
                            TerminalMode::Mixed),
            WriteLogger::new(LevelFilter::Info,
                             Config::default(),
                             File::create("zellinotes.log").unwrap()),
        ]
    ).unwrap();
}
