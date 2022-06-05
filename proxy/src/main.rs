use std::fmt::{Display, Formatter};
use std::io::{BufReader, BufWriter, Read, Write};
use actix_web::{get, App, HttpServer, Responder, HttpRequest, ResponseError, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::web::{Buf, Data};
use brotlic::{DecompressorReader, DecompressorWriter};
use clap::Parser;

#[derive(Debug)]
enum MyError {
    Reqwest(reqwest::Error),
    IO(std::io::Error),
    Actix(actix_web::Error),
}

impl ResponseError for MyError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::Reqwest(_) => actix_web::http::StatusCode::BAD_GATEWAY,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
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
            Self::IO(err) => write!(f, "I/O error: {err}"),
            Self::Actix(err) => write!(f, "Actix error: {err}"),
        }
    }
}

impl From<reqwest::Error> for MyError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<std::io::Error> for MyError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<actix_web::Error> for MyError {
    fn from(value: actix_web::Error) -> Self {
        Self::Actix(value)
    }
}

#[derive(Parser, Debug, Clone)]
struct Config {
    /// Upstream endpoint, not including slash at the end
    upstream: String,

    /// Our HTTP port
    #[clap(short = 'p', long = "--port")]
    port: u16,
}

#[get("/{path:.*}")]
async fn proxy_get(req: HttpRequest, config: Data<Config>)
    -> Result<impl Responder, MyError>
{
    let client = reqwest::Client::new();
    let reqwest_response = client.get(config.upstream.clone() + req.path())
        .send()
        .await?;
    let mut response_builder = HttpResponse::build(actix_web::http::StatusCode::OK);
    let (tx, rx) = std::sync::mpsc::channel();
    // let mut decoder = DecompressorWriter::new(rx);
    let mut input = reqwest_response.bytes_stream();
    let mut decompressor = DecompressorReader::new(stream::iter(rx.into_iter()));
    let mut decompressor = DecompressorWriter::new(BufWriter::new(tx));
    let result = response_builder.streaming(decompressor);
    // FIXME: tokio::spawn
    loop {
        let buf = rx.recv()?;
        decompressor.write(buf);
    }
    Ok(result)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> Result<(), MyError> {
    let config = Config::parse();
    let config2 = config.clone();

    Ok(HttpServer::new(move ||
        App::new()
            .app_data(Data::new(config.clone()))
            .service(proxy_get)
    )
        .bind(("127.0.0.1", config2.port))?
        .run()
        .await?)
}