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
use alloc_stdlib::heap_alloc::HeapPrealloc; // TODO: Should use stdlib.
use brotli_decompressor::{BrotliDecompressStream, BrotliState};
use brotli_decompressor::reader::HuffmanCode;
#[macro_use]
extern crate alloc_stdlib;

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

// brotli_decompressor::declare_stack_allocator_struct!(MemPool, heap);

async fn proxy_get_stream(req: HttpRequest, config: &Config) -> Result<impl Stream<Item = Result<Bytes, MyError>>, MyError> {
    let client = reqwest::Client::new();
    let reqwest_response = client.get(config.upstream.clone() + req.path())
        .send()
        .await?;
    let net_input = reqwest_response.bytes_stream();
    let net_input = net_input
        .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e));
    Ok(try_stream! {
        let mut u8_buffer = Box::new([0u8; 32 * 1024 * 1024]) as Box<[u8]>;
        let mut u32_buffer = Box::new([0u32; 1024 * 1024]) as Box<[u32]>;
        let mut hc_buffer = Box::new([HuffmanCode::default(); 4 * 1024 * 1024]) as Box<[HuffmanCode]>;
        let heap_u8_allocator = HeapPrealloc::<u8>::new_allocator(4096, &mut u8_buffer, |_| {});
        let heap_u32_allocator = HeapPrealloc::<u32>::new_allocator(4096, &mut u32_buffer, |_| {});
        let heap_hc_allocator = HeapPrealloc::<HuffmanCode>::new_allocator(4096, &mut hc_buffer, |_| {});

        let input_buf0 = [0u8; 4096];
        let mut input_buf = input_buf0;
        let output_buf0 = [0u8; 4096];
        let mut output_buf = output_buf0;
        let mut available_in = input_buf0.len();
        let mut available_out = output_buf0.len();
        let mut input_offset = 0;
        let mut output_offset = 0;

        let mut brotli_state = BrotliState::new(heap_u8_allocator, heap_u32_allocator, heap_hc_allocator);

        loop {
            if input_buf.len() == 0 {
                input_buf = input_buf0;
            }
            if output_buf.len() == 0 {
                output_buf = output_buf0;
            }
            if available_in == 0 {
                available_in = input_buf.len();
                input_offset = 0;
            }
            if available_out == 0 {
                available_out = input_buf.len();
                output_offset = 0;
            }
            let mut written = 0;
            let result = BrotliDecompressStream(
                &mut available_in, &mut input_offset, &input_buf,
                &mut available_out, &mut output_offset, &mut output_buf,
                &mut written, &mut brotli_state);
            if written == 0 {
                break;
            }
            yield Bytes::copy_from_slice(&output_buf[.. written]);
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