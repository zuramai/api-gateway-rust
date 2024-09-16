use std::{env, net::SocketAddr};

use config::{load_config, GatewayConfig};
use service::GatewayService;
use tokio::net::TcpListener;
use hyper::{body::Body, server::conn::{http1, http2}, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use hyper::service::Service;
use hyper::{body::Incoming as IncomingBody};
use std::convert::Infallible;

pub mod config;
pub mod service;
pub mod response;
pub mod error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).unwrap_or("config.yaml");
    let config = load_config(filename);

    let gateway_service = GatewayService::new(config);

    // let addr: SocketAddr = ([127,0,0,1], 3000).into();
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let config_clone = config.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, gateway_service).await {
                println!("Failed to serve connection {:?}", err)
            }
        });
    }
}
