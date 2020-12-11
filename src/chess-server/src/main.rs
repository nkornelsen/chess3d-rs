use std::net::{ TcpListener, TcpStream };
use std::sync::{Arc, Mutex};

use chess3d::Board;
use chess3d_common::ServerMessage;

struct Player {
    con: TcpStream,
    id: usize,
}

impl Player {
    fn new(stream: TcpStream, id: usize) -> Player {
        Player {
            con: stream,
            id: id,
        }
    }
}

struct ServerState {
    board: Board,
    players: Vec<Player>
}

impl ServerState {
    fn broadcast_all(&mut self, message: &ServerMessage) {
        for player in &mut self.players {
            println!("Sent message");
            chess3d_common::emit_message(&mut player.con, message);
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    let state = Arc::new(Mutex::new(ServerState {
        board: Board::new(),
        players: Vec::new()
    }));

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, state.clone());
    }
}

fn handle_connection(s: TcpStream, state: Arc<Mutex<ServerState>>) {
    println!("Connection Received");
    use std::thread;
    let stream = s.try_clone().unwrap();
    let id = { state.lock().unwrap().players.len() };
    { state.lock().unwrap().players.push(Player::new(s, id)) };

    let mut player = Player::new(stream, id);
    thread::spawn(move || {
        chess3d_common::emit_message(&mut player.con, &ServerMessage::BoardUpdate {
            board: state.lock().unwrap().board,
        });
    
        let mut running = { state.lock().unwrap().board.is_running() };
        while running {
            let data = chess3d_common::recv_message(&mut player.con);
            if let Ok(message) = data {
                match message {
                    ServerMessage::PlayerMove { r#move } => {
                        let s = &mut *state.lock().unwrap();
                        // s.execute_move(&r#move);
                        let piece = {
                            state.lock().unwrap().board.at(r#move.from())
                        };
                        match piece {
                            chess3d::BoardState::Piece((chess3d::Colors::White, _)) => {
                                if player.id == 0 {
                                    s.board.execute_move(&r#move);
                                }
                            },
                            chess3d::BoardState::Piece((chess3d::Colors::Black, _)) => {
                                if player.id == 1 {
                                    s.board.execute_move(&r#move);
                                }
                            },
                            _ => {}
                        }
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
        println!("Ended connection");
    });

    println!("handle_connection finished");
}
