use byteorder::{BigEndian, ByteOrder};
use hyper::{Body, Response as HttpResponse};
use std::convert::TryInto;
use tonic::metadata::MetadataMap;

pub type GrpcResponse = tonic::Response<Vec<u8>>;

pub struct GrpcWebResponse(Vec<u8>);

impl From<(Vec<u8>, MetadataMap)> for GrpcWebResponse {
    fn from((body, metadata): (Vec<u8>, MetadataMap)) -> Self {
        let body = copy_trailers_to_payload(body);
        let mut body = base64::encode(body);
        body.push_str(&base64::encode(&extract_headers(metadata)));
        Self(body.into_bytes())
    }
}

impl From<GrpcResponse> for GrpcWebResponse {
    fn from(grpc_response: GrpcResponse) -> Self {
        let body = grpc_response.get_ref().to_owned();
        let metadata = grpc_response.metadata().clone();
        Self::from((body, metadata))
    }
}

impl Into<Vec<u8>> for GrpcWebResponse {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl Into<HttpResponse<Body>> for GrpcWebResponse {
    fn into(self) -> HttpResponse<Body> {
        HttpResponse::new(self.0.into())
    }
}

fn copy_trailers_to_payload(body: Vec<u8>) -> Vec<u8> {
    let mut trailer: Vec<u8> = vec![0, 0, 0, 0, 1 << 7];
    BigEndian::write_u32(&mut trailer[1..5], body.len().try_into().unwrap());
    [&trailer[..], &body[..]].concat()
}

fn extract_headers(meta: MetadataMap) -> Vec<u8> {
    let headers = meta.into_headers();

    let body: Vec<u8> = headers
        .into_iter()
        .map(|(key, value)| {
            format!("{}:{}\r\n", key.unwrap(), value.to_str().unwrap()).into_bytes()
        })
        .flatten()
        .collect();

    let mut trailer: Vec<u8> = vec![1 << 7, 0, 0, 0, 0];
    BigEndian::write_u32(&mut trailer[1..5], body.len().try_into().unwrap());
    [&trailer[..], &body[..]].concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_is_correct() {
        println!(
            "{:?}",
            base64::decode("AAAAAA4KDEhlbGxvIFRvbmljIQ==").unwrap()
        );

        assert_eq!(
            copy_trailers_to_payload(Vec::from([0u8; 14])),
            vec![0, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        );
    }

    #[test]
    fn should_extract_headers() {
        let mut meta = MetadataMap::new();
        // "content-type": "application/grpc", "date": "Mon, 12 Oct 2020 08:32:05 GMT", "grpc-status": "0"
        meta.append(
            "content-type",
            AsciiMetadataValue::from_str("application/grpc").unwrap(),
        );
        meta.append("grpc-status", AsciiMetadataValue::from_str("0").unwrap());
        println!("{:?}", String::from_utf8_lossy(&extract_headers(meta)));
    }
}
