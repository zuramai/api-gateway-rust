use serde::{ser::SerializeStruct, Serialize, Serializer};
use hyper::{StatusCode, Response as HyperResponse, Error};
use http_body_util::Full;
use bytes::Bytes;

use crate::error::GatewayError;

pub struct Response {
    status_code: hyper::StatusCode,
    message: String
}
impl Response {
    pub fn new(status_code: StatusCode, message: String) -> Self {
        Response { status_code, message }
    }

    pub fn into_response(self) -> Result<HyperResponse<Full<Bytes>>, GatewayError> {
        let body = serde_json::to_string(&self).unwrap();
        let res = HyperResponse::builder()
            .status(self.status_code)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(body)))
            .unwrap();
        Ok(res)
    }

}

impl Serialize for Response {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Response", 1)?;
        state.serialize_field("message", &self.message)?;
        state.end()
    }
}
