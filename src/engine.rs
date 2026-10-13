use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

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

    pub fn evaluate_fen(
        &mut self,
        fen: &str,
    ) -> i32 {
        self.send_command(
            &format!("position fen {fen}")
        );

        self.send_command("go depth 15");

        let mut line = String::new();
        let mut score = 0;

        loop {
            line.clear();

            if self.reader.read_line(&mut line).unwrap() == 0 {
                break;
            }

            if line.starts_with("info") {
                let parts: Vec<&str> =
                    line.split_whitespace().collect();

                if let Some(idx) =
                    parts.iter().position(|x| *x == "cp")
                {
                    if let Some(value) = parts.get(idx + 1) {
                        score = value.parse().unwrap_or(0);
                    }
                }
            }

            if line.starts_with("bestmove") {
                break;
            }
        }

        score
    }
}

impl Drop for EngineAnalyzer {
    fn drop(&mut self) {
        let _ = writeln!(self.writer, "quit");
    }
}