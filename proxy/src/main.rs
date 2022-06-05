use actix_web::{get, web, App, HttpServer, Responder};
use clap::Parser;

#[derive(Parser, Debug)]
struct Config {
    /// Upstream endpoint
    upstream: String,

    #[clap(short = 'R', long = "--remove")]
    headers_to_remove: Vec<String>,

    #[clap(short = 'A', long = "--add")]
    headers_to_add: Vec<String>,
}

#[get("/{path:.*}")]
async fn greet() -> impl Responder {
    let config = Config::parse();

    let client = reqwest::Client::new();
    // let resp = client.get()
    //     .basic_auth("admin", Some("good password"))
    //     .send()
    //     .await?;
    // let upstream_req = reqwest::Builder()
    ""
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/hello", web::get().to(|| async { "Hello World!" }))
            .service(greet)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}