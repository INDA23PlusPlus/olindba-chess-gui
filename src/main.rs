mod gameboard_controller;
mod gameboard_view;
mod animation;
mod utils;

use alvinw_chess::piece::PieceType;
use alvinw_chess::{game::Game, pos::BoardPos, board};
use chess_network_protocol::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{OpenGL, Filter, GlGraphics, GlyphCache, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::{EventLoop, RenderEvent, WindowSettings};
use local_ip_address::local_ip;
use std::net::TcpListener;
use serde::{Serialize, Deserialize};
use std::io::prelude::*;

pub use crate::gameboard_controller::GameboardController;
pub use crate::gameboard_view::{GameboardView, GameboardViewSettings};

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: GlutinWindow = WindowSettings::new("Chess", [900, 600])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut events = Events::new(EventSettings::new().lazy(false));
    let mut gl = GlGraphics::new(opengl);

    let game= Game::new();
    let mut gameboard_controller = GameboardController::new(game);
    let gameboard_view_settings = GameboardViewSettings::new();
    let gameboard_view = GameboardView::new(gameboard_view_settings);

    let texture_settings = TextureSettings::new().filter(Filter::Nearest);
    let ref mut glyphs = GlyphCache::new("assets/CHEQ_TT.TTF", (), texture_settings)
        .expect("Could not load font");

    println!("Play as server or client?");
    let mut inp = String::new();
    std::io::stdin().read_line(&mut inp).unwrap();

    match inp.trim() {
        "server" => {
            println!("Use {} to join with client", local_ip().unwrap());
            let listener = TcpListener::bind("0.0.0.0:8384").unwrap();
            let (stream, _addr) = listener.accept().unwrap();
            let mut de = serde_json::Deserializer::from_reader(&stream);
            let deserialized = ClientToServerHandshake::deserialize(&mut de).unwrap();
            let handshake = ServerToClientHandshake {
                features: vec![
                    Features::EnPassant, 
                    Features::Castling, 
                    Features::Promotion,
                    Features::Stalemate,
                    Features::PossibleMoveGeneration,
                    ],
                board: current_board(&gameboard_controller.gameboard),
                moves: available_moves(&mut gameboard_controller.gameboard),
                joever: Joever::Ongoing,
            };
            serde_json::to_writer(&stream, &handshake).unwrap();

            while let Some(e) = events.next(&mut window) {

                if gameboard_controller.gameboard.current_turn() == board::Color::White {
                    let deserialized = ClientToServer::deserialize(&mut de).unwrap();
                    println!("Received: {:?}", deserialized);

                    let mv = match deserialized {
                        ClientToServer::Move(mv) => {
                            gameboard_controller.check_selected_move(&BoardPos::new(mv.start_x as u8, mv.start_y as u8), 
                            &BoardPos::new(mv.end_x as u8, mv.end_y as u8));
                        }
                        _ => unimplemented!()
                    };
                    
                    let state = ServerToClient::State {
                        board: current_board(&gameboard_controller.gameboard),
                        moves: available_moves(&mut gameboard_controller.gameboard),
                        joever: Joever::Ongoing,
                        //Client should know what move they made
                        move_made: Move {
                            start_x: 0,
                            start_y: 0,
                            end_x: 0,
                            end_y: 0,
                            promotion: Piece::None,
                        }
                    };
                    
                    //send
                    serde_json::to_writer(&stream, &state).unwrap();
                }
                else {
                    if let Some(mv) = gameboard_controller.event(
                        gameboard_view.settings.position,
                        gameboard_view.settings.size,
                        &e) {

                            let state = ServerToClient::State {
                                board: current_board(&gameboard_controller.gameboard),
                                moves: available_moves(&mut gameboard_controller.gameboard),
                                joever: Joever::Ongoing,
                                //should be the move made by the server
                                move_made: Move {
                                    start_x: mv.0.file() as usize,
                                    start_y: mv.0.rank() as usize,
                                    end_x: mv.1.file() as usize,
                                    end_y: mv.1.rank() as usize,
                                    promotion: Piece::BlackQueen,
                                }
                            };
            
                            serde_json::to_writer(&stream, &state).unwrap();
                            println!("Sent move");
                    }   
                    
                    if let Some(args) = e.render_args() {
                        gl.draw(args.viewport(), |c, g| {
                            use graphics::clear;
            
                            clear([0.3, 0.3, 0.5, 1.0], g);
                            gameboard_view.draw(&mut gameboard_controller, glyphs, &c, g);
                        });
                    }
                }
            }
        },
        "client" => {
            println!("cli");
        }
        _ => {
            panic!("Input has to be either client or server");
        }
    }
}

fn available_moves(game: &mut Game) -> Vec<Move> {
    let mut moves: Vec<Move> = vec![];
    for file in 0..8 {
        for rank in 0..8 {
            let pos = BoardPos::new(file, rank);
            if let Some(tile) = game.get_tile(&pos) {
                match game.get_legal_moves(&pos) {
                    Ok(legal_moves) => {
                        for mv in legal_moves {
                            moves.push(Move {start_x: file as usize, start_y: rank as usize, end_x: mv.file() as usize, end_y: mv.rank() as usize, promotion: Piece::None});
                        }
                    },
                    _ => {}
                }
            }
        }
    }
    moves
}

fn current_board(game: &Game) -> [[Piece; 8]; 8] {
    let mut board = [[Piece::None; 8]; 8];
    for file in 0..8 {
        for rank in 0..8 {
            let pos = BoardPos::new(file, rank);
            if let Some(tile) = game.get_tile(&pos) {
                board[file as usize][rank as usize] = match tile.piece() {
                    PieceType::Pawn => match tile.color() {
                        board::Color::White => Piece::WhitePawn,
                        board::Color::Black => Piece::BlackPawn
                    }, 
                    PieceType::Knight => match tile.color() {
                        board::Color::White => Piece::WhiteKnight,
                        board::Color::Black => Piece::BlackKnight
                    },
                    PieceType::Bishop => match tile.color() {
                        board::Color::White => Piece::WhiteBishop,
                        board::Color::Black => Piece::BlackBishop
                    },
                    PieceType::Rook => match tile.color() {
                        board::Color::White => Piece::WhiteRook,
                        board::Color::Black => Piece::BlackRook
                    },
                    PieceType::Queen => match tile.color() {
                        board::Color::White => Piece::WhiteQueen,
                        board::Color::Black => Piece::BlackQueen
                    },
                    PieceType::King => match tile.color() {
                        board::Color::White => Piece::WhiteKing,
                        board::Color::Black => Piece::BlackKing
                    },
                }
            }
        }
    }
    board
}