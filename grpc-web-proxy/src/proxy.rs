use futures::stream::StreamExt;
use grpc_web::{
    ConnectionType, Error, GrpcRequest, GrpcWebRequest, GrpcWebResponse, Metadata, ProxyCodec,
};
use hyper::{
    http::{
        header::{ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE},
        HeaderValue, Method,
    },
    Body, Request as HttpRequest, Response as HttpResponse,
};
use tonic::codegen::StdError;
use tonic::transport::Channel;
use tonic::transport::Endpoint;
use tonic::{client::Grpc as GrpcClient, Status};

#[allow(dead_code)]
pub const GRPC_CONTENT_TYPE: &str = "application/grpc";
#[allow(dead_code)]
pub const GRPC_WEB_CONTENT_TYPE: &str = "application/grpc-web";
pub const GRPC_WEB_TEXT_CONTENT_TYPE: &str = "application/grpc-web-text";
pub const GRPC_WEB_TEXT_CONTENT_TYPE_PROTO: &str = "application/grpc-web-text+proto";

fn is_gprc_web_request(req: &HttpRequest<Body>) -> bool {
    req.headers()
        .get("content-type")
        .map_or(false, |content_type| {
            content_type == GRPC_WEB_TEXT_CONTENT_TYPE
        })
}

#[derive(Clone)]
pub(crate) struct HttpConfig {
    pub allowed_cors_domains: HeaderValue,
    pub allowed_cors_headers: HeaderValue,
}

impl HttpConfig {
    fn add_default_headers(&self, http_response: &mut HttpResponse<Body>) {
        let header_map = http_response.headers_mut();
        header_map.insert(
            ACCESS_CONTROL_ALLOW_ORIGIN,
            self.allowed_cors_domains.clone(),
        );
        header_map.insert(
            ACCESS_CONTROL_ALLOW_HEADERS,
            self.allowed_cors_headers.clone(),
        );
    }
}

fn add_content_type(http_response: &mut HttpResponse<Body>) {
    // TODO: may not be proto
    let header_map = http_response.headers_mut();
    header_map.insert(
        CONTENT_TYPE,
        HeaderValue::from_static(GRPC_WEB_TEXT_CONTENT_TYPE_PROTO),
    );
}

#[derive(Clone)]
pub(crate) struct Proxy {
    client: GrpcClient<Channel>,
    metadata: Metadata,
    config: HttpConfig,
}

impl Proxy {
    pub async fn new<D>(dst: D, config: HttpConfig) -> Result<Self, Error>
    where
        D: Clone,
        D: std::convert::TryInto<Endpoint>,
        D::Error: Into<StdError>,
    {
        let metadata = Metadata::from_reflection_service(dst.clone()).await?;
        let channel = Endpoint::new(dst)?.connect().await?;
        let client = GrpcClient::new(channel);
        Ok(Self {
            client,
            metadata,
            config,
        })
    }

    async fn forward_http_request(
        &mut self,
        http_request: HttpRequest<Body>,
    ) -> Result<HttpResponse<Body>, Error> {
        self.client.ready().await?;

        let path = http_request
            .uri()
            .path_and_query()
            .ok_or(Error::InvalidRequest)?
            .to_owned();
        let codec = ProxyCodec::default();

        log::info!("Forwarding http request: {:?}", http_request);
        let grpc_request: GrpcRequest = GrpcWebRequest::from_http_request(http_request)
            .await?
            .into();

        // TODO: support client streaming and bi-directional streaming
        match self.metadata.get_query_type(path.clone())? {
            ConnectionType::Unary => {
                let grpc_response = self.client.unary(grpc_request, path, codec).await?;
                let grpc_web_response = GrpcWebResponse::from(grpc_response);

                let mut http_response: HttpResponse<Body> = grpc_web_response.into();
                self.config.add_default_headers(&mut http_response);
                add_content_type(&mut http_response);

                Ok(http_response)
            }
            ConnectionType::ServerStreaming => {
                let grpc_response = self
                    .client
                    .server_streaming(grpc_request, path, codec)
                    .await?;
                let metadata = grpc_response.metadata().clone();
                let streaming = grpc_response.into_inner();

                let mut http_response = HttpResponse::new(Body::empty());
                self.config.add_default_headers(&mut http_response);
                add_content_type(&mut http_response);

                *http_response.body_mut() =
                    Body::wrap_stream(streaming.map::<Result<Vec<u8>, Status>, _>(move |result| {
                        let grpc_web_response = GrpcWebResponse::from((result?, metadata.clone()));
                        Ok(grpc_web_response.into())
                    }));

                Ok(http_response)
            }
            _ => Err(Error::InvalidRequest),
        }
    }

    pub async fn handle_http_request(
        &mut self,
        http_request: HttpRequest<Body>,
    ) -> Result<HttpResponse<Body>, Error> {
        match *http_request.method() {
            Method::OPTIONS => {
                let mut http_response = HttpResponse::new(Body::empty());
                self.config.add_default_headers(&mut http_response);
                Ok(http_response)
            }
            Method::POST => {
                if is_gprc_web_request(&http_request) {
                    self.forward_http_request(http_request).await
                } else {
                    Err(Error::InvalidRequest)
                }
            }
            _ => Err(Error::InvalidRequest),
        }
    }
}
