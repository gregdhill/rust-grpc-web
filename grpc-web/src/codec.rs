use bytes::{Buf, BufMut};
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};

#[derive(Debug, Clone, Default)]
pub struct ProxyEncoder;

impl Encoder for ProxyEncoder {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        buf.put(&item[..]);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProxyDecoder;

impl Decoder for ProxyDecoder {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let mut out = vec![0u8; buf.remaining()];
        buf.copy_to_slice(&mut out);
        Ok(Some(out))
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProxyCodec;

impl Codec for ProxyCodec {
    type Encode = Vec<u8>;
    type Decode = Vec<u8>;

    type Encoder = ProxyEncoder;
    type Decoder = ProxyDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        ProxyEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        ProxyDecoder
    }
}
