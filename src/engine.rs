use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

pub struct EngineResult {
    pub eval: i32,
    pub best_move: String,
}

pub struct EngineAnalyzer {
    _process: Child,
    writer: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
}

impl EngineAnalyzer {
    pub fn new(stockfish_path: &str) -> Self {
        let mut process = Command::new(stockfish_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to launch Stockfish");

        let writer = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();

        let mut engine = Self {
            _process: process,
            writer,
            reader: BufReader::new(stdout),
        };

        engine.send_command("uci");
        engine.wait_for("uciok");

        engine.send_command("isready");
        engine.wait_for("readyok");

        engine
    }

    fn send_command(&mut self, command: &str) {
        writeln!(self.writer, "{command}").unwrap();
        self.writer.flush().unwrap();
    }

    fn wait_for(&mut self, target: &str) {
        let mut line = String::new();

        loop {
            line.clear();

            if self.reader.read_line(&mut line).unwrap() == 0 {
                break;
            }

            if line.contains(target) {
                break;
            }
        }
    }

    pub fn analyze_fen(
        &mut self,
        fen: &str,
    ) -> EngineResult {
        self.send_command(
            &format!("position fen {fen}")
        );

        self.send_command("go depth 15");

        let mut eval = 0;
        let mut best_move = String::new();
        let mut line = String::new();

        loop {
            line.clear();

            if self.reader.read_line(&mut line).unwrap() == 0 {
                break;
            }

            if line.contains(" score cp ") {
                let parts: Vec<&str> =
                    line.split_whitespace().collect();

                if let Some(idx) =
                    parts.iter().position(|x| *x == "cp")
                {
                    if let Some(val) = parts.get(idx + 1) {
                        eval = val.parse().unwrap_or(0);
                    }
                }
            }

            if line.starts_with("bestmove") {
                let parts: Vec<&str> =
                    line.split_whitespace().collect();

                best_move =
                    parts.get(1)
                        .unwrap_or(&"")
                        .to_string();

                break;
            }
        }

        EngineResult {
            eval,
            best_move,
        }
    }
}

impl Drop for EngineAnalyzer {
    fn drop(&mut self) {
        let _ = writeln!(self.writer, "quit");
    }
}