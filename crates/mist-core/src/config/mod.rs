//! Structs related to configuration of mist
mod cfg;
mod colors;
mod font;
mod keybinds;
mod panels;
pub use {cfg::Config, colors::Colors, font::Font, keybinds::KeybindsRaw, panels::Panel};
