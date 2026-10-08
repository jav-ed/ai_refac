//! What every language-server backend shares: the protocol client and session,
//! the text helpers for positions and edits, and the symbol-rename engine.

pub mod client;
pub mod rename;
pub mod session;
pub mod text;
