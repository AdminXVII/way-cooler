//! Contains code to handle an IPC channel which is issuing commands.

use std::io::Error as IOError;
use std::io::prelude::*;

use rustc_serialize::Decodable;
use rustc_serialize::json::{Json, ToJson, Decoder, ParserError, EncoderError};

use unix_socket::UnixStream;

/// Reasons a client message might be erroneous
#[derive(Debug)]
enum ResponseError {
    /// Connection was closed
    ConnectionClosed,
    /// Some bytes dun goofed
    InvalidString,
    /// There were IO issues
    IO(IOError),
    /// Json was invalid
    InvalidJson(ParserError)
}

/// Receives a packet from the given stream.
fn receive_packet(stream: &mut Read) -> Result<Json, ResponseError> {
    let mut buffer = [0u8; 4];
    try!(stream.read_exact(&mut buffer).map_err(ResponseError::IO));
    // This is what the byteorder crate does (needs testing)
    let len = u32::from_be(unsafe { buffer.as_ptr() as *const u32 } as u32);
    trace!("Listening for packet of length {}", len);
    return Json::from_reader(&mut stream.take(len as u64))
        .map_err(ResponseError::InvalidJson);
}

pub fn handle_client(mut stream: UnixStream) {
    println!("Starting connection.");

    // Listen for starting connection
}

fn command(mut stream: UnixStream) {
    
}

fn event(mut stream: UnixStream) {
    
}
