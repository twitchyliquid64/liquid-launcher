#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod ext;
pub use app::Launcher;

pub mod eq;
pub mod sys_apps;

pub(crate) mod eqwidget;
