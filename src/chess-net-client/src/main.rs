use std::io::prelude::*;
use std::net::TcpStream;
use std::convert::From;
use std::borrow::Borrow;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{ Arc, Mutex };

use chess3d::Board;
use chess3d::Location;

use cursive::views::{ EditView, TextView, Panel, BoxedView };
use cursive::view::{ Nameable, Resizable };
use cursive::Cursive;
use cursive::Printer;
use cursive::views::Dialog;
use cursive::views::Button;
use cursive::views::LinearLayout;
use cursive::Vec2;
use cursive::event::{ Event, EventResult, MouseEvent, MouseButton };
use cursive::theme::Color;
use cursive::theme::BaseColor;
use cursive::theme::ColorStyle;

use serde_json;
use chess3d_common::ServerMessage;

struct OnlineGame {
    chess_board: Arc<Mutex<Board>>,
    cursor: Option<(isize, isize, isize)>,
    current_moves: Option<Vec<chess3d::Move>>,
    stream: TcpStream,
}

impl OnlineGame {
    pub fn new(board: Arc<Mutex<Board>>, stream: TcpStream) -> OnlineGame {
        OnlineGame {
            chess_board: board,
            cursor: None,
            current_moves: None,
            stream: stream,
        }
    }
}

impl cursive::view::View for OnlineGame {
    fn draw(&self, printer: &Printer) {
        let mut bg_colors = [[[Color::Dark(BaseColor::White); 8]; 8]; 8];

        for x in 0..8 {
            for y in 0..8 {
                for z in 0..8 {
                    if ((x % 2) + (y % 2) + (z % 2)) % 2 == 0 {
                        bg_colors[x][y][z] = Color::RgbLowRes(2, 2, 2);
                    }
                }
            }
        }

        match self.cursor {
            None => {},
            Some(pos) => {
                bg_colors[pos.0 as usize][pos.1 as usize][pos.2 as usize] = Color::Dark(BaseColor::Yellow);
            
                // set bg_colors
                // let moves = self.chess_board.piece_moves((pos.0, pos.1, pos.2).into());
                // println!("Moves: {:?}", moves);
                if let Some(moves) = &self.current_moves {
                    for m in moves {
                        bg_colors[m.to().x as usize][m.to().y as usize][m.to().z as usize] = Color::Dark(BaseColor::Green);
                    }
                }
            },
        };
        
        for z in 0..8 {
            printer.print_box((19 * z + 1 as usize, 1 as usize), (18, 10), false);

            for y in (0..8).rev() {
                for x in 0..8 {
                    let tile = {
                        // self.chess_board.borrow().at((x as isize, y as isize, z as isize))
                        self.chess_board.as_ref().lock().unwrap().at((x as isize, y as isize, z as isize))
                    };

                    use chess3d::BoardState;
                    use chess3d::Colors;
                    let mut text: String = " ".to_owned();
                    let color;
                    match tile {
                        BoardState::Empty => { 
                            text.push_str(" ");
                            color = Color::Dark(BaseColor::White);
                        },
                        BoardState::Piece((Colors::Black, p)) => {
                            text.push(p.character());
                            color = Color::RgbLowRes(0, 0, 0);
                        },
                        BoardState::Piece((Colors::White, p)) => {
                            text.push(p.character());
                            color = Color::Rgb(255, 255, 255);
                        },
                    }
                    // let bg_color = match self.cursor {
                    //     None => Color::Dark(BaseColor::White),
                    //     Some(_) => {
                    //         bg_colors[x][y][z]
                    //     },
                    // };

                    printer.with_color(
                        ColorStyle::new(color, bg_colors[x][y][z]),
                        |printer| printer.print(((x * 2) + (19 * z + 2), (7 - y) + 2), &text)
                    );
                }
            }
        }
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        Vec2 {
            x: (19)*8 + 1,
            y: 12,
        }
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        use cursive::XY;
        match event {
            Event::Mouse {offset, position, event: MouseEvent::Release(MouseButton::Left)} => {
                let mut pos: XY<isize> = (position.x as isize, position.y as isize).into();
                let off: XY<isize> = (offset.x as isize, offset.y as isize).into();
                pos = pos - off;
                let board = (pos.x - 2) / 19;
                pos.x -= 19 * board;
                pos = pos - (3, 2);
                pos.x = (pos.x + 1) / 2;
                pos.y = 7 - pos.y;
                if pos.x >= 0 && pos.x <= 7 && pos.y >= 0 && pos.y <= 7 {
                    // valid position
                    if let Some(moves) = &self.current_moves {
                        let m = moves.iter().find(|i| i.to() == (pos.x, pos.y, board).into());
                        if let Some(selected_move) = m {
                            // self.chess_board.as_ref().borrow_mut().execute_move(selected_move);
                            chess3d_common::emit_message(&mut self.stream, &ServerMessage::PlayerMove { r#move: selected_move.clone() });

                            self.cursor = None;
                            self.current_moves = None;
                        } else {
                            self.cursor = Some((pos.x, pos.y, board));
                            self.current_moves = Some((*self.chess_board.as_ref()).lock().unwrap().piece_moves((pos.x, pos.y, board).into()));
                        }
                    } else {
                        self.cursor = Some((pos.x, pos.y, board));
                        self.current_moves = Some((*self.chess_board.as_ref()).lock().unwrap().piece_moves((pos.x, pos.y, board).into()));
                    }
                    EventResult::Consumed(None)
                } else {
                    EventResult::Ignored
                }
            },
            _ => EventResult::Ignored,
        }
    }
}

struct CursiveData {
    sink: cursive::CbSink
}

fn main() {
    // let b: Board = Board::new();

    // println!("{}", b);

    // println!("{:?}", b.piece_moves(Location::new(0, 1, 0)));
    let mut siv: cursive::Cursive = cursive::default();
    siv.add_global_callback('q', |s| s.quit());

    siv.set_user_data(CursiveData {
        sink: siv.cb_sink().clone()
    });

    siv.add_layer(
        Dialog::new()
            .title("Chess 3D")
            .padding_lrtb(2, 2, 1, 1)
            .content(
                LinearLayout::vertical()
                    .child(Button::new_raw("  Connect  ", show_connect_dialog)
            ))
    );
    siv.run();
}

fn show_connect_dialog(siv: &mut Cursive) {
    siv.add_layer(
        Dialog::new()
            .title("Connect to Chess Match")
            .content(
                LinearLayout::vertical()
                    .child(TextView::new("Address"))
                    .child(EditView::new()
                        .on_submit(connect_to_game)
                        .with_name("Address")
                        .fixed_width(20),
                    )
                
            )
    );
}

fn connect_to_game(siv: &mut Cursive, server: &str) {
    let stream = TcpStream::connect(server).unwrap();
    let game_board = Arc::new(Mutex::new(Board::new()));
    let board_view = OnlineGame::new(game_board.clone(), stream.try_clone().unwrap());
    siv.pop_layer().unwrap();
    siv.add_layer(
        Dialog::new()
            .title("Chess")
            .content(
                Panel::new(board_view)
            )
    );

    // start receiver thread
    use std::thread;
    let mut read_stream = stream.try_clone().unwrap();
    let game_ref = game_board.clone();

    let user_data = siv.user_data::<CursiveData>().unwrap();
    let sink = user_data.sink.clone();

    thread::spawn(move || {
        let mut running = { game_ref.as_ref().lock().unwrap().is_running() };
        while running {
            let data = chess3d_common::recv_message(&mut read_stream);
            if let Ok(message) = data {
                match message {
                    ServerMessage::BoardUpdate { board } => {
                        println!("Received board update");
                        game_ref.as_ref().lock().unwrap().update_board(&board);
                        sink.send(Box::new(|_| {})).unwrap();
                        println!("Updated board successfully");
                    },
                    _ => {}
                }

                running = game_ref.as_ref().lock().unwrap().is_running();
            }
        }
    });
}