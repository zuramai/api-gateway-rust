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