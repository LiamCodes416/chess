use eframe::egui;
use shakmaty::Chess;
use shakmaty::{Color, Role, Square};
use shakmaty::Position;

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

pub struct ChessApp {
    evaluations: Vec<i32>,
    pgn_text: String,
    selected_move: usize,
    position: Chess,
    engine: EngineAnalyzer,
    current_eval: i32
}

impl Default for ChessApp {
    fn default() -> Self {
        Self {
            evaluations: vec![30, 42, 38, 31, 20, 106, 9, -80, -738, -605],
            pgn_text: String::new(),
            selected_move: 0,
            position: Chess::default(),
            engine: EngineAnalyzer::new("stockfish-windows-x86-64-universal"),
            current_eval: 0,
        }
    }
}

impl eframe::App for ChessApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.button("Load PGN");
                if ui.button("Analyze").clicked() {
                    let score = self.engine.evaluate_fen(
                        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
                    );

                    println!("Stockfish: {}", score);
                }
            });
        });

        egui::SidePanel::right("analysis")
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Analysis");

                if let Some(eval) =
                    self.evaluations.get(self.selected_move)
                {
                    ui.label(format!(
                        "Evaluation: {:.2}",
                        *eval as f32 / 100.0
                    ));
                }

                ui.separator();
                ui.label("Best Move:");
                ui.monospace("Nf3");

                ui.separator();
                ui.label("Classification:");
                ui.colored_label(
                    egui::Color32::RED,
                    "Blunder",
                );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Board");

            egui::Grid::new("board")
                .spacing([0.0, 0.0])
                .show(ui, |ui| {
                    for rank in 0..8 {
                        for file in 0..8 {
                            let dark = (rank + file) % 2 == 1;

                            let color = if dark {
                                egui::Color32::from_rgb(
                                    181,
                                    136,
                                    99,
                                )
                            } else {
                                egui::Color32::from_rgb(
                                    240,
                                    217,
                                    181,
                                )
                            };

                            
                            egui::Frame::default()
                            .fill(color)
                            .show(ui, |ui| {
                                let sq = Square::from_coords(
                                    shakmaty::File::new(file as u32),
                                    shakmaty::Rank::new((7 - rank) as u32),
                                );

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
                                    [50.0, 50.0],
                                    egui::Label::new(
                                        egui::RichText::new(text)
                                            .size(36.0)
                                            .color(piece_color),
                                    )
                                    .sense(egui::Sense::click()),
                                );

                                if response.clicked() {
                                    println!("Square clicked: {:?}", sq);
                                }
                            });
                    }
                    ui.end_row();
                }
            });
        });

        egui::TopBottomPanel::bottom("moves")
            .default_height(150.0)
            .show(ctx, |ui| {
                ui.heading("Moves");

                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        for (idx, eval) in
                            self.evaluations.iter().enumerate()
                        {
                            if ui
                                .selectable_label(
                                    idx == self.selected_move,
                                    format!(
                                        "Move {} ({})",
                                        idx + 1,
                                        eval
                                    ),
                                )
                                .clicked()
                            {
                                self.selected_move = idx;
                            }
                        }
                    });
            });
    }
}