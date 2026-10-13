mod ui;
mod engine;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Chess Analyzer",
        options,
        Box::new(|_cc| Ok(Box::new(ui::ChessApp::default()))),
    )
}