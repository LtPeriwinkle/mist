//! Parse [Run](crate::run::Run)s from their file representations.
mod msf;
pub use msf::MsfParser;
mod lss;
pub use lss::LssParser;
