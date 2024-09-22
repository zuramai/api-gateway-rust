use std::{future::Future, ops::Deref, pin::Pin};
use bytes::{Buf, Bytes};
use hyper::{body::{Body, Incoming}, header, service::Service, Request, Response, StatusCode};
use http_body_util::Full;
use reqwest::{Client};
use std::convert::TryInto;


use crate::{config::{self, ServiceConfig}, error::{self, GatewayError}, response};

pub struct GatewayService {
    config: config::GatewayConfig
}

impl GatewayService {
    pub fn new(config: config::GatewayConfig) -> GatewayService {
        GatewayService {
            config
        }
    } 


    pub async fn forward_request(mut req: Request<hyper::body::Incoming>, service_config: &ServiceConfig) -> Result<hyper::Response<Full<Bytes>>, GatewayError> {
        let client = Client::new();

        let mut builder = client.request(req.method().clone(), service_config.get_full_url());
        
        for (key, value) in req.headers().iter() {
            builder = builder.header(key, value);
        }
        let bytes = http_body_util::BodyExt::collect(req.body_mut()).await.map_err(|_| error::GatewayError::GatewayError)?;


        let response = builder.body(bytes.to_bytes()).send().await.map_err(|_| error::GatewayError::GatewayError)?;
        let response_status = response.status();
        let response_headers = response.headers().clone();
        let bytes = response.bytes().await.map_err(|_| error::GatewayError::GatewayError)?;
        // Convert reqwest Response back to hyper Response
        let mut hyper_response = hyper::Response::builder()
            .status(response_status)
            .body(Full::from(bytes))
            .map_err(|_| error::GatewayError::GatewayError)?;

        // Copy headers from the reqwest response back to the hyper response
        for (key, value) in response_headers {
            hyper_response.headers_mut().insert(key.unwrap(), value.clone());
        }
        Ok(hyper_response)
    }

    pub async fn health_check() -> response::Response {
        let res = response::Response::new(StatusCode::OK, "service healthy".into());
        res
    }
}


impl Service<Request<Incoming>> for GatewayService {   
    type Error = GatewayError;
    type Response = hyper::Response<Full<Bytes>>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;


    fn call(&self, req: Request<Incoming>) -> Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>> {
        let path = req.uri().path().to_owned();
        let config = self.config.to_owned();
        Box::pin(async move {
            match config.get_service_config(path.to_owned()) {
                Some(service) => {
                    let response = GatewayService::forward_request(req, &service).await;

                    if let Err(err) = response {
                        return err.into_response();
                    }
                    response 
                },
                None => return {
                    match path.as_str() {
                        "/" => GatewayService::health_check().await.into_response(),
                        _ => {
                            let err = GatewayError::NotFound;
                            err.into_response()
                        }
                    }
                }
            }
        })
    }
} 