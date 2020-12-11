use std::fmt;
use std::ops;
use std::cmp;
use serde::{ Serialize, Deserialize };

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Pieces {
    // boolean is has completed first move
    Pawn(bool),
    Rook,
    Bishop,
    Queen,
    King,
    Knight,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Colors {
    Black,
    White,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BoardState {
    Piece((Colors, Pieces)),
    Empty
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Location {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl Location {
    pub fn new(x: isize, y: isize, z: isize) -> Location {
        Location {
            x: x,
            y: y,
            z: z,
        }
    }
}

impl ops::Add<Location> for Location {
    type Output = Location;

    fn add(self, loc: Location) -> Location {
        Location {
            x: self.x + loc.x,
            y: self.y + loc.y,
            z: self.z + loc.z,
        }
    }
}

impl ops::Mul<isize> for Location {
    type Output = Location;

    fn mul(self, fac: isize) -> Location {
        Location {
            x: self.x * fac,
            y: self.y * fac,
            z: self.z * fac,
        }
    }
}

impl ops::MulAssign<isize> for Location {
    
    fn mul_assign(&mut self, fac: isize) {
        self.x *= fac;
        self.y *= fac;
        self.z *= fac;
    }
}

impl cmp::PartialEq for Location {
    fn eq(&self, other: &Location) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Move {
    from: Location,
    to: Location,
    piece: (Colors, Pieces),
}

impl Move {
    pub fn new(from: Location, to: Location, piece: (Colors, Pieces)) -> Move {
        Move {
            from: from,
            to: to,
            piece: piece,
        }
    }

    pub fn from(&self) -> Location {
        self.from
    }
    
    pub fn to(&self) -> Location {
        self.to
    }
}

impl From<(isize, isize, isize)> for Location {
    fn from((x, y, z): (isize, isize, isize)) -> Self {
        Location {
            x: x,
            y: y,
            z: z,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Board {
    board: [[[BoardState; 8]; 8]; 8],
    is_running: bool,
}

impl Board {
    pub fn new() -> Board {
        let mut b: [[[BoardState; 8]; 8]; 8] = [[[BoardState::Empty; 8]; 8]; 8];
        for i in 0..8 {
            // fill pawns
            b[i][1][0] = BoardState::Piece((Colors::White, Pieces::Pawn(false)));
            b[i][6][7] = BoardState::Piece((Colors::Black, Pieces::Pawn(false)));
        }
        {
            use Pieces::*;
            use BoardState::*;
            use Colors::*;
            b[0][0][0] = Piece((White, Rook));
            b[7][0][0] = Piece((White, Rook));
            b[1][0][0] = Piece((White, Knight));
            b[6][0][0] = Piece((White, Knight));
            b[2][0][0] = Piece((White, Bishop));
            b[5][0][0] = Piece((White, Bishop));
            b[3][0][0] = Piece((White, Queen));
            b[4][0][0] = Piece((White, King));

            b[0][7][7] = Piece((Black, Rook));
            b[7][7][7] = Piece((Black, Rook));
            b[1][7][7] = Piece((Black, Knight));
            b[6][7][7] = Piece((Black, Knight));
            b[2][7][7] = Piece((Black, Bishop));
            b[5][7][7] = Piece((Black, Bishop));
            b[3][7][7] = Piece((Black, Queen));
            b[4][7][7] = Piece((Black, King));
        }

        Board {
            board: b,
            is_running: true,
        }
    }

    pub fn at<T: Into<Location>>(&self, l: T) -> BoardState {
        let loc = l.into();
        self.board[loc.x as usize][loc.y as usize][loc.z as usize]
    }

    pub fn set(&mut self, l: Location, s: BoardState) {
        self.board[l.x as usize][l.y as usize][l.z as usize] = s;
    }

    fn location_is_valid(l: Location) -> bool {
        if l.x < 0 || l.x > 7 || l.y < 0 || l.y > 7 || l.z < 0 || l.z > 7 {
            false
        } else {
            true
        }
    }

    fn pawn_moves(&self, l: Location) -> Vec<Move> {
        let piece = match self.at(l) {
            BoardState::Empty => (Colors::White, Pieces::Pawn(false)),
            BoardState::Piece(p) => p,
        };

        let mut offsets: Vec<Location> = Vec::new();
        offsets.push(Location::new(0, 1, 0));
        offsets.push(Location::new(0, 0, 1));
        
        let mut capture_offsets: Vec<Location> = Vec::new();
        capture_offsets.push(Location::new(-1, 1, 0));
        capture_offsets.push(Location::new(1, 1, 0));
        capture_offsets.push(Location::new(1, 0, 1));
        capture_offsets.push(Location::new(-1, 0, 1));
        capture_offsets.push(Location::new(-1, 1, 1));
        capture_offsets.push(Location::new(1, 1, 1));

        let mut max_distance = 1;

        if let (Colors::White, Pieces::Pawn(b)) = piece {
            if !b {
                max_distance = 2;
            }
        }
        if let (Colors::Black, Pieces::Pawn(b)) = piece {
            for i in 0..offsets.len() {
                offsets[i] *= -1;
            }
            for i in 0..capture_offsets.len() {
                capture_offsets[i] *= -1;
            }
            if !b {
                max_distance = 2;
            }
        }

        let mut moves: Vec<Move> = Vec::new();
        for o in offsets {
            let mut loc = l + o;
            let mut distance = 1;
            while Board::location_is_valid(loc) && distance <= max_distance {
                if let BoardState::Empty = self.at(loc) {
                    moves.push(Move::new(l, loc, piece));
                    loc = loc + o;
                    distance += 1;
                } else {
                    break;
                }
            }
        }
        for cap in capture_offsets {
            if Board::location_is_valid(l + cap) {
                match piece {
                    (Colors::Black, _) => {
                        if let BoardState::Piece((Colors::White, _)) = self.at(l + cap) {
                            moves.push(Move::new(l, l + cap, piece));
                        }
                    },
                    (Colors::White, _) => {
                        if let BoardState::Piece((Colors::Black, _)) = self.at(l + cap) {
                            moves.push(Move::new(l, l + cap, piece));
                        }
                    }
                }
            }
        }
        return moves;
    }

    fn knight_moves(&self, l: Location) -> Vec<Move> {
        let mut offsets: Vec<Location> = Vec::new();

        // xy plane
        offsets.push((-1, 2, 0).into());
        offsets.push((1, 2, 0).into());
        offsets.push((2, -1, 0).into());
        offsets.push((2, 1, 0).into());

        // xz plane
        offsets.push((1, 0, 2).into());
        offsets.push((-1, 0, 2).into());
        offsets.push((2, 0, 1).into());
        offsets.push((2, 0, -1).into());

        // yz plane
        offsets.push((0, 1, 2).into());
        offsets.push((0, -1, 2).into());
        offsets.push((0, 2, 1).into());
        offsets.push((0, 2, -1).into());

        for i in 0..12 {
            offsets.push(offsets[i] * -1);
        }

        let mut moves = Vec::new();
        if let BoardState::Piece((Colors::Black, _)) = self.at(l) {
            for o in offsets {
                if Board::location_is_valid(l+o) {
                    match self.at(l + o) {
                        BoardState::Piece((Colors::Black, _)) => {},
                        _ => { moves.push(Move::new(l, l+o, (Colors::Black, Pieces::Knight))) }
                    }
                }
            }
        } else if let BoardState::Piece((Colors::White, _)) = self.at(l) {
            for o in offsets {
                if Board::location_is_valid(l + o) {
                    match self.at(l + o) {
                        BoardState::Piece((Colors::White, _)) => {},
                        _ => { moves.push(Move::new(l, l+o, (Colors::White, Pieces::Knight))) }
                    }
                }
            }
        }

        moves
    }

    fn queen_moves(&self, l: Location) -> Vec<Move> {
        let mut offsets: Vec<Location> = Vec::new();
        // xy plane
        offsets.push((-1, 1, 0).into());
        offsets.push((1, 1, 0).into());
        offsets.push((1, 0, 0).into());
        
        // xz plane
        offsets.push((-1, 0, 1).into());
        offsets.push((0, 0, 1).into());
        offsets.push((1, 0, 1).into());

        // yz plane
        offsets.push((0, -1, 1).into());
        offsets.push((0, 1, 1).into());
        offsets.push((0, 1, 0).into());

        self.queen_rook_bishop_moves(l, offsets)
    }

    fn bishop_moves(&self, l: Location) -> Vec<Move> {
        let mut offsets: Vec<Location> = Vec::new();
        // xy plane
        offsets.push((-1, 1, 0).into());
        offsets.push((1, 1, 0).into());
        
        // xz plane
        offsets.push((-1, 0, 1).into());
        offsets.push((1, 0, 1).into());

        // yz plane
        offsets.push((0, -1, 1).into());
        offsets.push((0, 1, 1).into());

        self.queen_rook_bishop_moves(l, offsets)
    }

    fn rook_moves(&self, l: Location) -> Vec<Move> {
        let mut offsets: Vec<Location> = Vec::new();
        // xy plane
        offsets.push((1, 0, 0).into());
        
        // xz plane
        offsets.push((0, 0, 1).into());

        // yz plane
        offsets.push((0, 1, 0).into());

        self.queen_rook_bishop_moves(l, offsets)
    }

    fn queen_rook_bishop_moves(&self, l: Location, mut offsets: Vec<Location>) -> Vec<Move> {
        let len = offsets.len();
        for i in 0..len {
            offsets.push(offsets[i] * -1);
        }

        let piece = match self.at(l) {
            BoardState::Piece(p) => p,
            BoardState::Empty => (Colors::White, Pieces::Pawn(false)),
        };

        let mut moves: Vec<Move> = Vec::new();

        for o in offsets {
            let mut loc = l + o;
            while Self::location_is_valid(loc) {
                if let BoardState::Empty = self.at(loc) {
                    moves.push(Move::new(l, loc, piece));
                }
                if let BoardState::Piece((color, _)) = self.at(loc) {
                    if color == piece.0 {
                        break;
                    } else {
                        moves.push(Move::new(l, loc, piece));
                        break;
                    }
                }
                loc = loc + o;
            }
        }

        moves
    }

    fn king_moves(&self, l: Location) -> Vec<Move> {
        let mut offsets: Vec<Location> = Vec::new();
        // xy plane
        offsets.push((-1, 1, 0).into());
        offsets.push((1, 1, 0).into());
        offsets.push((1, 0, 0).into());
        
        // xz plane
        offsets.push((-1, 0, 1).into());
        offsets.push((0, 0, 1).into());
        offsets.push((1, 0, 1).into());

        // yz plane
        offsets.push((0, -1, 1).into());
        offsets.push((0, 1, 1).into());
        offsets.push((0, 1, 0).into());

        let len = offsets.len();
        for i in 0..len {
            offsets.push(offsets[i] * -1);
        }

        let mut moves: Vec<Move> = Vec::new();

        let piece = match self.at(l) {
            BoardState::Piece(p) => p,
            BoardState::Empty => (Colors::White, Pieces::Pawn(false)),
        };

        for o in offsets {
            if Self::location_is_valid(l+o) {
                if let BoardState::Empty = self.at(l+o) {
                    moves.push(Move::new(l, l+o, piece));
                }
                if let BoardState::Piece((color, _)) = self.at(l+o) {
                    if color != piece.0 {
                        moves.push(Move::new(l, l+o, piece));
                    }
                }
            }
        }

        moves
    }

    pub fn piece_moves(&self, l: Location) -> Vec<Move> {
        if let BoardState::Empty = self.at(l) {
            vec![]
        } else if let BoardState::Piece(p) = self.at(l) {
            use Pieces::*;
            match p {
                (_, Pawn(_)) => self.pawn_moves(l),
                (_, Knight) => self.knight_moves(l),
                (_, Queen) => self.queen_moves(l),
                (_, Bishop) => self.bishop_moves(l),
                (_, Rook) => self.rook_moves(l),
                (_, King) => self.king_moves(l),
            }
        } else {
            vec![]
        }
    }

    pub fn execute_move(&mut self, m: &Move) {
        self.set(m.to(), self.at(m.from()));
        self.set(m.from(), BoardState::Empty);
        match m.piece {
            (Colors::Black, Pieces::Pawn(false)) => self.set(m.to(), BoardState::Piece((Colors::Black, Pieces::Pawn(true)))),
            (Colors::White, Pieces::Pawn(false)) => self.set(m.to(), BoardState::Piece((Colors::White, Pieces::Pawn(true)))),
            _ => {}
        }
    }

    pub fn update_board(&mut self, other: &Board) {
        self.board = other.board;
        self.is_running = other.is_running;
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

impl Pieces {
    pub fn character(&self) -> char {
        match self {
            Pieces::Bishop => 'B',
            Pieces::King => 'K',
            Pieces::Knight => 'N',
            Pieces::Pawn(_) => 'P',
            Pieces::Queen => 'Q',
            Pieces::Rook => 'R',
        }
    }
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoardState::Piece(c) => {
                write!(f, "{}", match c {
                    (Colors::Black, p) => p.character().to_ascii_lowercase(),
                    (Colors::White, p) => p.character().to_ascii_uppercase(),
                })
            },
            BoardState::Empty => {
                write!(f, ".")
            }
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for z in 0..8 {
            write!(f, "Board {}: \n", z)?;
            for y in (0..8).rev() {
                for x in 0..8 {
                    write!(f, "{}", self.board[x][y][z])?;
                }
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}