//! Reading the active Omarchy theme: palette from `colors.toml` and font size
//! from `shell.toml`, as the shell does.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    /// The theme's red (`red`, else `color1`).
    pub urgent: String,
    pub muted: String,
    /// The theme's yellow (`yellow`, else `color3`), for fire and sparks.
    pub warm: String,
}

impl Default for Palette {
    /// Fallback colors of the Omarchy shell.
    fn default() -> Self {
        Palette {
            background: "#101315".into(),
            foreground: "#cacccc".into(),
            accent: "#cacccc".into(),
            urgent: "#a55555".into(),
            muted: "#707880".into(),
            warm: "#c8a050".into(),
        }
    }
}

/// `key = "#rrggbb"` pairs of a TOML color file.
fn colors(text: &str) -> impl Iterator<Item = (&str, &str)> {
    text.lines().filter_map(|line| {
        let (key, value) = line.split_once('=')?;
        let value = value.trim().trim_matches(['"', '\'']);
        let hex = value.get(..7).filter(|v| v.starts_with('#'))?;
        hex[1..]
            .chars()
            .all(|c| c.is_ascii_hexdigit())
            .then_some((key.trim(), hex))
    })
}

impl Palette {
    /// Same logic as the shell: `accent` else `color4`, `muted` else `color8`, etc.
    pub fn parse(text: &str) -> Palette {
        let mut p = Palette::default();
        let (mut accent, mut muted, mut fg, mut bg) = (None, None, None, None);
        let (mut c0, mut c4, mut c7, mut c8) = (None, None, None, None);
        for (key, value) in colors(text) {
            let hex = Some(value.to_string());
            match key {
                "foreground" => fg = hex,
                "background" => bg = hex,
                "accent" => accent = hex,
                "muted" => muted = hex,
                "color0" => c0 = hex,
                "color4" => c4 = hex,
                "color7" => c7 = hex,
                "color8" => c8 = hex,
                "red" | "color1" => p.urgent = value.to_string(),
                "yellow" | "color3" => p.warm = value.to_string(),
                _ => {}
            }
        }
        if let Some(v) = bg.or(c0) {
            p.background = v;
        }
        if let Some(v) = fg.or(c7) {
            p.foreground = v;
        }
        if let Some(v) = accent.or(c4) {
            p.accent = v;
        }
        p.muted = muted.or(c8).unwrap_or_else(|| p.foreground.clone());
        p
    }
}

/// `base-size` of the `[font]` section of a `shell.toml`.
pub fn font_base_size(text: &str) -> Option<i32> {
    let mut in_font = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_font = line.starts_with("[font]");
        } else if in_font
            && let Some((key, value)) = line.split_once('=')
            && key.trim() == "base-size"
        {
            let value = value.split('#').next()?.trim().trim_matches('"');
            return value
                .parse::<f64>()
                .ok()
                .map(|v| v.round() as i32)
                .filter(|&v| v > 0);
        }
    }
    None
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
}

pub fn theme_dir() -> PathBuf {
    home().join(".local/state/omarchy/current/theme")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// What the interface takes from the theme.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub palette: Palette,
    pub font_base_size: i32,
    pub corner_radius: i32,
}

impl Theme {
    pub fn load() -> Theme {
        let dir = theme_dir();
        // The machine setting wins over the theme's, as in the shell.
        let user_shell = read(&home().join(".config/omarchy/shell.toml"));
        let theme_shell = read(&dir.join("shell.toml"));
        Theme {
            palette: Palette::parse(&read(&dir.join("colors.toml"))),
            font_base_size: font_base_size(&user_shell)
                .or_else(|| font_base_size(&theme_shell))
                .unwrap_or(12),
            corner_radius: hyprland_rounding().unwrap_or(0),
        }
    }
}

/// Hyprland window rounding (`decoration:rounding`), which the shell uses too.
fn hyprland_rounding() -> Option<i32> {
    let out = Command::new("hyprctl")
        .args(["getoption", "decoration:rounding", "-j"])
        .output()
        .ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let start = text.find("\"int\"")? + 5;
    let digits: String = text[start..]
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// Fingerprint of the theme files: changes when the theme is switched (the
/// current directory is replaced) or a file is modified.
pub fn fingerprint() -> Vec<(PathBuf, Option<SystemTime>)> {
    let dir = theme_dir();
    let files = [
        dir.join("colors.toml"),
        dir.join("shell.toml"),
        home().join(".config/omarchy/shell.toml"),
    ];
    files
        .into_iter()
        .map(|f| {
            let modified = std::fs::metadata(&f).and_then(|m| m.modified()).ok();
            (f.canonicalize().unwrap_or(f), modified)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_colors() {
        let p = Palette::parse(
            "mode = \"dark\"\naccent = \"#e8952e\"\nbackground = \"#2b1810\"\nforeground = \"#f4e4c8\"\nred = \"#d1512f\"\nmuted = \"#8a6f52\"\n",
        );
        assert_eq!(
            p,
            Palette {
                background: "#2b1810".into(),
                foreground: "#f4e4c8".into(),
                accent: "#e8952e".into(),
                urgent: "#d1512f".into(),
                muted: "#8a6f52".into(),
                warm: Palette::default().warm,
            }
        );
    }

    #[test]
    fn falls_back_to_numbered_colors() {
        let p = Palette::parse(
            "color0 = '#000000'\ncolor1 = '#ff0000'\ncolor4 = '#0000ff'\ncolor7 = '#eeeeee'\n",
        );
        assert_eq!(
            (p.background.as_str(), p.accent.as_str(), p.urgent.as_str()),
            ("#000000", "#0000ff", "#ff0000")
        );
        assert_eq!(p.muted, "#eeeeee");
    }

    #[test]
    fn empty_file_gives_defaults() {
        // Without `muted` or `color8`, the shell uses the text color.
        let expected = Palette {
            muted: Palette::default().foreground,
            ..Palette::default()
        };
        assert_eq!(Palette::parse(""), expected);
        assert_eq!(Palette::parse("accent = \"not a color\""), expected);
    }

    #[test]
    fn reads_font_base_size() {
        assert_eq!(
            font_base_size("[bar]\nbase-size = 9\n[font]\nbase-size = 14 # large\n"),
            Some(14)
        );
        assert_eq!(font_base_size("[font]\nfamily = \"x\"\n"), None);
        assert_eq!(font_base_size(""), None);
    }
}
