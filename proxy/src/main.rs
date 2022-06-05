use std::fmt::{Display, Formatter};
use actix_web::{get, web, App, HttpServer, Responder, HttpRequest, ResponseError, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use clap::Parser;
use reqwest::{Error, StatusCode};

#[derive(Debug)]
enum MyError {
    Reqwest(reqwest::Error),
}

impl ResponseError for MyError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Reqwest(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(format!("Error {}", self.status_code()))
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reqwest(err) => write!(f, "Upstream error: {err}"),
        }
    }
}

impl From<reqwest::Error> for MyError {
    fn from(value: Error) -> Self {
        Self::Reqwest(value)
    }
}

#[derive(Parser, Debug, Clone)]
struct Config {
    /// Upstream endpoint, not including slash at the end
    upstream: String,

    /// Our HTTP port
    #[clap(short = 'p', long = "--port")]
    port: u16,

    /// Headers to remove
    #[clap(short = 'R', long = "--remove")]
    headers_to_remove: Vec<String>,

    /// Headers to add
    #[clap(short = 'A', long = "--add")]
    headers_to_add: Vec<String>,
}

#[get("/{path:.*}")]
async fn proxy_get(req: HttpRequest, config: Data<Config>, path: String) -> Result<impl Responder, MyError> {
    let client = reqwest::Client::new();
    let map = actix_web::http::header::HeaderMap::new();
    for (key, value) in map.iter() {

    }
    let resp = client.get(config.upstream.clone() + path.as_str())
        // .headers(*req.headers())
        .send()
        .await?;
    // let upstream_req = reqwest::Builder();
    Ok("")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = Config::parse();
    let config2 = config.clone();

    HttpServer::new(move ||
        App::new()
            .app_data(Data::new(config.clone()))
            .service(proxy_get)
    )
        .bind(("127.0.0.1", config2.port))?
        .run()
        .await
}