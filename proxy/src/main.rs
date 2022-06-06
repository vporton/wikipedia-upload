use std::env;
use std::fmt::{Display, Formatter};
use std::io::{ErrorKind};
use actix_web::{get, App, HttpServer, Responder, HttpRequest, ResponseError, HttpResponse};
use actix_web::http::header::ContentType;
use actix_web::web::{Bytes, Data};
use clap::Parser;
use async_stream::try_stream;
use futures::{Stream, TryStreamExt};
use futures::stream::StreamExt;
use alloc_stdlib::heap_alloc::HeapPrealloc; // TODO: Should use stdlib.
use brotli_decompressor::{BrotliDecompressStream, BrotliState};
use brotli_decompressor::reader::HuffmanCode;
use brotli_decompressor::reader::BrotliResult;
extern crate alloc_stdlib;

#[derive(Debug)]
struct BrotliDecodeError;

impl BrotliDecodeError {
    pub fn new() -> Self {
        Self { }
    }
}

impl Display for BrotliDecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error decoding Brotli.")
    }
}

#[derive(Debug)]
enum MyError {
    Reqwest(reqwest::Error),
    IO(std::io::Error),
    Actix(actix_web::Error),
    BrotliDecode(BrotliDecodeError),
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
            Self::BrotliDecode(err) => write!(f, "Brotli error: {err}"),
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

impl From<BrotliDecodeError> for MyError {
    fn from(value: BrotliDecodeError) -> Self {
        Self::BrotliDecode(value)
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

// brotli_decompressor::declare_stack_allocator_struct!(MemPool, heap);

async fn proxy_get_stream(req: HttpRequest, config: &Config) -> Result<impl Stream<Item = Result<Bytes, MyError>>, MyError> {
    let client = reqwest::Client::new();
    let reqwest_response = client.get(config.upstream.clone() + req.path())
        .send()
        .await?;
    let net_input = reqwest_response.bytes_stream();
    let mut net_input = net_input
        .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e));
    Ok(try_stream! {
        // TODO: Make them global variables?
        let mut u8_buffer = Box::new([0u8; 32 * 1024 * 1024]) as Box<[u8]>;
        let mut u32_buffer = Box::new([0u32; 1024 * 1024]) as Box<[u32]>;
        let mut hc_buffer = Box::new([HuffmanCode::default(); 4 * 1024 * 1024]) as Box<[HuffmanCode]>;
        let heap_u8_allocator = HeapPrealloc::<u8>::new_allocator(4096, &mut u8_buffer, |_| {});
        let heap_u32_allocator = HeapPrealloc::<u32>::new_allocator(4096, &mut u32_buffer, |_| {});
        let heap_hc_allocator = HeapPrealloc::<HuffmanCode>::new_allocator(4096, &mut hc_buffer, |_| {});

        let mut brotli_state = BrotliState::new(heap_u8_allocator, heap_u32_allocator, heap_hc_allocator);

        let output_buf0 = [0u8; 4096];
        let mut output_buf = output_buf0;
        let mut input_buf = Vec::new(); // should be `Bytes` instead?
        let mut available_in = 0;
        let mut available_out = output_buf.len();
        let mut input_offset = 0;
        let mut output_offset = 0;

        loop {
            if available_in == 0 {
                let piece = net_input.next().await;
                available_in = if let Some(piece) = piece {
                    input_buf = piece?.as_ref().to_vec(); // probably not efficient
                    input_buf.len()
                } else {
                    0
                };
                input_offset = 0;
            }
            if available_out == 0 {
                available_out = output_buf.len();
                output_offset = 0;
            }
            let mut written = 0;
            let old_output_offset = output_offset;
            output_buf = output_buf0; // It is essentially a constant.
            let result = BrotliDecompressStream(
                &mut available_in, &mut input_offset, &input_buf,
                &mut available_out, &mut output_offset, &mut output_buf,
                &mut written, &mut brotli_state);
            match result {
                BrotliResult::ResultFailure => return Err::<(), MyError>(BrotliDecodeError::new().into())?,
                _ => {},
            }
            if old_output_offset != output_offset {
                yield Bytes::copy_from_slice(&output_buf[old_output_offset .. output_offset]);
            }
            match result {
                BrotliResult::ResultSuccess => break,
                _ => {}
            }
        }
    })
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> Result<(), MyError> {
    env::set_var("RUST_MIN_STACK", "1000000000"); // With default 2MiB /proxy/* requests overflow stack

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