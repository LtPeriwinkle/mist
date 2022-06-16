//! Structs related to configuration of mist
mod colors;
mod config;
mod font;
mod keybinds;
mod panels;
pub use {colors::Colors, config::Config, font::Font, keybinds::KeybindsRaw, panels::Panel};
