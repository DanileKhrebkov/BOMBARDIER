// src/reporters/mod.rs
mod terminal;
mod progress;
mod json;
mod html;

pub use terminal::TerminalReporter;
pub use progress::ProgressReporter;
pub use json::JsonReporter;
pub use html::HtmlReporter;