//! Casos de uso. Cada um é uma struct que recebe ports por construtor (DIP)
//! e expõe um único método `execute`.

#![forbid(unsafe_code)]

pub mod error;
pub mod use_cases;

pub use error::AppError;
