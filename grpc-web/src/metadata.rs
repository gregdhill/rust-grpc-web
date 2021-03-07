use crate::error::Error;
use futures::stream;
use hyper::http::uri::PathAndQuery;
use prost::Message;
use prost_types::{FileDescriptorProto, MethodDescriptorProto};
use std::collections::HashMap;
use tokio_stream::StreamExt;
use tonic::codegen::StdError;
use tonic::transport::Channel;
use tonic::Request as GrpcRequest;

use proto::server_reflection_client::ServerReflectionClient;
use proto::server_reflection_request::MessageRequest;
use proto::server_reflection_response::MessageResponse;
use proto::{ServerReflectionRequest, ServiceResponse};

pub mod proto {
    tonic::include_proto!("grpc.reflection.v1alpha");
}

#[derive(Debug, Clone)]
pub enum ConnectionType {
    Unary,
    ClientStreaming,
    ServerStreaming,
    Streaming,
}

impl From<MethodDescriptorProto> for ConnectionType {
    fn from(method: MethodDescriptorProto) -> Self {
        match method {
            MethodDescriptorProto {
                client_streaming: Some(true),
                server_streaming: None,
                ..
            } => ConnectionType::ClientStreaming,
            MethodDescriptorProto {
                client_streaming: None,
                server_streaming: Some(true),
                ..
            } => ConnectionType::ServerStreaming,
            MethodDescriptorProto {
                client_streaming: Some(true),
                server_streaming: Some(true),
                ..
            } => ConnectionType::Streaming,
            _ => ConnectionType::Unary,
        }
    }
}

#[derive(Clone)]
pub struct Metadata(HashMap<String, HashMap<String, ConnectionType>>);

impl Metadata {
    pub async fn from_reflection_service<D>(dst: D) -> Result<Self, Error>
    where
        D: std::convert::TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let mut ref_client = ServerReflectionClient::connect(dst).await?;
        let services = get_services(&mut ref_client).await?;
        log::info!("Found {} services", services.len());

        let mut metadata = HashMap::new();
        for service in services {
            let methods = get_methods(&mut ref_client, service.name.clone()).await?;
            log::debug!("{:?}: {:?}", service, methods);

            metadata.insert(
                service.name,
                methods
                    .iter()
                    .filter_map(|method| {
                        if let Some(name) = method.name.clone() {
                            Some((name, method.clone().into()))
                        } else {
                            None
                        }
                    })
                    .collect(),
            );
        }

        Ok(Self(metadata))
    }

    pub fn get_query_type(&self, path: PathAndQuery) -> Result<ConnectionType, Error> {
        let parts = path.path().split("/").collect::<Vec<&str>>();
        let parts = parts.get(1..3).ok_or(Error::InvalidQuery)?;
        let connection_type = self
            .0
            .get(parts[0])
            .ok_or(Error::UnknownService)?
            .get(parts[1])
            .ok_or(Error::UnknownMethod)?
            .clone();
        Ok(connection_type)
    }
}

async fn get_services(
    client: &mut ServerReflectionClient<Channel>,
) -> Result<Vec<ServiceResponse>, Error> {
    let request = ServerReflectionRequest {
        host: "".to_string(),
        message_request: Some(MessageRequest::ListServices(String::new())),
    };

    let request = GrpcRequest::new(stream::iter(vec![request]));
    let mut inbound = client.server_reflection_info(request).await?.into_inner();
    let response = inbound
        .next()
        .await
        .ok_or(Error::NoResponse)??
        .message_response
        .ok_or(Error::NoResponse)?;

    if let MessageResponse::ListServicesResponse(services) = response {
        Ok(services.service)
    } else {
        Err(Error::NoServices)
    }
}

async fn get_methods(
    client: &mut ServerReflectionClient<Channel>,
    service_name: String,
) -> Result<Vec<MethodDescriptorProto>, Error> {
    let request = ServerReflectionRequest {
        host: "".to_string(),
        message_request: Some(MessageRequest::FileContainingSymbol(service_name)),
    };
    let request = GrpcRequest::new(stream::iter(vec![request]));
    let mut inbound = client.server_reflection_info(request).await?.into_inner();

    let response = inbound
        .next()
        .await
        .ok_or(Error::NoResponse)??
        .message_response
        .ok_or(Error::NoResponse)?;

    if let MessageResponse::FileDescriptorResponse(descriptor) = response {
        let file_descriptor_proto = descriptor
            .file_descriptor_proto
            .first()
            .expect("descriptor");

        let service = FileDescriptorProto::decode(file_descriptor_proto.as_ref())?.service;
        let service = service.first().expect("service");
        Ok(service.method.clone())
    } else {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_err {
        ($result:expr, $err:pat) => {{
            match $result {
                Err($err) => (),
                Ok(v) => panic!("assertion failed: Ok({:?})", v),
                _ => panic!("expected: Err($err)"),
            }
        }};
    }

    macro_rules! assert_ok {
        ( $x:expr $(,)? ) => {
            let is = $x;
            match is {
                Ok(_) => (),
                _ => assert!(false, "Expected Ok(_). Got {:#?}", is),
            }
        };
        ( $x:expr, $y:expr $(,)? ) => {
            assert_eq!($x, Ok($y));
        };
    }

    #[tokio::test]
    async fn should_get_query_type() -> Result<(), Error> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "service".to_string(),
            vec![("method".to_string(), ConnectionType::Unary)]
                .into_iter()
                .collect(),
        );

        let metadata = Metadata(metadata);
        assert_ok!(metadata.get_query_type(PathAndQuery::from_static("/service/method")));
        assert_err!(
            metadata.get_query_type(PathAndQuery::from_static("/unknown/method")),
            Error::UnknownService
        );
        assert_err!(
            metadata.get_query_type(PathAndQuery::from_static("/service/unknown")),
            Error::UnknownMethod
        );

        Ok(())
    }
}
