var PROTO_PATH = __dirname + '/helloworld.proto';

var assert = require('assert');
var async = require('async');
var _ = require('lodash');
var grpc = require('@grpc/grpc-js');
var protoLoader = require('@grpc/proto-loader');
var packageDefinition = protoLoader.loadSync(
    PROTO_PATH,
    {keepCase: true,
     longs: String,
     enums: String,
     defaults: true,
     oneofs: true
    });
var protoDescriptor = grpc.loadPackageDefinition(packageDefinition);
var helloworld = protoDescriptor.helloworld;

/**
 * @param {!Object} call
 * @param {function():?} callback
 */
function doSayHello(call, callback) {
  callback(null, {message: `Hello ${call.request.name}!`});
}

/**
 * @param {!Object} call
 */
function doSayRepeatHello(call) {
  var senders = [];
  function sender(name) {
    return (callback) => {
      call.write({
        message: 'Hey! ' + name
      });
      _.delay(callback, 500); // in ms
    };
  }
  for (var i = 0; i < call.request.count; i++) {
    senders[i] = sender(call.request.name + i);
  }
  async.series(senders, () => {
    call.end();
  });
}

/**
 * @return {!Object} gRPC server
 */
function getServer() {
  var server = new grpc.Server();
  server.addService(helloworld.Greeter.service, {
    sayHello: doSayHello,
    sayRepeatHello: doSayRepeatHello,
  });
  return server;
}

if (require.main === module) {
  var server = getServer();
  server.bindAsync(
    '0.0.0.0:9090', grpc.ServerCredentials.createInsecure(), (err, port) => {
      assert.ifError(err);
      server.start();
  });
}

exports.getServer = getServer;