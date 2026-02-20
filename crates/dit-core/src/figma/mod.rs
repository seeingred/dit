//! Figma integration for DIT.
//!
//! This module provides:
//! - [`downloader`] — Download .fig files via Playwright browser automation
//! - [`fig_converter`] — Convert .fig files to DIT snapshots via fig2json

pub mod downloader;
pub mod fig_converter;

pub use downloader::{augment_node_path, download_fig_file, resolve_command, setup_downloader, FigmaAuth};
pub use fig_converter::fig_to_snapshot;
