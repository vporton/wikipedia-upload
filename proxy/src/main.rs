use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use actix_web::{get, web, App, HttpServer, Responder, HttpRequest, ResponseError, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Header};
use clap::Parser;
use reqwest::header::{HeaderName, HeaderValue, InvalidHeaderName, InvalidHeaderValue};

#[derive(Debug)]
struct WrongHeaderError;

impl WrongHeaderError {
    pub fn new() -> Self {
        Self { }
    }
}

impl Display for WrongHeaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wrong header specified")
    }
}

#[derive(Debug)]
enum MyError {
    Reqwest(reqwest::Error),
    IO(std::io::Error),
    WrongHeader(WrongHeaderError),
    InvalidHeaderName(InvalidHeaderName),
    InvalidHeaderValue(InvalidHeaderValue),
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
            Self::WrongHeader(err) => write!(f, "Header error: {err}"),
            Self::InvalidHeaderName(err) => write!(f, "Header name error: {err}"),
            Self::InvalidHeaderValue(err) => write!(f, "Header value error: {err}"),
            Self::Actix(err) => write!(f, "Actix error: {err}"),
        }
    }
}

impl From<reqwest::Error> for MyError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<WrongHeaderError> for MyError {
    fn from(value: WrongHeaderError) -> Self {
        Self::WrongHeader(value)
    }
}

impl From<std::io::Error> for MyError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<InvalidHeaderName> for MyError {
    fn from(value: InvalidHeaderName) -> Self {
        Self::InvalidHeaderName(value)
    }
}

impl From<InvalidHeaderValue> for MyError {
    fn from(value: InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValue(value)
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

    /// Headers to remove
    #[clap(short = 'R', long = "--remove")]
    headers_to_remove: Vec<String>,

    /// Headers to add
    #[clap(short = 'A', long = "--add")]
    headers_to_add: Vec<String>,
}

#[derive(Clone)]
struct ConfigMore {
    headers_to_remove_set: HashSet<String>,
    headers_to_add: Vec<(String, String)>,
}

#[get("/{path:.*}")]
async fn proxy_get(req: HttpRequest, config: Data<Config>, config_more: Data<ConfigMore>, path: String)
    -> Result<impl Responder, MyError>
{
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    for (key, value) in req.headers().iter() {
        if !config_more.headers_to_remove_set.contains(&key.to_string()) {
            headers.insert(key.clone(), value.clone());
        }
    }
    for (key, value) in &config_more.headers_to_add {
        headers.insert(HeaderName::from_bytes(key.as_bytes())?, HeaderValue::from_bytes(value.as_bytes())?);
    }
    let resp = client.get(config.upstream.clone() + path.as_str())
        .headers(headers)
        .send()
        .await?;
    Ok(resp.bytes().await?)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> Result<(), MyError> {
    let config = Config::parse();
    let config2 = config.clone();
    let config3 = config.clone();
    let config_more = ConfigMore {
        headers_to_remove_set: HashSet::from_iter(config.headers_to_remove.into_iter()),
        headers_to_add: config.headers_to_add
            .into_iter()
            .map(|h| -> Result<_, MyError> {
                let p = h.split_once(": ").ok_or::<MyError>(WrongHeaderError::new().into())?;
                Ok((p.0.to_string(), p.1.to_string()))
            })
            .collect::<Result<Vec<(String, String)>, MyError>>()?,
    };

    Ok(HttpServer::new(move ||
        App::new()
            .app_data(Data::new(config3.clone()))
            .app_data(Data::new(config_more.clone()))
            .service(proxy_get)
    )
        .bind(("127.0.0.1", config2.port))?
        .run()
        .await?)
}