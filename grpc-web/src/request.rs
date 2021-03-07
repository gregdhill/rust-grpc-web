use crate::Error;
use hyper::{Body, Request as HttpRequest};
use std::convert::TryInto;
use tonic::Request;

pub type GrpcRequest = Request<Vec<u8>>;

#[derive(Debug)]
pub struct GrpcWebRequest(Vec<u8>);

impl GrpcWebRequest {
    pub async fn from_http_request(mut req: HttpRequest<Body>) -> Result<Self, Error> {
        let body = hyper::body::to_bytes(req.body_mut()).await?;
        Ok(Self(base64::decode(body)?))
    }
}

impl TryInto<GrpcRequest> for GrpcWebRequest {
    type Error = Error;

    fn try_into(self) -> Result<GrpcRequest, Self::Error> {
        Ok(GrpcRequest::new(
            self.0.get(5..).ok_or(Error::InvalidRequest)?.to_vec(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_decode_http_request() -> Result<(), Error> {
        let http_request = HttpRequest::<Body>::new(b"AAAAAAcKBVRvbmlj".to_vec().into());
        assert_eq!(
            GrpcWebRequest::from_http_request(http_request).await?.0,
            vec![0, 0, 0, 0, 7, 10, 5, 84, 111, 110, 105, 99]
        );

        Ok(())
    }
}
