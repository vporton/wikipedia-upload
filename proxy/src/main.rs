use std::fmt::{Display, Formatter};
use std::io::{BufReader, BufWriter, Read, Write};
use actix_web::{get, App, HttpServer, Responder, HttpRequest, ResponseError, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::web::{Buf, Bytes, Data};
use brotlic::{BrotliDecoder, DecompressorReader, DecompressorWriter};
use clap::Parser;
use async_stream::try_stream;
use brotlic::decode::{DecodeError, DecoderInfo};
use futures::Stream;
use futures::stream::StreamExt;

#[derive(Debug)]
enum MyError {
    Reqwest(reqwest::Error),
    IO(std::io::Error),
    Actix(actix_web::Error),
    Decode(DecodeError),
}

impl std::error::Error for MyError { }

impl ResponseError for MyError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::Reqwest(_) | Self::Decode(_) => actix_web::http::StatusCode::BAD_GATEWAY,
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
            Self::Decode(err) => write!(f, "Decode error: {err}"),
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

impl From<DecodeError> for MyError {
    fn from(value: DecodeError) -> Self {
        Self::Decode(value)
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
        // .streaming(Box::pin(proxy_get_stream(req, &*config).await?)))
        .streaming(proxy_get_stream(req, &*config).await?))
}

async fn proxy_get_stream(req: HttpRequest, config: &Config) -> Result<impl Stream<Item = Result<Bytes, MyError>>, MyError> {
    let client = reqwest::Client::new();
    let reqwest_response = client.get(config.upstream.clone() + req.path())
        .send()
        .await?;
    let mut input = reqwest_response.bytes_stream();
    let mut decompressor = BrotliDecoder::new();
    // FIXME: tokio::spawn
    Ok(try_stream! {
        let mut buf2 = [0u8; 4096];
        loop {
            if buf2.is_empty() {
                buf2 = [0u8; 4096];
            }
            let mut buf = input.next().await;
            loop {
                if let Some(buf) = buf {
                    let mut buf = buf?;
                    let result = decompressor.decompress(&buf, &mut buf2)?;
                    yield Bytes::copy_from_slice(&buf2 as &[u8]);
                    // buf = buf.slice(result.bytes_read ..);
                    if result.info == DecoderInfo::Finished {
                        break;
                    }
                }
            }
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