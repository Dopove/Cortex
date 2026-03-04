use actix_web::{web, App, HttpServer, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct Response {
    message: String,
}

async fn hello() -> impl Responder {
    web::Json(Response {
        message: "Hello from Rust + Actix Web!".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Rust server running on http://127.0.0.1:8080");
    HttpServer::new(|| App::new().route("/", web::get().to(hello)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
