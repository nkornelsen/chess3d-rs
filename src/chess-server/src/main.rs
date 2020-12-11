use std::net::{ TcpListener, TcpStream };
use std::io::prelude::*;
use serde_json;
use crossbeam::thread;
use std::sync::{Arc, Mutex};

use chess3d::Board;
use chess3d_common::ServerMessage;

pub struct ServerState {
    board: Board,
    connections: Vec<TcpStream>,
}

impl ServerState {
    fn broadcast_all(&mut self, message: &ServerMessage) {
        for stream in &mut self.connections {
            println!("Sent message");
            chess3d_common::emit_message(stream, message);
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    let state = Arc::new(Mutex::new(ServerState {
        board: Board::new(),
        connections: Vec::new()
    }));

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, state.clone());
    }
}

fn handle_connection(s: TcpStream, state: Arc<Mutex<ServerState>>) {
    println!("Connection Received");
    use std::thread;
    let mut stream = s.try_clone().unwrap();
    { state.lock().unwrap().connections.push(s) };

    thread::spawn(move || {
        chess3d_common::emit_message(&mut stream, &ServerMessage::BoardUpdate {
            board: state.lock().unwrap().board,
        });
    
        let mut running = { state.lock().unwrap().board.is_running() };
        while running {
            let data = chess3d_common::recv_message(&mut stream);
            if let Ok(message) = data {
                match message {
                    ServerMessage::PlayerMove { r#move } => {
                        let s = &mut *state.lock().unwrap();
                        // s.execute_move(&r#move);
                        s.board.execute_move(&r#move);
                        println!("Executing move: {:?}", r#move);
                        s.broadcast_all(&ServerMessage::BoardUpdate { board: s.board });
                    },
                    _ => {}
                }
            } else {
                break;
            }
            running = state.lock().unwrap().board.is_running();
        }
    });

    println!("handle_connection finished");
}
