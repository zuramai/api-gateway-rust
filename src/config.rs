use std::{fs::File, io::Read};

use hyper::{body::{Body, Incoming}, service::Service, Request, Response};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use hyper::http::request::Parts;
use tokio::net::TcpStream;

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub path: String,
    pub target: String,
    pub target_port: String,
    pub append_path: bool,
}


#[derive(Debug, Deserialize, Clone)]
pub struct GatewayConfig {
    pub name: String,
    pub services: Vec<ServiceConfig>,
}

impl GatewayConfig {
    pub fn get_service_config(&self, path: &str) -> Option<&ServiceConfig> {
        self.services.iter().find(|s| s.path == path)
    }

}

pub fn load_config(path: &str) -> GatewayConfig {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    serde_yaml::from_str(&contents).unwrap()
}


impl ServiceConfig {
    pub fn build_downstream_request(&self, parts: Parts, body: hyper::body::Incoming) -> Result<Request<Body>, hyper::http::Error> {
        let req = Request::from_parts(parts, body);
        let uri = format!(
            "{}:{}{}",
            self.target,
            self.target_port,
            req.uri().path()
        );

        let mut downstream_req_builder = Request::builder()
            .uri(uri)
            .method(req.method())
            .version(req.version());

        *downstream_req_builder.headers_mut().unwrap() = req.headers().clone();

        let downstream_req = downstream_req_builder.body(body);

        downstream_req
    }


    pub async fn send_request(&self, req: Request<Incoming>) -> Result<Response<Incoming>, hyper::Error> {
        let addr = format!("{}:{}", self.target, self.target_port);
        let stream = TcpStream::connect(addr).await.unwrap();
        let io = TokioIo::new(stream);

        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });
        let res = sender.send_request(req).await?;

        Ok(res)
    }
}