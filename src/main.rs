mod cli;
mod mod_install;
mod repl;

use std::sync::Arc;

use clap::Parser;
use factorio_rcon::RconClient;
use factorio_sensei::agent;
use tokio::sync::Mutex;

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const GREEN_BOLD: &str = "\x1b[1;32m";

fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let cli = cli::Cli::parse();

    if cli.command == Some(cli::Command::InstallMod) {
        let path = mod_install::install()?;
        println!("{GREEN_BOLD}Mod installed to:{RESET} {}", path.display());
        println!("{DIM}Restart Factorio and enable the mod in your save.{RESET}");
        return Ok(());
    }

    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("\x1b[1;31mError:{RESET} ANTHROPIC_API_KEY not set.\n");
        eprintln!("Get your API key at: https://console.anthropic.com/settings/keys\n");
        eprintln!("Then either:");
        eprintln!("  export ANTHROPIC_API_KEY=sk-ant-...");
        eprintln!("  or create a .env file with: ANTHROPIC_API_KEY=sk-ant-...\n");
        std::process::exit(1);
    }

    let rt = tokio::runtime::Runtime::new()?;

    eprintln!("{DIM}Connecting to Factorio RCON at {}...{RESET}", cli.addr);
    let rcon = rt.block_on(async {
        let client = RconClient::connect(&cli.addr, &cli.password).await?;
        Ok::<_, anyhow::Error>(Arc::new(Mutex::new(client)))
    })?;

    let mut wiki_articles = factorio_sensei::knowledge::builtin_articles();
    let wiki_dir = std::path::Path::new("data/wiki");
    if wiki_dir.exists() {
        match factorio_sensei::knowledge::load_wiki_articles(wiki_dir) {
            Ok(extra) => wiki_articles.extend(extra),
            Err(e) => {
                eprintln!("{DIM}Warning: could not load extra wiki ({e}){RESET}");
            }
        }
    }
    eprintln!(
        "{DIM}Loaded {} knowledge article(s).{RESET}",
        wiki_articles.len()
    );

    let model_name = cli.model.as_deref().unwrap_or(agent::DEFAULT_MODEL);
    eprintln!("{DIM}Connected! Model: {model_name}. Type /help for commands.{RESET}\n");

    let _rt_guard = rt.enter();
    let sensei = agent::build_sensei(&rcon, cli.model.as_deref(), &wiki_articles);

    if cli.bridge {
        let bridge_rcon = rcon.clone();
        let bridge_sensei = sensei.clone();
        rt.spawn(async move {
            factorio_sensei::bridge::run(
                bridge_rcon,
                bridge_sensei,
                std::time::Duration::from_secs(2),
            )
            .await;
        });
        eprintln!("{DIM}In-game /sensei bridge enabled.{RESET}");
    }

    repl::run(&rt, &sensei)
}
