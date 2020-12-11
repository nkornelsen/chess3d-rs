use chess3d::{ Board, Move };
use serde::{ Serialize, Deserialize };
use std::net::TcpStream;
use serde_json;
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    BoardUpdate {
        board: Board,
    },
    PlayerMove {
        r#move: Move,
    }
}

pub fn emit_message(stream: &mut TcpStream, message: &ServerMessage) {
    let data = serde_json::to_string(message).unwrap();
    let len = data.as_bytes().len() as u32;
    stream.write(&u32::to_be_bytes(len)).unwrap();
    stream.write(data.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub fn recv_message(stream: &mut TcpStream) -> Result<ServerMessage, std::io::Error> {
    let mut len_buffer = [0; 4];
    stream.read(&mut len_buffer)?;
    let len = u32::from_be_bytes(len_buffer) as usize;
    
    let mut data: Vec<u8> = Vec::with_capacity(len as usize);
    data.resize(len, 0);
    stream.read_exact(&mut data).unwrap();

    Ok(serde_json::from_slice::<ServerMessage>(&data).unwrap())
}