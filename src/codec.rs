use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use bytes::BytesMut;
use log::*;
use std::io::Cursor;
use tokio_util::codec::{Decoder, Encoder};

use crate::error::{Error, Result};

// [u16][u64][u8...]
#[derive(Debug)]
pub struct Message {
    pub port: u16,
    pub size: u64,
    pub payload: BytesMut, // [u64][u8...]
}

impl Message {
    pub fn new(port: u16, payload: &[u8]) -> Self {
        Self {
            port,
            size: payload.len() as u64,
            payload: payload.into(),
        }
    }
}

pub struct Codec {
    msg: Option<Message>,
}

impl Codec {
    pub fn new() -> Self {
        Self { msg: None }
    }
}

impl Decoder for Codec {
    type Item = Message;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        if self.msg.is_none() {
            if src.len() < 10 {
                trace!("Buffer length is not enough {} < 10", src.len());
                return Ok(None);
            }

            let header = src.split_to(10);
            let mut rdr = Cursor::new(header);
            let port = ReadBytesExt::read_u16::<LittleEndian>(&mut rdr)?;
            let size = ReadBytesExt::read_u64::<LittleEndian>(&mut rdr)?;
            let msg = Message {
                port,
                size,
                payload: BytesMut::new(),
            };

            trace!("Received header {:?}", self.msg);
            self.msg = Some(msg);
        }

        if let Some(mut msg) = self.msg.take() {
            let size = msg.size as usize;
            if src.len() < size {
                trace!("Paylaod length is not enough {} < {}", src.len(), size);
                self.msg = Some(msg);
                Ok(None)
            } else {
                msg.payload = src.split_to(size);
                trace!("Received message {:?}", msg);
                Ok(Some(msg))
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder for Codec {
    type Item = Message;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<()> {
        dst.resize(10, 0);
        {
            let mut dst = dst as &mut [u8];
            dst.write_u16::<LittleEndian>(item.port)?;
            dst.write_u64::<LittleEndian>(item.size)?;
        }
        dst.extend_from_slice(&item.payload);
        trace!("Sending {:#x?}", dst);
        Ok(())
    }
}
