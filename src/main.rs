#[macro_use]
extern crate bson;
#[macro_use]
extern crate log;
extern crate mongodb;
extern crate simplelog;


use std::fs::File;

use actix_web::{App, HttpServer, web, Responder};
use mongodb::Database;
use simplelog::{CombinedLogger, Config, LevelFilter, TerminalMode, TermLogger, WriteLogger};

use crate::dao::Dao;
use crate::recipe_routes::RecipeRoutes;

mod model;

mod dao;
mod pagination;

mod recipe_routes;


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let dao = Dao::new().await.unwrap();

    let addr = "127.0.0.1:8088";

    println!("Running on: {}", addr);

    HttpServer::new(move || {
        App::new()
            .data(dao.clone())
            .service(
                web::scope("/api/v1")
                    .service(web::resource("/recipes")
                        .route(web::get().to(recipe_routes::get_many_recipes))
                        .route(web::post().to(RecipeRoutes::add_many_recipes)))
                    .service(web::resource("/recipes/{id}")
                                 .route(web::post().to(RecipeRoutes::add_one_recipe))
                                 .route(web::get().to(recipe_routes::get_one_recipe))
                             .route(web::put().to(RecipeRoutes::update_one_recipe))
                    )
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


pub trait LogExtensionOk<T> {
    fn log_if_ok<F: FnOnce(&T)>(self, if_ok: F) -> Self;
}

pub trait LogExtensionErr<E> {
    fn log_if_err<F: FnOnce(&E)>(self, if_err: F) -> Self;
}

pub trait TakeDefined<T> {
    fn take_defined(self) -> T;
}

impl<T, E> LogExtensionOk<T> for Result<T, E> {
    fn log_if_ok<F: FnOnce(&T)>(self, if_ok: F) -> Self {
        if let Ok(ok) = &self {
            if_ok(ok)
        }
        self
    }
}

impl<T, E> LogExtensionErr<E> for Result<T, E> {
    fn log_if_err<F: FnOnce(&E)>(self, if_err: F) -> Self {
        if let Err(err) = &self {
            if_err(err)
        }
        self
    }
}

impl<T> TakeDefined<T> for Result<T, T> {
    fn take_defined(self) -> T {
        return match self {
            Ok(ok) => ok,
            Err(err) => err
        };
    }
}
