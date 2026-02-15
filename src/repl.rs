use std::{borrow::Cow, path::PathBuf, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};
use reedline::{FileBackedHistory, Prompt, PromptEditMode, PromptHistorySearch, Reedline, Signal};
use rig::{
    agent::Agent,
    completion::{Message, Prompt as RigPrompt},
    providers::anthropic::completion::CompletionModel,
};
use termimad::{crossterm::style::Color, MadSkin};

const GREEN_BOLD: &str = "\x1b[1;32m";
const RED_BOLD: &str = "\x1b[1;31m";
const CYAN_BOLD: &str = "\x1b[1;36m";
const DIM: &str = "\x1b[2m";
const GRAY: &str = "\x1b[38;5;245m";
const RESET: &str = "\x1b[0m";

const STATUS_PROMPT: &str = "Give me a quick status overview: check my position, \
    power grid, current research, and production of iron-plate and copper-plate.";

// ── Custom prompt ──────────────────────────────────────────────

struct SenseiPrompt;

impl Prompt for SenseiPrompt {
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

// ── Markdown skin ─────────────────────────────────────────────

fn build_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(Color::White);
    skin.italic.set_fg(Color::AnsiValue(183)); // light purple
    skin.headers[0].set_fg(Color::Cyan);
    skin.headers[1].set_fg(Color::Cyan);
    skin.headers[2].set_fg(Color::Cyan);
    skin.bullet = termimad::StyledChar::from_fg_char(Color::Green, '•');
    skin.code_block.set_fg(Color::AnsiValue(222)); // warm yellow
    skin.inline_code.set_fg(Color::AnsiValue(222));
    skin
}

// ── Agent interaction ──────────────────────────────────────────

fn handle_prompt(
    rt: &tokio::runtime::Runtime,
    sensei: &Agent<CompletionModel>,
    history: &mut Vec<Message>,
    input: &str,
) {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&[
                &format!("{GRAY}⠋{RESET}"),
                &format!("{GRAY}⠙{RESET}"),
                &format!("{GRAY}⠹{RESET}"),
                &format!("{GRAY}⠸{RESET}"),
                &format!("{GRAY}⠼{RESET}"),
                &format!("{GRAY}⠴{RESET}"),
                &format!("{GRAY}⠦{RESET}"),
                &format!("{GRAY}⠧{RESET}"),
                &format!("{GRAY}⠇{RESET}"),
                &format!("{GRAY}⠏{RESET}"),
            ])
            .template("{spinner} {msg}")
            .expect("valid template"),
    );
    spinner.set_message(format!("{GRAY}Thinking...{RESET}"));
    spinner.enable_steady_tick(Duration::from_millis(80));

    match rt.block_on(async { sensei.prompt(input).with_history(history).await }) {
        Ok(response) => {
            spinner.finish_and_clear();
            let skin = build_skin();
            println!("\n{GREEN_BOLD}Sensei>{RESET}");
            skin.print_text(&response);
            println!();
        }
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("\n{RED_BOLD}[Error]{RESET} {e}\n");
        }
    }
}

// ── Public entry point ─────────────────────────────────────────

pub fn run(rt: &tokio::runtime::Runtime, sensei: &Agent<CompletionModel>) -> anyhow::Result<()> {
    let mut editor = build_editor();
    let prompt = SenseiPrompt;
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
                        handle_prompt(rt, sensei, &mut chat_history, STATUS_PROMPT);
                    }
                    _ => handle_prompt(rt, sensei, &mut chat_history, input),
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
