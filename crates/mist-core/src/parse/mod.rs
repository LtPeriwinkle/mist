//! Parse [`Runs`](crate::run::Run) from their file representations.
mod msf;
pub use msf::MsfParser;

#[cfg(feature = "lss")]
mod lss;
#[cfg(feature = "lss")]
pub use lss::LssParser;
