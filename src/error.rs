use std::{env, fmt::Display};

use axum::{http::StatusCode, response::{IntoResponse, Response}};


#[derive(Debug)]
pub enum AppError {
    Anyhow(anyhow::Error),
    Sqlx(sqlx::Error),
    InvalidFormatDescription(time::error::InvalidFormatDescription),
    Parse(time::error::Parse),
    VarError(env::VarError),
    IoError(std::io::Error)
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Anyhow(error) => error.fmt(f),
            AppError::Sqlx(error) => error.fmt(f),
            AppError::InvalidFormatDescription(error) => error.fmt(f),
            AppError::Parse(error) => error.fmt(f),
            AppError::VarError(error) => error.fmt(f),
            AppError::IoError(error) => error.fmt(f),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Can't do what we wanted...: {self}"),
        )
            .into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::Anyhow(err)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::Sqlx(err)
    }
}

impl From<time::error::InvalidFormatDescription> for AppError {
    fn from(err: time::error::InvalidFormatDescription) -> Self {
        Self::InvalidFormatDescription(err)
    }
}

impl From<time::error::Parse> for AppError {
    fn from(err: time::error::Parse) -> Self {
        Self::Parse(err)
    }
}

impl From<env::VarError> for AppError {
    fn from(err: env::VarError) -> Self {
        Self::VarError(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}