use std::{future::Future, ops::Deref, pin::Pin};
use bytes::{Buf, Bytes};
use hyper::{body::{Body, Incoming}, service::Service, Request, StatusCode};
use http_body_util::Full;
use reqwest::{Client};
use std::convert::TryInto;


use crate::{config::{self, ServiceConfig}, error, response};

pub struct GatewayService {
    config: config::GatewayConfig
}

impl GatewayService {
    pub fn new(config: config::GatewayConfig) -> GatewayService {
        GatewayService {
            config
        }
    } 

    fn not_found(&self) -> response::Response {
        response::Response::new(StatusCode::NOT_FOUND, String::from("Not found"))
    }

    pub async fn forward_request(mut req: Request<hyper::body::Incoming>) -> Result<hyper::Response<bytes::Bytes>, error::GatewayError> {
        let client = Client::new();

        let mut builder = client.request(req.method().clone(), req.uri().to_string());
        
        for (key, value) in req.headers().iter() {
            builder = builder.header(key, value);
        }
        let bytes = http_body_util::BodyExt::collect(req.body_mut()).await.map_err(|_| error::GatewayError::gateway_error())?;


        let response = builder.body(bytes.to_bytes()).send().await.map_err(|_| error::GatewayError::gateway_error())?;
        let response_status = response.status();
        let response_headers = response.headers().clone();
        let bytes = response.bytes().await.map_err(|_| error::GatewayError::gateway_error())?;
        // Convert reqwest Response back to hyper Response
        let mut hyper_response = hyper::Response::builder()
            .status(response_status)
            .body(bytes)
            .map_err(|_| error::GatewayError::gateway_error())?;

        // Copy headers from the reqwest response back to the hyper response
        for (key, value) in response_headers {
            hyper_response.headers_mut().insert(key.unwrap(), value.clone());
        }
        Ok(hyper_response)
    }
}


impl Service<Request<Incoming>> for GatewayService {   
    type Error = hyper::Error;
    type Response = hyper::Response<Full<Bytes>>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;


    fn call(&self, req: Request<Incoming>) -> Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>> {
        let path = req.uri().path();
        let service_config: &ServiceConfig;
        let render_response = |response: response::Response| -> Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>> {
            Box::pin(async {response.into_response()})
        };
        match self.config.get_service_config(path) {
            Some(service) => service_config =service,
            None => return Box::pin(async {self.not_found().into_response()})
        };


        Box::pin(async {
            let response = GatewayService::forward_request(req).await.unwrap();
            response
        })
    }
} 