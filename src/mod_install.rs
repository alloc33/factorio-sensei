//! Installs the embedded Factorio mod into the game's mods directory.

use std::path::PathBuf;

const MOD_INFO: &str = include_str!("../factorio-mod/info.json");
const MOD_CONTROL: &str = include_str!("../factorio-mod/control.lua");
const MOD_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Detect Factorio's mods directory for the current platform.
fn mods_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|h| h.join("Library/Application Support/factorio/mods"))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::home_dir().map(|h| h.join(".factorio/mods"))
    }

    #[cfg(target_os = "windows")]
    {
        dirs::data_dir().map(|d| d.join("Factorio/mods"))
    }
}

/// Install the Factorio Sensei mod. Returns the path it was installed to.
pub fn install() -> anyhow::Result<PathBuf> {
    let base =
        mods_dir().ok_or_else(|| anyhow::anyhow!("Could not detect Factorio mods directory"))?;

    if !base.exists() {
        anyhow::bail!(
            "Factorio mods directory not found at {}\nIs Factorio installed?",
            base.display()
        );
    }

    let mod_dir = base.join(format!("factorio-sensei_{MOD_VERSION}"));

    std::fs::create_dir_all(&mod_dir)?;
    std::fs::write(mod_dir.join("info.json"), MOD_INFO)?;
    std::fs::write(mod_dir.join("control.lua"), MOD_CONTROL)?;

    Ok(mod_dir)
}
