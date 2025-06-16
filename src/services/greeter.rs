// gRPC Greeter service implementation

use tonic::{Request, Response, Status};
use log::info;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

use hello_world::greeter_server::Greeter;
use hello_world::{HelloReply, HelloRequest};

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        info!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hello_world::HelloRequest;
    use tonic::Request;

    #[tokio::test]
    async fn test_say_hello() {
        let greeter = MyGreeter::default();
        let request = Request::new(HelloRequest {
            name: "TestUser".into(),
        });
        let response = greeter.say_hello(request).await.unwrap();
        assert_eq!(response.get_ref().message, "Hello TestUser!");
    }
}
