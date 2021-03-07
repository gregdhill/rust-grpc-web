use crate::Error;
use hyper::{Body, Request as HttpRequest};
use tonic::Request;

pub type GrpcRequest = Request<Vec<u8>>;

pub struct GrpcWebRequest(Vec<u8>);

impl GrpcWebRequest {
    pub async fn from_http_request(mut req: HttpRequest<Body>) -> Result<Self, Error> {
        let body = hyper::body::to_bytes(req.body_mut()).await?;
        Ok(Self(base64::decode(body)?))
    }
}

impl Into<GrpcRequest> for GrpcWebRequest {
    fn into(self) -> GrpcRequest {
        GrpcRequest::new(self.0[5..].to_vec())
    }
}
