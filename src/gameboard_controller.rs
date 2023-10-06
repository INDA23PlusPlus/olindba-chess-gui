use std::collections::HashSet;
use piston::GenericEvent;
use alvinw_chess::{game::Game, pos::BoardPos, game::GameState, piece::PieceType, game::MovePieceError, board};

use crate::animation::MoveAnimation;
use crate::utils::current_time;

/// Handles events for chess game.
pub struct GameboardController {
    /// Stores the gameboard state.
    pub gameboard: Game,
    pub selected_cell: Option<BoardPos>,
    pub selected_cell_moves: HashSet<BoardPos>,
    pub ongoing_promotion: Option<BoardPos>,
    ongoing_promotion_from: BoardPos,
    pub hovered_promotion_square: Option<usize>,
    pub is_check: bool,
    pub is_checkmate: bool,
    pub animation: MoveAnimation,
    cursor_pos: [f64; 2]
}

impl GameboardController {
    /// Creates a new gameboard controller.
    pub fn new(gameboard: Game) -> GameboardController {
        GameboardController {
            gameboard: gameboard,
            selected_cell: None,
            selected_cell_moves: HashSet::new(),
            ongoing_promotion: None,
            ongoing_promotion_from: BoardPos::new(0, 0),
            hovered_promotion_square: None,
            is_check: false,
            is_checkmate: false,
            animation: MoveAnimation::new(),
            cursor_pos: [0.0; 2],
        }
    }

    /// Handles events.
    pub fn event<E: GenericEvent>(&mut self, pos: [f64; 2], size: f64, e: &E) -> Option<(BoardPos, BoardPos, usize)> {
        use piston::input::{Button, MouseButton};

        if let Some(cursor_pos) = e.mouse_cursor_args() {
            self.cursor_pos = cursor_pos;

            if let Some(promotion_square) = self.ongoing_promotion.clone() {
                let square_x = (cursor_pos[0] - pos[0]) / size * 8.0;
                let square_y = (cursor_pos[1] - pos[1]) / size * 8.0;
                let pro_menu_min = promotion_square.file() as f64 - 1.5;

                //Opposite color because api swaps turn before promotion
                if square_x >= pro_menu_min && square_x < pro_menu_min + 4.0  && 
                    ((self.gameboard.current_turn() != board::Color::White && square_y >= -1.0 && square_y < 0.0) ||
                     (self.gameboard.current_turn() != board::Color::Black && square_y >=  8.0 && square_y < 9.0)) { 
                        
                    self.hovered_promotion_square = Some((square_x - pro_menu_min) as usize);
                }
                else {
                    self.hovered_promotion_square = None;
                }
            }
        }

        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            // Find coordinates relative to upper left corner.
            let x = self.cursor_pos[0] - pos[0];
            let y = self.cursor_pos[1] - pos[1];
            
            if self.ongoing_promotion.is_some() {
                if let Some(hovered_square) = self.hovered_promotion_square {
                    let promotion = match hovered_square {
                        0 => PieceType::Knight,
                        1 => PieceType::Bishop,
                        2 => PieceType::Rook,
                        3 => PieceType::Queen,
                        _ => PieceType::Queen
                    };
                    self.gameboard.promote(promotion);
                    let to = self.ongoing_promotion.clone().unwrap();
                    let from = self.ongoing_promotion_from.clone();
                    self.ongoing_promotion = None;
                    self.hovered_promotion_square = None;
                    return Some((from, to, hovered_square));
                }
            }
            else {  
                if x >= 0.0 && x < size && y >= 0.0 && y < size {
                    let file = (x / size * 8.0) as u8;
                    let rank = (y / size * 8.0) as u8;
                    let clicked_cell = BoardPos::new(file, 7 - rank);
        
                    if let Some(selected_cell) = self.selected_cell.clone() {
                        if selected_cell != clicked_cell {
                            
                            if let Some(tile) = self.gameboard.get_tile(&clicked_cell) {

                                if tile.color() == self.gameboard.current_turn() {
                                    self.selected_cell = Some(clicked_cell);
                                    self.update_selected_cell_moves();
                                }
                                else {
                                    let mut mv = None;
                                    if self.check_selected_move(&selected_cell, &clicked_cell) {
                                        mv = Some((selected_cell, clicked_cell, 0));
                                    };
                                    self.selected_cell = None;
                                    self.selected_cell_moves.drain();
                                    return mv;
                                }
                            }
                            else {
                                let mut mv = None;
                                if self.check_selected_move(&selected_cell, &clicked_cell) {
                                    mv = Some((selected_cell, clicked_cell, 0));
                                };
                                self.selected_cell = None;
                                self.selected_cell_moves.drain();
                                return mv;
                            }
                        }
                    }
                    else if let Some(tile) = self.gameboard.get_tile(&clicked_cell) {
                        if tile.color() == self.gameboard.current_turn() {
                            self.selected_cell = Some(clicked_cell);
                            self.update_selected_cell_moves();
                        }
                    }
                }
                else {
                    self.selected_cell = None;
                    self.selected_cell_moves.drain();
                }
            }
        }
        return None;
    }

    fn update_selected_cell_moves(&mut self) {
        if let Some(selected_cell) = self.selected_cell.clone() {
            self.selected_cell_moves.drain();

            match self.gameboard.get_legal_moves(&selected_cell) {
                Ok(moves) => {
                    for mv in moves {
                        self.selected_cell_moves.insert(mv);
                    }
                },
                _ => {}
            }
        }
    }

    pub fn check_selected_move(&mut self, from: &BoardPos, to: &BoardPos) -> bool {
        
            match self.gameboard.move_piece(&from, &to) {
                Ok(_) => {
                    println!("Moved");
                    self.animation.set_animation(current_time(), 150, &from, &to);
                    self.is_check = false;
                    self.is_checkmate = false;
                },
                Err(MovePieceError::NoTile) => panic!("The tile is empty!"),
                Err(MovePieceError::NotCurrentTurn) => panic!("You cannot move your opponent's pieces!"),
                Err(MovePieceError::InvalidMove) => panic!("That is not a valid move."),
            }

            match self.gameboard.get_state() {
                GameState::Normal => println!("Game is in progress"),
                GameState::Check(_) => self.is_check = true,
                GameState::Checkmate(_) => self.is_checkmate = true,
                GameState::PromotionRequired(_) => {
                    println!("Select promotion!");
                    self.gameboard.promote(PieceType::Queen);
                    return true;
                },
            };
            return true;
        }
}