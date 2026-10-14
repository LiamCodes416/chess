use eframe::egui;
use shakmaty::Chess;
use shakmaty::{Color, Role, Square, EnPassantMode};
use shakmaty::Position;
use shakmaty::fen::Fen;
use shakmaty::CastlingMode;
use shakmaty::san::San;

use crate::engine::EngineAnalyzer;

fn piece_unicode(
    role: Role,
    color: Color,
) -> &'static str {
    match (role, color) {
        (Role::Pawn,   Color::White) => "♙",
        (Role::Knight, Color::White) => "♘",
        (Role::Bishop, Color::White) => "♗",
        (Role::Rook,   Color::White) => "♖",
        (Role::Queen,  Color::White) => "♕",
        (Role::King,   Color::White) => "♔",

        (Role::Pawn,   Color::Black) => "♟",
        (Role::Knight, Color::Black) => "♞",
        (Role::Bishop, Color::Black) => "♝",
        (Role::Rook,   Color::Black) => "♜",
        (Role::Queen,  Color::Black) => "♛",
        (Role::King,   Color::Black) => "♚",
    }
}

pub struct MoveAnalysis {
    pub san: String,          // e.g., "Nf3"
    pub fen: String,          // FEN after the move
    pub eval: i32,            // Score in centipawns
    pub best_move: String,    // Engine's suggested best move
    pub classification: MoveClass,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MoveClass {
    Best,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

pub struct ChessApp {
    pub history: Vec<MoveAnalysis>, // Step-by-step history
    pub selected_move: usize,       // Current ply selected in UI
    pub position: Chess,            // Active board position for rendering
    pub engine: EngineAnalyzer,
    pub selected_square: Option<Square>,
}

impl Default for ChessApp {
    fn default() -> Self {
        let initial_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        Self {
            history: vec![MoveAnalysis {
                san: "Start".to_string(),
                fen: initial_fen.to_string(),
                eval: 0,
                best_move: "-".to_string(),
                classification: MoveClass::Best,
            }],
            selected_move: 0,
            position: Chess::default(),
            engine: EngineAnalyzer::new("stockfish-windows-x86-64-universal"),
            selected_square: None,
        }
    }
}

impl ChessApp {
    // Syncs active rendering board with the currently selected history index
    fn update_position_from_history(&mut self) {
        if let Some(analysis) = self.history.get(self.selected_move) {
            let fen_str = analysis.fen.clone();
            if let Ok(fen) = fen_str.parse::<Fen>() {
                if let Ok(parsed_pos) = fen.into_position(CastlingMode::Standard) {
                    self.position = parsed_pos;
                }
            }
        }
    }
}

impl eframe::App for ChessApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        // --- TOP TOOLBAR WITH NAVIGATION & PGN LOADING ---
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📁 Load PGN").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("PGN Files", &["pgn"]).pick_file() {
                        if let Ok(contents) = std::fs::read_to_string(path) {
                            println!("Loaded PGN file successfully (Length: {} bytes)", contents.len());
                        }
                    }
                }

                ui.separator();

                // Previous move button (<)
                if ui.button("◀ Prev").clicked() && self.selected_move > 0 {
                    self.selected_move -= 1;
                    self.update_position_from_history();
                }

                // Next move button (>)
                if ui.button("Next ▶").clicked() && self.selected_move + 1 < self.history.len() {
                    self.selected_move += 1;
                    self.update_position_from_history();
                }

                ui.separator();

                if ui.button("⚡ Evaluate Position").clicked()
                {
                    let idx = self.selected_move;

                    if let Some(analysis)
                        = self.history.get(idx)
                    {
                        let result =
                            self.engine.analyze_fen(
                                &analysis.fen
                            );

                        if let Some(current) =
                            self.history.get_mut(idx)
                        {
                            current.eval = result.eval;
                            current.best_move =
                                result.best_move;
                        }

                    }
                }
            });
        });

        // --- RIGHT SIDE PANEL: ANALYSIS METRICS ---
        let current_analysis = self.history.get(self.selected_move);
        let eval_val = current_analysis.map_or(0, |a| a.eval);
        let best_move_str = current_analysis.map_or("-".to_string(), |a| a.best_move.clone());
        let class_opt = current_analysis.map(|a| a.classification);

        egui::SidePanel::right("analysis")
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Analysis");

                let normalized =
                    ((eval_val + 1000) as f32 / 2000.0)
                        .clamp(0.0, 1.0);

                let (rect, _) =
                    ui.allocate_exact_size(
                        egui::vec2(30.0, 250.0),
                        egui::Sense::hover(),
                    );

                ui.painter().rect_filled(
                    rect,
                    0.0,
                    egui::Color32::BLACK,
                );

                let white_height =
                    rect.height() * normalized;

                let white_rect =
                    egui::Rect::from_min_max(
                        egui::pos2(
                            rect.left(),
                            rect.bottom() -
                                white_height,
                        ),
                        rect.max,
                    );

                ui.painter().rect_filled(
                    white_rect,
                    0.0,
                    egui::Color32::WHITE,
                );

                ui.label(format!("Evaluation: {:.2}", eval_val as f32 / 100.0));

                ui.separator();
                ui.label("Best Move:");
                ui.monospace(best_move_str);

                ui.separator();
                ui.label("Classification:");
                let (class_str, class_color) = if let Some(class) = class_opt {
                    match class {
                        MoveClass::Best => ("Best", egui::Color32::GREEN),
                        MoveClass::Good => ("Good", egui::Color32::LIGHT_GREEN),
                        MoveClass::Inaccuracy => ("Inaccuracy", egui::Color32::YELLOW),
                        MoveClass::Mistake => ("Mistake", egui::Color32::GOLD),
                        MoveClass::Blunder => ("Blunder", egui::Color32::RED),
                    }
                } else {
                    ("N/A", egui::Color32::WHITE)
                };
                ui.colored_label(class_color, class_str);
            });

        // --- CENTRAL PANEL: THE CHESS BOARD ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Board");

            egui::Grid::new("board")
                .spacing([0.0, 0.0])
                .show(ui, |ui| {
                    for rank in 0..8 {
                        for file in 0..8 {
                            let sq = Square::from_coords(
                                shakmaty::File::new(file as u32),
                                shakmaty::Rank::new((7 - rank) as u32),
                            );

                            let dark = (rank + file) % 2 == 1;

                            let mut color = if dark {
                                egui::Color32::from_rgb(181, 136, 99)
                            } else {
                                egui::Color32::from_rgb(240, 217, 181)
                            };

                            if self.selected_square == Some(sq) {
                                color = egui::Color32::LIGHT_BLUE;
                            }

                            egui::Frame::default()
                                .fill(color)
                                .show(ui, |ui| {
                                    let text = if let Some(piece) =
                                        self.position.board().piece_at(sq)
                                    {
                                        piece_unicode(
                                            piece.role,
                                            piece.color,
                                        )
                                    } else {
                                        ""
                                    };

                                    let piece_color = if let Some(piece) =
                                        self.position.board().piece_at(sq)
                                    {
                                        match piece.color {
                                            Color::White => egui::Color32::WHITE,
                                            Color::Black => egui::Color32::BLACK,
                                        }
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };

                                    let response = ui.add_sized(
                                        [75.0, 75.0],
                                        egui::Label::new(
                                            egui::RichText::new(text)
                                                .size(50.0)
                                                .color(piece_color),
                                        )
                                        .sense(egui::Sense::click()),
                                    );

                                    if response.clicked() {
                                        match self.selected_square {
                                            None => {
                                                if self.position.board().piece_at(sq).is_some() {
                                                    self.selected_square = Some(sq);
                                                }
                                            }
                                            Some(from) => {
                                                let legal_moves = self.position.legal_moves();

                                                let found_move = legal_moves.into_iter().find(|m| {
                                                    m.from() == Some(from) &&
                                                    m.to() == sq
                                                });

                                                if let Some(mv) = found_move {
                                                    let san_str = San::from_move(&self.position, mv).to_string();

                                                    self.position = self.position.clone().play(mv).unwrap();

                                                    let new_fen = Fen::from_position(&self.position, EnPassantMode::Legal).to_string();

                                                    if self.selected_move + 1 < self.history.len() {
                                                        self.history.truncate(self.selected_move + 1);
                                                    }

                                                    let result =
                                                        self.engine.analyze_fen(&new_fen);

                                                    let class =
                                                        classify(result.eval);

                                                    self.history.push(
                                                        MoveAnalysis {
                                                            san: san_str,
                                                            fen: new_fen,
                                                            eval: result.eval,
                                                            best_move: result.best_move,
                                                            classification: class,
                                                        }
                                                    );

                                                    self.selected_move = self.history.len() - 1;
                                                }

                                                self.selected_square = None;
                                            }
                                        }
                                    }
                                });
                        }
                        ui.end_row();
                    }
                });
        });

        // --- BOTTOM PANEL: MOVE HISTORY SCROLLER ---
        egui::TopBottomPanel::bottom("moves")
            .default_height(150.0)
            .show(ctx, |ui| {
                ui.heading("Moves");

                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        for idx in 0..self.history.len() {
                            let item_san = self.history[idx].san.clone();
                            let item_eval = self.history[idx].eval;

                            if ui
                                .selectable_label(
                                    idx == self.selected_move,
                                    format!(
                                        "{}. {} ({:.2})",
                                        idx,
                                        item_san,
                                        item_eval as f32 / 100.0
                                    ),
                                )
                                .clicked()
                            {
                                self.selected_move = idx;
                                self.update_position_from_history();
                            }
                        }
                    });
            });
    }
}
fn classify(cp_loss: i32) -> MoveClass {
    match cp_loss.abs() {
        0..=20 => MoveClass::Best,
        21..=80 => MoveClass::Good,
        81..=200 => MoveClass::Inaccuracy,
        201..=400 => MoveClass::Mistake,
        _ => MoveClass::Blunder,
    }
}
