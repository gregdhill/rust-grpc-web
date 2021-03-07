use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tokio::sync::mpsc;
use std::pin::Pin;
use futures::Stream;

mod proto {
    tonic::include_proto!("helloworld");

    pub(crate) const FILE_DESCRIPTOR_SET: &'static [u8] =
        tonic::include_file_descriptor_set!("helloworld_descriptor");
}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl proto::greeter_server::Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<proto::HelloRequest>,
    ) -> Result<Response<proto::HelloReply>, Status> {
        println!("SayHello = {:?}", request);

        let reply = proto::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }

    type SayRepeatHelloStream =
        Pin<Box<dyn Stream<Item = Result<proto::HelloReply, Status>> + Send + Sync + 'static>>;

    async fn say_repeat_hello(
        &self,
        request: Request<proto::RepeatHelloRequest>,
    ) -> Result<Response<Self::SayRepeatHelloStream>, Status> {
        println!("SayRepeatHello = {:?}", request);

        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            let request = request.into_inner();
            for _ in 0..request.count {
                let reply = proto::HelloReply {
                    message: format!("Hello {}!", request.name).into(),
                };
                println!("  => send {:?}", reply);
                tx.send(Ok(reply)).await.unwrap();
            }

            println!(" /// done sending");
        });


        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}

// fn intercept(req: Request<()>) -> Result<Request<()>, Status> {
//     println!("Intercepting request: {:?}", req);
//     Ok(req)
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let addr = "[::1]:50052".parse().unwrap();
    let greeter = MyGreeter::default();

    // let greeter = GreeterServer::with_interceptor(greeter, intercept);

    Server::builder()
        .add_service(service)
        .add_service(proto::greeter_server::GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}