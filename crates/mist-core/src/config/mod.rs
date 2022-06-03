//! Structs related to configuration of mist
mod colors;
mod config;
mod font;
mod keybinds;
mod layout;
mod panels;
pub use {
    colors::Colors, config::Config, font::Font, keybinds::KeybindsRaw, layout::LayoutOpts,
    panels::Panel,
};
