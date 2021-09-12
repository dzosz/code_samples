#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod employees;
mod db;
mod schema;
mod error_handler;

use actix_web::{web, App, HttpRequest, HttpServer, Responder};

async fn welcome(request: HttpRequest) -> impl Responder {
    let name = request.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("running");
    /*
    let addr = "127.0.0.1:8000".parse().unwrap();
    let socket = TcpSocket::new_v4()?;
    socket.set_reuseaddr(true)?;
    assert!(socket.reuseaddr().unwrap());
    socket.bind(addr)?;
    */

    HttpServer::new(|| {
        App::new()
            .configure(employees::init_routes)
    })
    .bind( "127.0.0.1:8000")?
    .run()
    .await
}
