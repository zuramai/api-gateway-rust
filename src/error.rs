use std::error::Error;
use std::fmt;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum GatewayError {
    NotFound(String),
    GatewayError(String)                                                                                                  
}

impl GatewayError {
    pub fn not_found() -> Self {
        GatewayError::NotFound("Not found".into())
    }
    pub fn gateway_error() -> Self {
        GatewayError::GatewayError("Internal server error".into())
    }
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatewayError::NotFound(ref msg) => write!(f, "URL not found"),
            GatewayError::GatewayError(ref msg) => write!(f, "Internal server error")
        }
    }
}

impl Error for GatewayError {}