use graphics::types::Color;
use graphics::{Context, Graphics};
use graphics::character::CharacterCache;
use alvinw_chess::{pos::BoardPos, piece::PieceType, board};

use crate::gameboard_controller::GameboardController;
use crate::utils::current_time;

/// Stores gameboard view settings.
pub struct GameboardViewSettings {
    pub position: [f64; 2],
    pub size: f64,
    pub border_color: Color,
    pub board_edge_radius: f64,
    pub cell_edge_radius: f64,
    pub selected_cell_background_color: Color,
    pub promotion_cell_color: Color,
    pub hovered_promotion_cell_color: Color,
    pub dark_square_color: Color,
    pub light_square_color: Color,
    pub text_color: Color
}

impl GameboardViewSettings {
    /// Creates new gameboard view settings.
    pub fn new() -> GameboardViewSettings {
        GameboardViewSettings {
            position: [250.0, 100.0],
            size: 400.0,
            border_color: [0.0, 0.0, 0.0, 1.0],
            board_edge_radius: 3.0,
            cell_edge_radius: 1.0,
            selected_cell_background_color: [0.9, 0.7, 0.8, 1.0],
            promotion_cell_color: [0.8, 0.6, 0.6, 0.5],
            hovered_promotion_cell_color: [0.8, 1.0, 0.6, 0.6],
            dark_square_color: [0.4, 0.3, 0.3, 1.0],
            light_square_color: [0.6, 0.45, 0.45, 1.0],
            text_color: [0.0, 0.0, 0.1, 1.0],
        }
    }
}

/// Stores visual information about a gameboard.
pub struct GameboardView {
    /// Stores gameboard view settings.
    pub settings: GameboardViewSettings,
}

impl GameboardView {
    /// Creates a new gameboard view.
    pub fn new(settings: GameboardViewSettings) -> GameboardView {
        GameboardView {
            settings: settings,
        }
    }

    /// Draw gameboard.
    pub fn draw<G: Graphics, C>(
        &self,
        controller: &mut GameboardController,
        glyphs: &mut C,
        c: &Context,
        g: &mut G
    ) 
    where C: CharacterCache<Texture = G::Texture>
    {
        use graphics::{Image, Rectangle, Transformed};

        let ref settings = self.settings;

        let cell_size = settings.size / 8.0;
        for rank in 0..8 {  
            for file in 0..8 {
                let pos = [file as f64 * cell_size, (7 - rank) as f64 * cell_size];
                let cell_rect = [
                    settings.position[0] + pos[0], settings.position[1] + pos[1],
                    cell_size, cell_size
                ];
                let mut square_color = match (file + rank) % 2 == 0 {
                    false => settings.light_square_color,
                    true => settings.dark_square_color,
                };
                if let Some(selected_cell) = controller.selected_cell.clone() {
                    if selected_cell == BoardPos::new(file, rank) {
                        square_color[1] += 0.25;
                    }
                    else if controller.selected_cell_moves.contains(&BoardPos::new(file, rank)) {
                        if controller.gameboard.get_tile(&BoardPos::new(file, rank)).is_some() {
                            square_color[0] += 0.2;
                        }
                        else {
                            square_color[1] += 0.25;
                        }
                    }
                }
                
                if controller.is_checkmate {
                    square_color[0] += 0.2;
                }
                else if let Some(tile) = controller.gameboard.get_tile(&BoardPos::new(file, rank)) {
                    if controller.is_check && tile.piece() == PieceType::King && tile.color() == controller.gameboard.current_turn() {
                        square_color[0] += 0.2;
                    }
                }

                Rectangle::new(square_color).draw(cell_rect, &c.draw_state, c.transform, g);
            }
        }
        
        let text_image = Image::new_color(settings.text_color);
        for rank in 0..8 {
            for file in 0..8 {
                
                let pos = BoardPos::new(file, rank);
                if let Some(tile) = controller.gameboard.get_tile(&pos) {
                    let ch = match tile.piece() {
                        PieceType::Pawn => match tile.color() {
                            board::Color::White => 'p',
                            board::Color::Black => 'o',
                        },
                        PieceType::Knight => match tile.color() {
                            board::Color::White => 'h',
                            board::Color::Black => 'j',
                            
                        },
                        PieceType::Bishop => match tile.color() {
                            board::Color::White => 'b',
                            board::Color::Black => 'n',
                        },
                        PieceType::Rook => match tile.color() {
                            board::Color::White => 'r',
                            board::Color::Black => 't',
                        },
                        PieceType::Queen => match tile.color() {
                            board::Color::White => 'q',
                            board::Color::Black => 'w',
                        },
                        PieceType::King => match tile.color() {
                            board::Color::White => 'k',
                            board::Color::Black => 'l',
                        },
                    };
                    if let Ok(character) = glyphs.character(34, ch) {

                        let mut piece_file = file as f64;
                        let mut piece_rank = rank as f64;

                        if controller.animation.running {
                            if let Some(target) = controller.animation.target() {
                                if target == pos {
                                    if let Some(position) = controller.animation.update_animation(current_time()) {
                                        piece_file = position.0;
                                        piece_rank = position.1;
                                    }
                                }
                            }
                        }

                        let pos = [
                            settings.position[0] + cell_size * (piece_file + 0.05),
                            settings.position[1] + cell_size * (7.0 - piece_rank + 0.85), 
                        ];
                        let ch_x = pos[0] + character.left();
                        let ch_y = pos[1] - character.top();
                        let text_image = text_image.src_rect([
                            character.atlas_offset[0],
                            character.atlas_offset[1],
                            character.atlas_size[0],
                            character.atlas_size[1],
                        ]);
                        text_image.draw(character.texture,
                                        &c.draw_state,
                                        c.transform.trans(ch_x, ch_y),
                                        g);
                    }
                }
            }
        }

        let board_rect = [
            settings.position[0],
            settings.position[1],
            settings.size,
            settings.size,
        ];

        // Draw board edge.
        Rectangle::new_border(
            settings.border_color,
            settings.board_edge_radius,
        )
        .draw(board_rect, &c.draw_state, c.transform, g);

        
        if let Some(promotion_pos) = controller.ongoing_promotion.clone() {
            let file = promotion_pos.file() as f64 - 1.5;
            let mut rank = (7 - promotion_pos.rank()) as f64;
            let piece_textures: [char; 4];

            //Opposite color because api swaps turn before promotion
            if controller.gameboard.current_turn() != board::Color::White {
                piece_textures = ['h', 'b', 'r', 'q'];
                rank -= 1.0;
            }
            else {
                piece_textures = ['j', 'n', 't', 'w'];
                rank += 1.0;
            }

            for file_offset in 0..4 {
                let pos = [(file + file_offset as f64) * cell_size, rank * cell_size];
                let cell_rect = [
                    settings.position[0] + pos[0] + 1.0, settings.position[1] + pos[1] + 1.0,
                    cell_size - 2.0, cell_size - 2.0
                ];

                let mut square_color = settings.promotion_cell_color;
                if let Some(square) = controller.hovered_promotion_square {
                    if square == file_offset {
                        square_color = settings.hovered_promotion_cell_color;
                    }
                }
                Rectangle::new(square_color)
                    .draw(cell_rect, &c.draw_state, c.transform, g);

                if let Ok(character) = glyphs.character(34, piece_textures[file_offset]) {
                    let ch_x = settings.position[0] + pos[0] + character.left() + cell_size * 0.05;
                    let ch_y = settings.position[1] + pos[1] - character.top() + cell_size * 0.85;
                    let text_image = text_image.src_rect([
                        character.atlas_offset[0],
                        character.atlas_offset[1],
                        character.atlas_size[0],
                        character.atlas_size[1],
                    ]);
                    text_image.draw(character.texture,
                                    &c.draw_state,
                                    c.transform.trans(ch_x, ch_y),
                                    g);
                }
            }
        }
    }
}