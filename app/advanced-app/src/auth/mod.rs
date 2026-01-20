//! Authentication Module
//!
//! Provides complete authentication functionality including:
//! - User registration and login
//! - JWT token generation and validation
//! - Password hashing with Argon2
//! - Refresh token rotation
//! - Password reset flow

pub mod models;
pub mod service;
pub mod handlers;
pub mod config;

pub use models::*;
pub use service::AuthService;
pub use config::AuthConfig;
