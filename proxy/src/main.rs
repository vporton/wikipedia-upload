use std::cmp::min;
use std::fmt::{Display, Formatter};
use std::io::{ErrorKind, Read};
use std::pin::Pin;
use actix_web::{get, App, HttpServer, Responder, HttpRequest, ResponseError, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::web::{Bytes, Data};
use clap::Parser;
use async_stream::try_stream;
use futures::executor::block_on;
use futures::{Stream, TryStreamExt};
use futures::stream::StreamExt;

#[derive(Debug)]
enum MyError {
    Reqwest(reqwest::Error),
    IO(std::io::Error),
    Actix(actix_web::Error),
}

impl std::error::Error for MyError { }

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

impl<T: Into<MyError> + Clone> From<&T> for MyError {
    fn from(value: &T) -> Self {
        value.clone().into()
    }
}

// impl<T: Into<MyError> + Copy> From<&mut T> for MyError {
//     fn from(value: &mut T) -> Self {
//         (*value).into()
//     }
// }

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
    Ok(HttpResponse::Ok()
        .content_type("application/xml")
        .streaming(proxy_get_stream(req, &*config).await?))
}

struct BytesStreamRead {
    stream: Pin<Box<dyn futures_core::Stream<Item = std::io::Result<Bytes>>>>,
    upstream_buf: Vec<u8>,
}

impl BytesStreamRead {
    pub fn new(stream: Pin<Box<dyn futures_core::Stream<Item = std::io::Result<Bytes>>>>) -> Self {
        Self {
            stream,
            upstream_buf: Vec::new(),
        }
    }
}

impl Read for BytesStreamRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.upstream_buf.is_empty() {
            if let Some(upstream_buf) = block_on(async {
                self.stream.next().await
            }) {
                self.upstream_buf = upstream_buf?.to_vec();
            } else {
                return Ok(0);
            }
        }
        let size = min(buf.len(), self.upstream_buf.len());
        buf[.. size].clone_from_slice(&self.upstream_buf[.. size]);
        self.upstream_buf = Vec::from(&self.upstream_buf[size ..]);
        return Ok(size);
    }
}

async fn proxy_get_stream(req: HttpRequest, config: &Config) -> Result<impl Stream<Item = Result<Bytes, MyError>>, MyError> {
    let client = reqwest::Client::new();
    let reqwest_response = client.get(config.upstream.clone() + req.path())
        .send()
        .await?;
    let input = reqwest_response.bytes_stream();
    let input = input
        .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e));
    let mut decompressor = brotli::Decompressor::new(BytesStreamRead::new(Box::pin(input)), 4096);
    Ok(try_stream! {
        let buf0 = [0u8; 4096];
        let mut buf = buf0;
        loop {
            if buf.len() == 0 {
                buf = buf0;
            }
            println!("YYY");
            let bytes = decompressor.read(&mut buf)?;
            println!("XXX={bytes}");
            if bytes == 0 {
                break;
            }
            println!("BUF={}", String::from_utf8_lossy(&buf[.. bytes]));
            yield Bytes::copy_from_slice(&buf[.. bytes]);
        }
    })
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