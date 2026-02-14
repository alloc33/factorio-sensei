//! In-game chat bridge: polls `/sensei_poll` for player messages, routes them
//! through the coaching agent, and delivers responses via `/sensei_respond`.

use std::collections::HashMap;
use std::time::Duration;

use rig::{
    agent::Agent,
    completion::{Message, Prompt},
    providers::anthropic::completion::CompletionModel,
};
use serde::Deserialize;

use crate::SharedRcon;

const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// A queued message from a player's `/coach` command.
#[derive(Debug, Deserialize)]
struct CoachMessage {
    player: String,
    message: String,
}

/// Run the in-game chat bridge polling loop.
///
/// Polls the Factorio mod for unread `/coach` messages, sends each through the
/// coaching agent, and delivers responses back to game chat. Runs indefinitely
/// until the runtime shuts down.
pub async fn run(
    rcon: SharedRcon,
    coach: Agent<CompletionModel>,
    poll_interval: Duration,
) {
    let mut histories: HashMap<String, Vec<Message>> = HashMap::new();
    let mut consecutive_errors: u32 = 0;

    loop {
        match poll_messages(&rcon).await {
            Ok(messages) => {
                consecutive_errors = 0;
                for msg in messages {
                    handle_message(&rcon, &coach, &mut histories, &msg).await;
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                if consecutive_errors <= 3 {
                    eprintln!("{DIM}[Bridge] Poll error: {e}{RESET}");
                } else if consecutive_errors == 4 {
                    eprintln!(
                        "{DIM}[Bridge] Repeated errors, backing off to 10s intervals{RESET}"
                    );
                }
            }
        }

        let sleep_duration = if consecutive_errors > 3 {
            Duration::from_secs(10)
        } else {
            poll_interval
        };
        tokio::time::sleep(sleep_duration).await;
    }
}

/// Execute `/sensei_poll` and parse the JSON response into messages.
async fn poll_messages(rcon: &SharedRcon) -> Result<Vec<CoachMessage>, BridgeError> {
    let response = rcon.lock().await.execute("/sensei_poll").await?;
    let trimmed = response.trim();

    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(Vec::new());
    }

    // Check for Lua-side error
    if trimmed.contains("\"error\"") {
        return Err(BridgeError::Lua(trimmed.to_string()));
    }

    let messages: Vec<CoachMessage> = serde_json::from_str(trimmed)?;
    Ok(messages)
}

/// Send a player message through the agent and deliver the response in-game.
async fn handle_message(
    rcon: &SharedRcon,
    coach: &Agent<CompletionModel>,
    histories: &mut HashMap<String, Vec<Message>>,
    msg: &CoachMessage,
) {
    eprintln!(
        "{DIM}[Bridge] {}: {}{RESET}",
        msg.player, msg.message
    );

    let history = histories.entry(msg.player.clone()).or_default();
    let prompt = format!(
        "[In-game message from player {}] {}",
        msg.player, msg.message
    );

    match coach.prompt(&prompt).with_history(history).await {
        Ok(response) => {
            let sanitized = sanitize_for_game(&response);
            if let Err(e) = send_response(rcon, &sanitized).await {
                eprintln!("{DIM}[Bridge] Failed to send response: {e}{RESET}");
            }
        }
        Err(e) => {
            eprintln!("{DIM}[Bridge] Agent error: {e}{RESET}");
            let _ = send_response(rcon, "Sorry, I encountered an error processing your question.")
                .await;
        }
    }
}

/// Send a coaching response back to game chat via `/sensei_respond`.
async fn send_response(rcon: &SharedRcon, message: &str) -> Result<(), BridgeError> {
    let command = format!("/sensei_respond {message}");
    rcon.lock().await.execute(&command).await?;
    Ok(())
}

/// Strip markdown formatting and prepare text for Factorio's game chat.
///
/// - Strips bold markers (`**`, `__`)
/// - Removes code fences and inline backticks
/// - Strips markdown headers (`## `)
/// - Flattens newlines to ` | `
/// - Escapes square brackets (Factorio rich text)
/// - Collapses multiple spaces
/// - Truncates to 1000 characters
fn sanitize_for_game(response: &str) -> String {
    let mut result = response.to_string();

    // Strip code fences
    result = result.replace("```", "");

    // Strip bold/italic markers
    result = result.replace("**", "");
    result = result.replace("__", "");

    // Strip inline backticks
    result = result.replace('`', "");

    // Strip markdown headers
    for prefix in ["#### ", "### ", "## ", "# "] {
        result = result.replace(prefix, "");
    }

    // Escape square brackets (Factorio rich text syntax)
    result = result.replace('[', "(");
    result = result.replace(']', ")");

    // Flatten newlines to separator
    result = result
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");

    // Collapse multiple spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    // Truncate to 1000 bytes (find nearest char boundary)
    if result.len() > 1000 {
        let mut end = 1000;
        while !result.is_char_boundary(end) {
            end -= 1;
        }
        result.truncate(end);
        result.push_str("...");
    }

    result.trim().to_string()
}

// ── Error type (module-private) ──────────────────────────────────

#[derive(Debug)]
enum BridgeError {
    Rcon(factorio_rcon::RconError),
    Json(serde_json::Error),
    Lua(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rcon(e) => write!(f, "RCON: {e}"),
            Self::Json(e) => write!(f, "JSON parse: {e}"),
            Self::Lua(e) => write!(f, "Lua: {e}"),
        }
    }
}

impl From<factorio_rcon::RconError> for BridgeError {
    fn from(e: factorio_rcon::RconError) -> Self {
        Self::Rcon(e)
    }
}

impl From<serde_json::Error> for BridgeError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── sanitize_for_game tests ──────────────────────────────────

    #[test]
    fn strips_bold_markers() {
        assert_eq!(sanitize_for_game("use **bold** text"), "use bold text");
    }

    #[test]
    fn strips_code_fences() {
        let input = "before\n```lua\nprint('hi')\n```\nafter";
        let result = sanitize_for_game(input);
        assert!(result.contains("print('hi')"));
        assert!(!result.contains("```"));
    }

    #[test]
    fn strips_inline_backticks() {
        assert_eq!(sanitize_for_game("use `iron-plate`"), "use iron-plate");
    }

    #[test]
    fn strips_markdown_headers() {
        assert_eq!(sanitize_for_game("## Analysis"), "Analysis");
    }

    #[test]
    fn flattens_newlines() {
        let input = "line one\nline two\nline three";
        assert_eq!(sanitize_for_game(input), "line one | line two | line three");
    }

    #[test]
    fn truncates_long_text() {
        let long = "a".repeat(1500);
        let result = sanitize_for_game(&long);
        assert!(result.len() <= 1003); // 1000 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn collapses_whitespace() {
        assert_eq!(sanitize_for_game("too   many   spaces"), "too many spaces");
    }

    #[test]
    fn escapes_brackets() {
        assert_eq!(
            sanitize_for_game("use [item=iron-plate]"),
            "use (item=iron-plate)"
        );
    }

    #[test]
    fn empty_input() {
        assert_eq!(sanitize_for_game(""), "");
    }

    #[test]
    fn strips_blank_lines() {
        let input = "first\n\n\nsecond";
        assert_eq!(sanitize_for_game(input), "first | second");
    }

    #[test]
    fn truncates_multibyte_safely() {
        // Place a 3-byte UTF-8 char (€) right at the 1000-byte boundary
        let mut input = "a".repeat(999);
        input.push('€'); // 3 bytes — spans bytes 999..1002
        input.push_str("tail");
        let result = sanitize_for_game(&input);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 1003);
    }

    #[test]
    fn combined_markdown() {
        let input = "## Tips\n\n**First**, use `assembler`.\n\n### Next\n\nBuild more.";
        let result = sanitize_for_game(input);
        assert!(!result.contains("**"));
        assert!(!result.contains("##"));
        assert!(!result.contains('`'));
        assert!(result.contains("First"));
        assert!(result.contains("assembler"));
    }

    // ── CoachMessage deserialization tests ────────────────────────

    #[test]
    fn deserialize_single_message() {
        let json = r#"[{"player":"nick","message":"what now?","tick":12345}]"#;
        let msgs: Vec<CoachMessage> = serde_json::from_str(json).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].player, "nick");
        assert_eq!(msgs[0].message, "what now?");
    }

    #[test]
    fn deserialize_multiple_messages() {
        let json = r#"[
            {"player":"alice","message":"help","tick":100},
            {"player":"bob","message":"tips?","tick":200}
        ]"#;
        let msgs: Vec<CoachMessage> = serde_json::from_str(json).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[1].player, "bob");
    }

    #[test]
    fn deserialize_empty_array() {
        let json = "[]";
        let msgs: Vec<CoachMessage> = serde_json::from_str(json).unwrap();
        assert!(msgs.is_empty());
    }
}
