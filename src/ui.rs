//! Tiny terminal styling helpers (no extra deps). Colors are disabled when stdout is not a TTY
//! or when `NO_COLOR` is set, so output piped into the GUI job console stays clean.

use std::io::IsTerminal;
use std::sync::OnceLock;

fn color_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal())
}

fn paint(code: &str, s: impl AsRef<str>) -> String {
    if color_enabled() {
        format!("\x1b[{code}m{}\x1b[0m", s.as_ref())
    } else {
        s.as_ref().to_string()
    }
}

pub fn bold(s: impl AsRef<str>) -> String {
    paint("1", s)
}
pub fn dim(s: impl AsRef<str>) -> String {
    paint("2", s)
}
pub fn red(s: impl AsRef<str>) -> String {
    paint("31", s)
}
pub fn green(s: impl AsRef<str>) -> String {
    paint("32", s)
}
pub fn yellow(s: impl AsRef<str>) -> String {
    paint("33", s)
}
pub fn cyan(s: impl AsRef<str>) -> String {
    paint("36", s)
}

pub fn header(s: impl AsRef<str>) {
    println!("{}", bold(cyan(format!("▸ {}", s.as_ref()))));
}
pub fn ok(s: impl AsRef<str>) {
    println!("{} {}", green("✔"), s.as_ref());
}
pub fn warn(s: impl AsRef<str>) {
    println!("{} {}", yellow("!"), s.as_ref());
}
pub fn fail(s: impl AsRef<str>) {
    println!("{} {}", red("✘"), s.as_ref());
}
pub fn info(s: impl AsRef<str>) {
    println!("{} {}", dim("·"), s.as_ref());
}

pub fn table(headers: &[&str]) -> comfy_table::Table {
    let mut t = comfy_table::Table::new();
    t.load_style(comfy_table::presets::UTF8_HORIZONTAL_ONLY);
    t.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
    t.set_header(headers.iter().map(|h| *h));
    t
}

pub fn kv(key: &str, value: impl std::fmt::Display) {
    println!("  {:<18} {}", dim(key), value);
}
