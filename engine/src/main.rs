//! Le moteur en processus : lit des commandes JSON sur stdin (une par ligne)
//! et écrit les réponses sur stdout.

use omatro_engine::protocol::Session;
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let mut session = Session::default();
    let mut out = io::stdout().lock();
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        serde_json::to_writer(&mut out, &session.handle(&line))?;
        writeln!(out)?;
        out.flush()?;
    }
    Ok(())
}
