//! The engine as a process: reads JSON commands on stdin (one per line)
//! and writes the responses on stdout.

use omatro_engine::protocol::Session;
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let mut session = Session::from_env();
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
