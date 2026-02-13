use std::{
    io::{self, BufRead, Write},
    sync::Arc,
};

use factorio_rcon::RconClient;
use factorio_sensei::agent;
use rig::completion::{Message, Prompt};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // RCON connection (env vars with defaults)
    let host = std::env::var("FACTORIO_RCON_ADDR").unwrap_or_else(|_| "127.0.0.1:27015".into());
    let pass = std::env::var("FACTORIO_RCON_PASS").unwrap_or_else(|_| "factorio".into());
    let model = std::env::var("FACTORIO_MODEL").ok();

    eprintln!("Connecting to Factorio RCON at {host}...");
    let client = RconClient::connect(&host, &pass).await?;
    let rcon = Arc::new(Mutex::new(client));
    eprintln!(
        "Connected! Model: {}. Type your questions (Ctrl+D to quit).\n",
        model.as_deref().unwrap_or(agent::DEFAULT_MODEL)
    );

    let coach = agent::build_coach(&rcon, model.as_deref());

    // Minimal REPL — Phase 4 upgrades this to reedline
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut history: Vec<Message> = Vec::new();

    loop {
        print!("You> ");
        stdout.flush()?;

        let mut input = String::new();
        if stdin.lock().read_line(&mut input)? == 0 {
            break; // EOF / Ctrl+D
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input == "/quit" {
            break;
        }

        match coach.prompt(input).with_history(&mut history).await {
            Ok(response) => {
                println!("\nSensei> {response}\n");
            }
            Err(e) => {
                eprintln!("\n[Error] {e}\n");
            }
        }
    }

    Ok(())
}
