use crate::Error;
use byteorder::{BigEndian, ByteOrder};
use hyper::{Body, Response as HttpResponse};
use std::convert::{TryFrom, TryInto};
use tonic::metadata::MetadataMap;

pub type GrpcResponse = tonic::Response<Vec<u8>>;

pub struct GrpcWebResponse(Vec<u8>);

impl TryFrom<(Vec<u8>, MetadataMap)> for GrpcWebResponse {
    type Error = Error;

    fn try_from((body, metadata): (Vec<u8>, MetadataMap)) -> Result<Self, Self::Error> {
        let body = copy_trailers_to_payload(body)?;
        let mut body = base64::encode(body);
        body.push_str(&base64::encode(&extract_headers(metadata)?));
        Ok(Self(body.into_bytes()))
    }
}

impl TryFrom<GrpcResponse> for GrpcWebResponse {
    type Error = Error;

    fn try_from(grpc_response: GrpcResponse) -> Result<Self, Self::Error> {
        let body = grpc_response.get_ref().to_owned();
        let metadata = grpc_response.metadata().clone();
        Self::try_from((body, metadata))
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

fn copy_trailers_to_payload(body: Vec<u8>) -> Result<Vec<u8>, Error> {
    let mut trailer: Vec<u8> = vec![0, 0, 0, 0, 1 << 7];
    BigEndian::write_u32(&mut trailer[1..5], body.len().try_into()?);
    Ok([&trailer[..], &body[..]].concat())
}

fn extract_headers(meta: MetadataMap) -> Result<Vec<u8>, Error> {
    let headers = meta.into_headers();

    let body: Vec<u8> = headers
        .into_iter()
        .filter_map(|(key, value)| {
            Some(format!("{}:{}\r\n", key?, value.to_str().ok()?).into_bytes())
        })
        .flatten()
        .collect();

    let mut trailer: Vec<u8> = vec![1 << 7, 0, 0, 0, 0];
    BigEndian::write_u32(&mut trailer[1..5], body.len().try_into()?);
    Ok([&trailer[..], &body[..]].concat())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::AsciiMetadataValue;

    #[test]
    fn should_copy_trailers_to_payload() -> Result<(), Error> {
        assert_eq!(
            copy_trailers_to_payload(Vec::from([0u8; 14]))?,
            vec![0, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        );

        Ok(())
    }

    #[test]
    fn should_extract_headers() -> Result<(), Error> {
        // "content-type": "application/grpc", "date": "Mon, 12 Oct 2020 08:32:05 GMT", "grpc-status": "0"

        let mut meta = MetadataMap::new();
        meta.append(
            "content-type",
            AsciiMetadataValue::from_str("application/grpc").unwrap(),
        );
        meta.append("grpc-status", AsciiMetadataValue::from_str("0").unwrap());

        assert_eq!(
            vec![
                128, 0, 0, 0, 46, 99, 111, 110, 116, 101, 110, 116, 45, 116, 121, 112, 101, 58, 97,
                112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 103, 114, 112, 99, 13, 10, 103,
                114, 112, 99, 45, 115, 116, 97, 116, 117, 115, 58, 48, 13, 10
            ],
            extract_headers(meta)?
        );

        Ok(())
    }
}
