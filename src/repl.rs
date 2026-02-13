use std::{borrow::Cow, path::PathBuf};

use reedline::{FileBackedHistory, Prompt, PromptEditMode, PromptHistorySearch, Reedline, Signal};
use rig::{
    agent::Agent,
    completion::{Message, Prompt as RigPrompt},
    providers::anthropic::completion::CompletionModel,
};

const GREEN_BOLD: &str = "\x1b[1;32m";
const RED_BOLD: &str = "\x1b[1;31m";
const CYAN_BOLD: &str = "\x1b[1;36m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

const STATUS_PROMPT: &str = "Give me a quick status overview: check my position, \
    power grid, current research, and production of iron-plate and copper-plate.";

// ── Custom prompt ──────────────────────────────────────────────

struct CoachPrompt;

impl Prompt for CoachPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Borrowed("\x1b[1;36mYou\x1b[0m")
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("> ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed(".. ")
    }

    fn render_prompt_history_search_indicator(&self, _search: PromptHistorySearch) -> Cow<'_, str> {
        Cow::Borrowed("(search)> ")
    }
}

// ── History ────────────────────────────────────────────────────

fn history_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".factorio-sensei")
        .join("history.txt")
}

fn build_editor() -> Reedline {
    match FileBackedHistory::with_file(1000, history_path()) {
        Ok(history) => Reedline::create().with_history(Box::new(history)),
        Err(e) => {
            eprintln!(
                "{DIM}Warning: could not open history file ({e}), using in-memory history{RESET}"
            );
            Reedline::create()
        }
    }
}

// ── Slash commands ─────────────────────────────────────────────

fn print_help() {
    println!(
        "\n{CYAN_BOLD}Factorio Sensei{RESET} — AI coaching copilot\n\n\
         {DIM}Commands:{RESET}\n  \
         /help    Show this help message\n  \
         /status  Quick game state overview\n  \
         /clear   Clear conversation history\n  \
         /quit    Exit (or Ctrl+D)\n\n\
         Ask anything about your factory and Sensei will check your game state.\n"
    );
}

// ── Agent interaction ──────────────────────────────────────────

fn handle_prompt(
    rt: &tokio::runtime::Runtime,
    coach: &Agent<CompletionModel>,
    history: &mut Vec<Message>,
    input: &str,
) {
    match rt.block_on(async { coach.prompt(input).with_history(history).await }) {
        Ok(response) => println!("\n{GREEN_BOLD}Sensei>{RESET} {response}\n"),
        Err(e) => eprintln!("\n{RED_BOLD}[Error]{RESET} {e}\n"),
    }
}

// ── Public entry point ─────────────────────────────────────────

pub fn run(rt: &tokio::runtime::Runtime, coach: &Agent<CompletionModel>) -> anyhow::Result<()> {
    let mut editor = build_editor();
    let prompt = CoachPrompt;
    let mut chat_history: Vec<Message> = Vec::new();

    loop {
        match editor.read_line(&prompt) {
            Ok(Signal::Success(input)) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                match input {
                    "/quit" => break,
                    "/help" => print_help(),
                    "/clear" => {
                        chat_history.clear();
                        println!("{DIM}Conversation history cleared.{RESET}");
                    }
                    "/status" => {
                        handle_prompt(rt, coach, &mut chat_history, STATUS_PROMPT);
                    }
                    _ => handle_prompt(rt, coach, &mut chat_history, input),
                }
            }
            Ok(Signal::CtrlD | Signal::CtrlC) => break,
            Err(e) => {
                eprintln!("{RED_BOLD}[Error]{RESET} {e}");
                break;
            }
        }
    }

    Ok(())
}
