//! Authentication Service
//!
//! Core authentication logic including:
//! - Password hashing and verification
//! - JWT token generation and validation
//! - User management
//! - Refresh token rotation

use crate::auth::config::AuthConfig;
use crate::auth::models::*;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// Authentication service
pub struct AuthService {
    db: PgPool,
    config: AuthConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    /// Create a new auth service instance
    pub fn new(db: PgPool, config: AuthConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());

        Self {
            db,
            config,
            encoding_key,
            decoding_key,
        }
    }

    // ============================================
    // User Registration
    // ============================================

    /// Register a new user
    pub async fn register(&self, req: RegisterRequest) -> Result<User, AuthError> {
        // Check if email already exists
        let existing = self.find_user_by_email(&req.email).await?;
        if existing.is_some() {
            return Err(AuthError::EmailExists);
        }

        // Validate password strength
        self.validate_password(&req.password)?;

        // Hash password
        let password_hash = self.hash_password(&req.password)?;

        // Determine initial status
        let status = if self.config.require_email_verification {
            UserStatus::Pending
        } else {
            UserStatus::Active
        };

        // Insert user
        let user: User = sqlx::query_as(
            r#"
            INSERT INTO users (email, password_hash, name, role, status)
            VALUES ($1, $2, $3, 'user', $4)
            RETURNING *
            "#,
        )
        .bind(&req.email.to_lowercase())
        .bind(&password_hash)
        .bind(&req.name)
        .bind(&status)
        .fetch_one(&self.db)
        .await?;

        tracing::info!(user_id = %user.id, email = %user.email, "User registered");

        Ok(user)
    }

    // ============================================
    // User Login
    // ============================================

    /// Authenticate user and return tokens
    pub async fn login(
        &self,
        req: LoginRequest,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResponse, AuthError> {
        // Find user by email
        let user = self
            .find_user_by_email(&req.email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Check if account is locked
        if user.is_locked() {
            tracing::warn!(user_id = %user.id, "Login attempt on locked account");
            return Err(AuthError::AccountLocked);
        }

        // Check if account is active
        if user.status != UserStatus::Active {
            if user.status == UserStatus::Pending && self.config.require_email_verification {
                return Err(AuthError::EmailNotVerified);
            }
            return Err(AuthError::AccountNotActive);
        }

        // Verify password
        if !self.verify_password(&req.password, &user.password_hash)? {
            // Increment failed attempts
            self.increment_failed_attempts(&user).await?;
            return Err(AuthError::InvalidCredentials);
        }

        // Reset failed attempts and update login info
        self.record_successful_login(&user, ip_address.as_deref()).await?;

        // Generate tokens
        let (access_token, refresh_token) = self
            .generate_token_pair(&user, ip_address, user_agent)
            .await?;

        Ok(AuthResponse {
            user: user.into(),
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_expiration,
        })
    }

    // ============================================
    // Token Management
    // ============================================

    /// Refresh access token using refresh token
    pub async fn refresh_tokens(
        &self,
        refresh_token: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<TokenResponse, AuthError> {
        // Decode and validate refresh token
        let claims = self.decode_refresh_token(refresh_token)?;

        // Find refresh token in database
        let token_hash = self.hash_token(refresh_token);
        let stored_token: RefreshToken = sqlx::query_as(
            "SELECT * FROM refresh_tokens WHERE token_hash = $1 AND user_id = $2",
        )
        .bind(&token_hash)
        .bind(claims.sub)
        .fetch_optional(&self.db)
        .await?
        .ok_or(AuthError::InvalidToken)?;

        // Validate token
        if stored_token.is_revoked() {
            tracing::warn!(
                token_id = %stored_token.id,
                user_id = %stored_token.user_id,
                "Attempt to use revoked refresh token"
            );
            // Revoke all tokens for this user (possible token theft)
            self.revoke_all_user_tokens(stored_token.user_id).await?;
            return Err(AuthError::TokenRevoked);
        }

        if stored_token.is_expired() {
            return Err(AuthError::InvalidToken);
        }

        // Get user
        let user = self
            .find_user_by_id(claims.sub)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        // Check if user can still login
        if !user.can_login() {
            return Err(AuthError::AccountNotActive);
        }

        // Revoke old refresh token
        self.revoke_refresh_token(&stored_token, None).await?;

        // Generate new token pair
        let (access_token, new_refresh_token) = self
            .generate_token_pair(&user, ip_address, user_agent)
            .await?;

        // Link old token to new one (for audit trail)
        sqlx::query("UPDATE refresh_tokens SET replaced_by = $1 WHERE id = $2")
            .bind(claims.tid)
            .bind(stored_token.id)
            .execute(&self.db)
            .await?;

        Ok(TokenResponse {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_expiration,
        })
    }

    /// Logout user by revoking refresh token
    pub async fn logout(&self, refresh_token: &str) -> Result<(), AuthError> {
        let token_hash = self.hash_token(refresh_token);

        sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE token_hash = $1")
            .bind(&token_hash)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Validate access token and return claims
    pub fn validate_access_token(&self, token: &str) -> Result<AccessTokenClaims, AuthError> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.config.jwt_issuer]);
        validation.set_audience(&[&self.config.jwt_audience]);

        let token_data = decode::<AccessTokenClaims>(token, &self.decoding_key, &validation)?;

        Ok(token_data.claims)
    }

    // ============================================
    // Password Management
    // ============================================

    /// Change user password
    pub async fn change_password(
        &self,
        user_id: Uuid,
        req: ChangePasswordRequest,
    ) -> Result<(), AuthError> {
        let user = self
            .find_user_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        // Verify current password
        if !self.verify_password(&req.current_password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Validate new password
        self.validate_password(&req.new_password)?;

        // Hash new password
        let password_hash = self.hash_password(&req.new_password)?;

        // Update password
        sqlx::query(
            "UPDATE users SET password_hash = $1, password_changed_at = NOW() WHERE id = $2",
        )
        .bind(&password_hash)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Revoke all refresh tokens (force re-login on all devices)
        self.revoke_all_user_tokens(user_id).await?;

        tracing::info!(user_id = %user_id, "Password changed");

        Ok(())
    }

    /// Initiate password reset
    pub async fn forgot_password(&self, email: &str) -> Result<String, AuthError> {
        let user = self.find_user_by_email(email).await?;

        // Always return success to prevent email enumeration
        let Some(user) = user else {
            return Ok(String::new());
        };

        // Generate reset token
        let token = self.generate_random_token();
        let token_hash = self.hash_token(&token);
        let expires_at =
            Utc::now() + Duration::seconds(self.config.password_reset_expiration);

        // Store token
        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user.id)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(&self.db)
        .await?;

        tracing::info!(user_id = %user.id, "Password reset requested");

        Ok(token)
    }

    /// Complete password reset
    pub async fn reset_password(&self, req: ResetPasswordRequest) -> Result<(), AuthError> {
        let token_hash = self.hash_token(&req.token);

        // Find valid token
        let token_record: Option<(Uuid, Uuid)> = sqlx::query_as(
            r#"
            SELECT id, user_id FROM password_reset_tokens
            WHERE token_hash = $1 AND expires_at > NOW() AND used_at IS NULL
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(&self.db)
        .await?;

        let (token_id, user_id) = token_record.ok_or(AuthError::InvalidToken)?;

        // Validate new password
        self.validate_password(&req.password)?;

        // Hash new password
        let password_hash = self.hash_password(&req.password)?;

        // Update password
        sqlx::query(
            "UPDATE users SET password_hash = $1, password_changed_at = NOW() WHERE id = $2",
        )
        .bind(&password_hash)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Mark token as used
        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1")
            .bind(token_id)
            .execute(&self.db)
            .await?;

        // Revoke all refresh tokens
        self.revoke_all_user_tokens(user_id).await?;

        tracing::info!(user_id = %user_id, "Password reset completed");

        Ok(())
    }

    // ============================================
    // Email Verification
    // ============================================

    /// Create email verification token
    pub async fn create_email_verification(&self, user_id: Uuid) -> Result<String, AuthError> {
        let token = self.generate_random_token();
        let token_hash = self.hash_token(&token);
        let expires_at =
            Utc::now() + Duration::seconds(self.config.email_verification_expiration);

        sqlx::query(
            r#"
            INSERT INTO email_verification_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user_id)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(&self.db)
        .await?;

        Ok(token)
    }

    /// Verify email with token
    pub async fn verify_email(&self, token: &str) -> Result<User, AuthError> {
        let token_hash = self.hash_token(token);

        // Find valid token
        let token_record: Option<(Uuid, Uuid)> = sqlx::query_as(
            r#"
            SELECT id, user_id FROM email_verification_tokens
            WHERE token_hash = $1 AND expires_at > NOW() AND verified_at IS NULL
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(&self.db)
        .await?;

        let (token_id, user_id) = token_record.ok_or(AuthError::InvalidToken)?;

        // Update user and token
        let user: User = sqlx::query_as(
            r#"
            UPDATE users SET email_verified_at = NOW(), status = 'active'
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        sqlx::query("UPDATE email_verification_tokens SET verified_at = NOW() WHERE id = $1")
            .bind(token_id)
            .execute(&self.db)
            .await?;

        tracing::info!(user_id = %user_id, "Email verified");

        Ok(user)
    }

    // ============================================
    // Helper Methods
    // ============================================

    /// Find user by email
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AuthError> {
        let user = sqlx::query_as("SELECT * FROM users WHERE email = $1")
            .bind(email.to_lowercase())
            .fetch_optional(&self.db)
            .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>, AuthError> {
        let user = sqlx::query_as("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db)
            .await?;

        Ok(user)
    }

    /// Hash password using Argon2
    fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);

        let params = Params::new(
            self.config.argon2_memory_cost,
            self.config.argon2_time_cost,
            self.config.argon2_parallelism,
            None,
        )
        .map_err(|_| AuthError::Internal)?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

        let hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        Ok(hash)
    }

    /// Verify password against hash
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(hash).map_err(|_| AuthError::Internal)?;

        let argon2 = Argon2::default();

        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Validate password strength
    fn validate_password(&self, password: &str) -> Result<(), AuthError> {
        if password.len() < self.config.min_password_length {
            return Err(AuthError::WeakPassword);
        }

        // Additional checks can be added here:
        // - Require uppercase/lowercase
        // - Require numbers
        // - Require special characters
        // - Check against common passwords

        Ok(())
    }

    /// Generate access and refresh token pair
    async fn generate_token_pair(
        &self,
        user: &User,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(String, String), AuthError> {
        let now = Utc::now();
        let access_exp = now + Duration::seconds(self.config.access_token_expiration);
        let refresh_exp = now + Duration::seconds(self.config.refresh_token_expiration);

        // Generate access token
        let access_claims = AccessTokenClaims {
            sub: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
            role: format!("{:?}", user.role).to_lowercase(),
            iat: now.timestamp(),
            exp: access_exp.timestamp(),
            iss: self.config.jwt_issuer.clone(),
            aud: self.config.jwt_audience.clone(),
            jti: Uuid::new_v4(),
        };

        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)?;

        // Generate refresh token
        let refresh_token_id = Uuid::new_v4();
        let refresh_claims = RefreshTokenClaims {
            sub: user.id,
            tid: refresh_token_id,
            iat: now.timestamp(),
            exp: refresh_exp.timestamp(),
            iss: self.config.jwt_issuer.clone(),
        };

        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)?;
        let token_hash = self.hash_token(&refresh_token);

        // Store refresh token
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, user_agent, ip_address)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(refresh_token_id)
        .bind(user.id)
        .bind(&token_hash)
        .bind(refresh_exp)
        .bind(&user_agent)
        .bind(&ip_address)
        .execute(&self.db)
        .await?;

        Ok((access_token, refresh_token))
    }

    /// Decode refresh token claims
    fn decode_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, AuthError> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.config.jwt_issuer]);

        let token_data = decode::<RefreshTokenClaims>(token, &self.decoding_key, &validation)?;

        Ok(token_data.claims)
    }

    /// Hash token for storage (SHA256)
    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Generate random token
    fn generate_random_token(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        hex::encode(bytes)
    }

    /// Increment failed login attempts
    async fn increment_failed_attempts(&self, user: &User) -> Result<(), AuthError> {
        let new_attempts = user.failed_login_attempts + 1;

        let locked_until = if new_attempts >= self.config.max_login_attempts {
            Some(Utc::now() + Duration::seconds(self.config.lockout_duration))
        } else {
            None
        };

        sqlx::query(
            "UPDATE users SET failed_login_attempts = $1, locked_until = $2 WHERE id = $3",
        )
        .bind(new_attempts)
        .bind(locked_until)
        .bind(user.id)
        .execute(&self.db)
        .await?;

        if locked_until.is_some() {
            tracing::warn!(user_id = %user.id, "Account locked due to failed attempts");
        }

        Ok(())
    }

    /// Record successful login
    async fn record_successful_login(
        &self,
        user: &User,
        ip_address: Option<&str>,
    ) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            UPDATE users SET
                failed_login_attempts = 0,
                locked_until = NULL,
                last_login_at = NOW(),
                last_login_ip = $1
            WHERE id = $2
            "#,
        )
        .bind(ip_address)
        .bind(user.id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Revoke a specific refresh token
    async fn revoke_refresh_token(
        &self,
        token: &RefreshToken,
        replaced_by: Option<Uuid>,
    ) -> Result<(), AuthError> {
        sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW(), replaced_by = $1 WHERE id = $2")
            .bind(replaced_by)
            .bind(token.id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Revoke all refresh tokens for a user
    async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
        )
        .bind(user_id)
        .execute(&self.db)
        .await?;

        tracing::info!(user_id = %user_id, "All refresh tokens revoked");

        Ok(())
    }
}
