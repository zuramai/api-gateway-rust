use thiserror::Error;

use bytes::Bytes;
use http_body_util::Full;
use hyper::{StatusCode};
use serde::Serialize;

use crate::response::Response;

#[derive(Debug, Serialize, Error)]
pub enum GatewayError {
    #[error("not found")]
    NotFound,
    #[error("internal server error")]
    GatewayError,
}

impl GatewayError {
    pub fn into_response(self) -> Result<hyper::Response<Full<Bytes>>, GatewayError> {
        let body = serde_json::to_string(&self).unwrap();
        let code = match self {
            GatewayError::GatewayError => StatusCode::INTERNAL_SERVER_ERROR,
            GatewayError::NotFound => StatusCode::NOT_FOUND,
        };

        let res = Response::new(code, self.to_string());
        res.into_response()
    }
}
