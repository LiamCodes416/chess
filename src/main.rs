use shakmaty::{Chess, Move, Position};
use pgn_reader::{Visitor, Reader, SanPlus};
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio, Child};
use std::ops::ControlFlow;

struct EngineAnalyzer {
    process: Child,
    writer: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
}

impl EngineAnalyzer {
    fn new(stockfish_path: &str) -> Self {
        let mut process = Command::new(stockfish_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn Stockfish binary");

        let writer = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);

        // Initialize UCI Mode
        let mut obj = Self { process, writer, reader };
        obj.send_command("uci");
        obj.wait_for_line("uciok");
        obj.send_command("isready");
        obj.wait_for_line("readyok");
        obj
    }

    fn send_command(&mut self, cmd: &str) {
        writeln!(self.writer, "{}", cmd).unwrap();
        self.writer.flush().unwrap();
    }

    fn wait_for_line(&mut self, target: &str) {
        let mut line = String::new();
        while self.reader.read_line(&mut line).unwrap() > 0 {
            if line.contains(target) { break; }
            line.clear();
        }
    }

    fn evaluate_position(&mut self, fen: &str, depth: u32) -> i32 {
        self.send_command(&format!("position fen {}", fen));
        self.send_command(&format!("go depth {}", depth));

        let mut line = String::new();
        let mut score = 0;
        while self.reader.read_line(&mut line).unwrap() > 0 {
            if line.starts_with("info") && line.contains("score cp") {
                // Parse centipawn score out of info string
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pos) = parts.iter().position(|&x| x == "cp") {
                    if let Some(val) = parts.get(pos + 1) {
                        score = val.parse::<i32>().unwrap_or(0);
                    }
                }
            }
            if line.starts_with("bestmove") {
                break;
            }
            line.clear();
        }
        score
    }
}

// Struct to track our game state through the pgn-reader Visitor pattern
struct AnalysisVisitor {
    pos: Chess,
    engine: EngineAnalyzer,
    prev_score: i32,
    move_count: u32,
}

impl Visitor for AnalysisVisitor {
    type Tags = ();
    type Movetext = ();
    type Output = ();

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        ControlFlow::Continue(())
    }

    fn san(&mut self, _movetext: &mut Self::Movetext, san_plus: SanPlus) -> ControlFlow<Self::Output> {
        self.move_count += 1;
        // Parse the SAN string into a concrete Move structure
        if let Ok(m) = san_plus.san.to_move(&self.pos) {
            let turn = self.pos.turn();
            self.pos = self.pos.clone().play(&m).expect("Illegal move encountered");
            
            // Generate FEN for Stockfish
            let fen = format!("{}", shakmaty::fen::Fen::from_position(self.pos.clone(), shakmaty::CastlingMode::Standard));
            let current_score = self.engine.evaluate_position(&fen, 10); // Lookahead depth: 10
            
            // Normalize score perspective to White
            let score_delta = if turn.is_white() {
                current_score - self.prev_score
            } else {
                self.prev_score - current_score
            };

            print!("Move {}: {} (Score: {})", self.move_count, san_plus.san, current_score);
            if score_delta < -150 {
                print!(" ❌ BLUNDER (Dropped {} centipawns)", score_delta.abs());
            } else if score_delta < -75 {
                print!(" ⚠️ MISTAKE");
            }
            println!();

            self.prev_score = current_score;
        }
        ControlFlow::Continue(())
    }
}

fn main() {
    let pgn_data = b"1. e4 e5 2. Nf3 Nc6 3. Bc4 Nd4 4. Nxe5 Qg5 5. Nxf7 Qxg2";
    
    // Change this to your local Stockfish path
    let stockfish_path = "/usr/local/bin/stockfish"; 
    let engine = EngineAnalyzer::new(stockfish_path);

    let mut visitor = AnalysisVisitor {
        pos: Chess::default(),
        engine,
        prev_score: 30, // Slight starting advantage for White
        move_count: 0,
    };

    let mut reader = Reader::new(io::Cursor::new(&pgn_data[..]));
    let _ = reader.read_game(&mut visitor);
    println!("Analysis complete.");
}
