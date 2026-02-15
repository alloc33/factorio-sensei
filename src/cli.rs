use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "factorio-sensei")]
#[command(about = "AI coaching copilot for Factorio 2.x")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// RCON server address (host:port)
    #[arg(long, env = "FACTORIO_RCON_ADDR", default_value = "127.0.0.1:27015")]
    pub addr: String,

    /// RCON password
    #[arg(long, env = "FACTORIO_RCON_PASS", default_value = "factorio")]
    pub password: String,

    /// Claude model override
    #[arg(long, env = "FACTORIO_MODEL")]
    pub model: Option<String>,

    /// Enable in-game chat bridge (requires factorio-sensei mod installed)
    #[arg(long, env = "FACTORIO_BRIDGE")]
    pub bridge: bool,
}

#[derive(Subcommand, PartialEq)]
pub enum Command {
    /// Install the Factorio Sensei mod into Factorio's mods directory
    InstallMod,
}
