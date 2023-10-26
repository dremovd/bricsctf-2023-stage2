use crate::app::{self, JsonError, LoggedError, Validate, ValidatedJson};
use crate::rng::APP_RNG;
use crate::session::Session;

use anyhow::{Context, Result};
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;
use axum::{extract::State, http::StatusCode, Extension};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;

static USERNAME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9][a-z0-9_-]+[a-z0-9]$").expect("failed to construct username regex")
});

#[derive(Clone, Deserialize)]
pub(super) struct RegistrationRequest {
    username: String,
    password: String,
}

impl Validate for RegistrationRequest {
    fn validate(&self) -> Result<(), String> {
        if !USERNAME_RE.is_match(&self.username) {
            return Err("We currently allow usernames consisting only of lowercase english letters, numbers, and dashes/underscores in between the rest! Sorry!".into());
        } else if self.username.len() < 5 {
            return Err(
                "Your username is too short! Please make it at least 5 characters long.".into(),
            );
        } else if self.username.len() > 15 {
            return Err(
                "Your username is too long! Please shorten it to 15 characters or less.".into(),
            );
        }

        if self.password.len() < 8 {
            return Err("Please lengthen your password to at least 8 characters, it is dangerously short right now!".into());
        } else if self.password.len() > 30 {
            return Err(
                "Your password is very long, please shorten it to 30 characters or less.".into(),
            );
        }

        Ok(())
    }
}

/// Handler implementing the POST /register API endpoint.
pub(super) async fn handler(
    State(state): State<app::State>,
    ValidatedJson(request): ValidatedJson<RegistrationRequest>,
) -> Result<(StatusCode, Result<Extension<Session>, JsonError>), LoggedError> {
    let salt = APP_RNG.with_borrow_mut(|rng| SaltString::generate(rng));
    let password_hash = Argon2::default()
        .hash_password(request.password.as_bytes(), &salt)
        .map_err(anyhow::Error::msg)
        .with_context(|| "hashing password")?;

    let created = state
        .repository
        .create_account(&request.username, &password_hash.to_string())
        .await
        .with_context(|| "creating account")?;

    if !created {
        Ok((StatusCode::CONFLICT, Err(JsonError::new("Sorry, but someone has beaten you to the punch and taken your username! You should choose another one."))))
    } else {
        Ok((
            StatusCode::CREATED,
            Ok(Extension(Session {
                username: request.username,
            })),
        ))
    }
}
