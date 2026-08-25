//! HFS-RS library root - core modules are UI-agnostic where possible.
#![allow(
    clippy::type_complexity,
    clippy::collapsible_if,
    clippy::too_many_arguments
)]

pub mod config;
pub mod http;
pub mod i18n;
pub mod server;
pub mod util;
pub mod vfs;

pub mod ui;

pub use config::AppConfig;
pub use server::{AppState, ServerHandle, ServerStatus};
pub use vfs::{NodeId, NodeKind, Vfs, VfsNode};
