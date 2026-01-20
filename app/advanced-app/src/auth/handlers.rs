//! Authentication HTTP Handlers
//!
//! REST API endpoints for authentication operations.

use crate::auth::models::*;
use crate::auth::service::AuthService;
use crate::extractors::{AuthUser, ClientInfo};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use validator::Validate;

/// Shared auth service state
pub type AuthState = Arc<AuthService>;

// ============================================
// Error Response Handling
// ============================================

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_code, message) = match &self {
            AuthError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                self.to_string(),
            ),
            AuthError::AccountLocked => (
                StatusCode::FORBIDDEN,
                "account_locked",
                self.to_string(),
            ),
            AuthError::AccountNotActive => (
                StatusCode::FORBIDDEN,
                "account_not_active",
                self.to_string(),
            ),
            AuthError::EmailNotVerified => (
                StatusCode::FORBIDDEN,
                "email_not_verified",
                self.to_string(),
            ),
            AuthError::InvalidToken | AuthError::TokenRevoked => (
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                self.to_string(),
            ),
            AuthError::UserNotFound => (
                StatusCode::NOT_FOUND,
                "user_not_found",
                self.to_string(),
            ),
            AuthError::EmailExists => (
                StatusCode::CONFLICT,
                "email_exists",
                self.to_string(),
            ),
            AuthError::WeakPassword => (
                StatusCode::BAD_REQUEST,
                "weak_password",
                self.to_string(),
            ),
            AuthError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "validation_error",
                msg.clone(),
            ),
            AuthError::Database(_) | AuthError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "An internal error occurred".to_string(),
            ),
        };

        (
            status,
            Json(serde_json::json!({
                "error": error_code,
                "message": message
            })),
        )
            .into_response()
    }
}

// ============================================
// Registration
// ============================================

/// POST /api/v1/auth/register
///
/// Register a new user account
pub async fn register(
    State(auth): State<AuthState>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate request
    req.validate()
        .map_err(|e| AuthError::Validation(e.to_string()))?;

    // Register user
    let user = auth.register(req).await?;

    // Create email verification token if needed
    // In production, you would send this via email
    let verification_token = auth.create_email_verification(user.id).await?;

    tracing::info!(
        user_id = %user.id,
        "User registered, verification token created"
    );

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "message": "Registration successful. Please verify your email.",
            "user": UserResponse::from(user),
            // In production, don't return this - send via email
            "verification_token": verification_token
        })),
    ))
}

// ============================================
// Login / Logout
// ============================================

/// POST /api/v1/auth/login
///
/// Authenticate user and return access/refresh tokens
pub async fn login(
    State(auth): State<AuthState>,
    ClientInfo { ip, user_agent }: ClientInfo,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    // Validate request
    req.validate()
        .map_err(|e| AuthError::Validation(e.to_string()))?;

    // Attempt login
    let response = auth.login(req, ip, user_agent).await?;

    Ok(Json(response))
}

/// POST /api/v1/auth/logout
///
/// Revoke refresh token and logout user
pub async fn logout(
    State(auth): State<AuthState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AuthError> {
    auth.logout(&req.refresh_token).await?;

    Ok(Json(MessageResponse::new("Logged out successfully")))
}

// ============================================
// Token Refresh
// ============================================

/// POST /api/v1/auth/refresh
///
/// Refresh access token using refresh token
pub async fn refresh_token(
    State(auth): State<AuthState>,
    ClientInfo { ip, user_agent }: ClientInfo,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AuthError> {
    req.validate()
        .map_err(|e| AuthError::Validation(e.to_string()))?;

    let response = auth.refresh_tokens(&req.refresh_token, ip, user_agent).await?;

    Ok(Json(response))
}

// ============================================
// Password Management
// ============================================

/// POST /api/v1/auth/forgot-password
///
/// Initiate password reset process
pub async fn forgot_password(
    State(auth): State<AuthState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<impl IntoResponse, AuthError> {
    req.validate()
        .map_err(|e| AuthError::Validation(e.to_string()))?;

    // Generate reset token
    let token = auth.forgot_password(&req.email).await?;

    // In production, send token via email, don't return it
    // Always return success to prevent email enumeration

    Ok(Json(serde_json::json!({
        "message": "If an account with that email exists, a password reset link has been sent.",
        // In production, remove this line - send via email
        "reset_token": if !token.is_empty() { Some(token) } else { None }
    })))
}

/// POST /api/v1/auth/reset-password
///
/// Complete password reset with token
pub async fn reset_password(
    State(auth): State<AuthState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, AuthError> {
    req.validate()
        .map_err(|e| AuthError::Validation(e.to_string()))?;

    auth.reset_password(req).await?;

    Ok(Json(MessageResponse::new(
        "Password reset successful. Please login with your new password.",
    )))
}

/// POST /api/v1/auth/change-password
///
/// Change password for authenticated user
pub async fn change_password(
    State(auth): State<AuthState>,
    AuthUser(user): AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AuthError> {
    req.validate()
        .map_err(|e| AuthError::Validation(e.to_string()))?;

    auth.change_password(user.id, req).await?;

    Ok(Json(MessageResponse::new(
        "Password changed successfully. Please login again on all devices.",
    )))
}

// ============================================
// Email Verification
// ============================================

/// POST /api/v1/auth/verify-email
///
/// Verify email address with token
pub async fn verify_email(
    State(auth): State<AuthState>,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<impl IntoResponse, AuthError> {
    req.validate()
        .map_err(|e| AuthError::Validation(e.to_string()))?;

    let user = auth.verify_email(&req.token).await?;

    Ok(Json(serde_json::json!({
        "message": "Email verified successfully",
        "user": UserResponse::from(user)
    })))
}

/// POST /api/v1/auth/resend-verification
///
/// Resend email verification token
pub async fn resend_verification(
    State(auth): State<AuthState>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, AuthError> {
    if user.email_verified_at.is_some() {
        return Ok(Json(serde_json::json!({
            "message": "Email is already verified"
        })));
    }

    let token = auth.create_email_verification(user.id).await?;

    // In production, send via email
    Ok(Json(serde_json::json!({
        "message": "Verification email sent",
        // In production, remove this - send via email
        "verification_token": token
    })))
}

// ============================================
// User Profile
// ============================================

/// GET /api/v1/auth/me
///
/// Get current user profile
pub async fn get_current_user(
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse, AuthError> {
    // Build response from extractors::User (derived from JWT claims)
    Ok(Json(serde_json::json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "name": user.name,
            "role": user.role,
            "email_verified": user.email_verified_at.is_some()
        }
    })))
}
