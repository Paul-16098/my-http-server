//! Test module - Central organization for all test submodules
//!
//! This module coordinates tests for different aspects of the my-http-server application:
//! - Config loading, precedence, and fixtures
//! - Markdown parsing and templating logic
//! - Full HTTP integration tests
//! - Security tests (path traversal, auth, IP filtering)
//!
//! WHY: Organize tests by functional area matching the copilot-instructions.md structure,
//! making it easy to navigate and extend test coverage for specific features.

pub(crate) mod config;
pub(crate) mod integration;
pub(crate) mod parser;
pub(crate) mod security;
