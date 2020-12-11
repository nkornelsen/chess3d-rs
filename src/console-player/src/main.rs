use std::convert::From;

use chess3d::Board;
use chess3d::Location;

use cursive::views::TextView;
use cursive::Printer;
use cursive::views::Dialog;
use cursive::views::Panel;
use cursive::views::LinearLayout;
use cursive::Vec2;
use cursive::event::{ Event, EventResult, MouseEvent, MouseButton };
use cursive::theme::Color;
use cursive::theme::BaseColor;
use cursive::theme::ColorStyle;

struct BoardView {
    chess_board: Board,
    cursor: Option<(isize, isize, isize)>,
    current_moves: Option<Vec<chess3d::Move>>,
}

impl BoardView {
    pub fn new() -> BoardView {
        BoardView {
            chess_board: Board::new(),
            cursor: None,
            current_moves: None,
        }
    }
}

impl cursive::view::View for BoardView {
    fn draw(&self, printer: &Printer) {
        let mut bg_colors = [[[Color::Dark(BaseColor::White); 8]; 8]; 8];

        for x in 0..8 {
            for y in 0..8 {
                for z in 0..8 {
                    if (x + (y % 2) + (z % 2)) == 0 {
                        bg_colors[x][y][z] = Color::Light(BaseColor::Black);
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
                    let tile = self.chess_board.at((x as isize, y as isize, z as isize));

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
                    let bg_color = match self.cursor {
                        None => Color::Dark(BaseColor::White),
                        Some(_) => {
                            bg_colors[x][y][z]
                        },
                    };

                    printer.with_color(
                        ColorStyle::new(color, bg_color),
                        |printer| printer.print(((x * 2) + (19 * z + 2), y + 2), &text)
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
                if pos.x >= 0 && pos.x <= 7 && pos.y >= 0 && pos.y <= 7 {
                    // valid position
                    if let Some(moves) = &self.current_moves {
                        let m = moves.iter().find(|i| i.to() == (pos.x, pos.y, board).into());
                        if let Some(selected_move) = m {
                            self.chess_board.execute_move(selected_move);
                            self.cursor = None;
                            self.current_moves = None;
                        } else {
                            self.cursor = Some((pos.x, pos.y, board));
                            self.current_moves = Some(self.chess_board.piece_moves((pos.x, pos.y, board).into()));
                        }
                    } else {
                        self.cursor = Some((pos.x, pos.y, board));
                        self.current_moves = Some(self.chess_board.piece_moves((pos.x, pos.y, board).into()));
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

fn main() {
    // let b: Board = Board::new();

    // println!("{}", b);

    // println!("{:?}", b.piece_moves(Location::new(0, 1, 0)));
    let mut siv = cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    siv.add_layer(
        Dialog::new()
            .title("3D Chess")
            .content(
                LinearLayout::horizontal()
                    .child(Panel::new(BoardView::new())),
            )
    );
    siv.run();
}
