#[cfg(feature = "config")]
pub mod config;
#[cfg(feature = "dialogs")]
pub mod dialogs;
pub mod parse;
mod run;
pub mod timing;
pub use run::Run;
#[cfg(feature = "instant")]
mod instant;
#[cfg(feature = "instant")]
pub use instant::MistInstant;
#[cfg(not(feature = "instant"))]
pub use std::time::Instant as MistInstant;
