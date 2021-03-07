const { HelloRequest, RepeatHelloRequest, HelloReply } = require('./helloworld_pb.js');
const { GreeterClient } = require('./helloworld_grpc_web_pb.js');

var client = new GreeterClient('http://localhost:8080', null, null);

// simple unary call
var request = new HelloRequest();
request.setName('Tonic');

client.sayHello(request, {}, (err, response) => {
    if (err) {
        console.log(`Unexpected error for sayHello: code = ${err.code}` + `, message = "${err.message}"`);
    } else {
        console.log(response.getMessage());
    }
});


// server streaming call
var streamRequest = new RepeatHelloRequest();
streamRequest.setName('World');
streamRequest.setCount(5);

var stream = client.sayRepeatHello(streamRequest, {});
stream.on('data', (response) => {
    console.log(response.getMessage());
});
stream.on('error', (err) => {
    console.log(`Unexpected stream error: code = ${err.code}` + `, message = "${err.message}"`);
});