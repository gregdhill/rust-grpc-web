OS := $(shell uname)
ROOT_DIR := $(shell pwd)
PROTO_INC := $(ROOT_DIR)/examples/helloworld/proto
PROTO_SRC := helloworld.proto
GRPC_OUT := $(ROOT_DIR)/examples/helloworld/js

helloworld:
	protoc -I=${PROTO_INC} ${PROTO_SRC} --js_out=import_style=commonjs:${GRPC_OUT}
	protoc -I=${PROTO_INC} ${PROTO_SRC} --grpc-web_out=import_style=commonjs,mode=grpcwebtext:${GRPC_OUT}
	cd ${GRPC_OUT} && npx webpack client.js

proxy:
	cargo build --bin grpc-web-proxy