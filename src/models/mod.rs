//! Módulo raíz de modelos del dominio.
//!
//! Re-exporta todos los tipos públicos para que los handlers puedan
//! importarlos con `use crate::models::*` sin conocer la estructura interna.

pub mod cocktail;
pub mod ingredient;
pub mod payloads;
pub mod rows;

#[cfg(test)]
mod tests;

pub use cocktail::*;
pub use ingredient::*;
pub use payloads::*;
pub use rows::*;
