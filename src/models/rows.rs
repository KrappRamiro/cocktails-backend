//! Structs para deserializar filas crudas de D1.
//!
//! Estos tipos NO se serializan a JSON — son intermedios entre la DB y los
//! modelos de dominio. Solo implementan `Deserialize`.

use crate::models::cocktail::{CocktailBase, CocktailTaste, GlassType};
use crate::models::ingredient::IngredientCategory;
use serde::Deserialize;

/// Fila de la tabla `cocktails`.
/// `base` y `glass` usan sus enums directamente para validación temprana.
#[derive(Deserialize)]
pub struct CocktailRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub base: CocktailBase,
    pub glass: GlassType,
    pub description: String,
    pub garnish: String,
    /// SQLite no tiene BOOLEAN — se guarda como INTEGER (0/1).
    pub is_adapted: i32,
    pub adaptation_note: Option<String>,
}

/// Fila de la tabla `ingredients`.
/// `category` usa el enum directamente — si la DB tiene un valor inválido, falla al deserializar.
/// `is_available` viene como INTEGER de SQLite (0/1) y se convierte a `bool` al ensamblar.
#[derive(Deserialize)]
pub struct IngredientRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub category: IngredientCategory,
    pub is_available: i32,
}

/// Fila de la tabla `cocktail_tastes`.
/// `taste` usa el enum directamente para validación temprana.
#[derive(Deserialize)]
pub struct TasteRow {
    pub cocktail_id: uuid::Uuid,
    pub taste: CocktailTaste,
}

/// Fila del JOIN entre `cocktail_ingredients` e `ingredients`.
/// Incluye `name` para evitar que el frontend tenga que cruzar datos.
#[derive(Deserialize)]
pub struct CocktailIngredientRow {
    pub cocktail_id: uuid::Uuid,
    pub ingredient_id: uuid::Uuid,
    /// Nombre legible del ingrediente, obtenido via JOIN con `ingredients`.
    pub name: String,
    pub amount: String,
    pub note: Option<String>,
    #[allow(dead_code)]
    pub sort_order: i32,
}

/// Fila de la tabla `cocktail_steps`.
#[derive(Deserialize)]
pub struct StepRow {
    pub cocktail_id: uuid::Uuid,
    #[allow(dead_code)]
    pub step_order: i32,
    pub description: String,
}

/// Fila de la tabla `cocktail_required_ingredients`.
#[derive(Deserialize)]
pub struct RequiredIngredientRow {
    pub cocktail_id: uuid::Uuid,
    pub ingredient_id: uuid::Uuid,
}

/// Resultado de `SELECT COUNT(*) as count FROM ...`
#[derive(Deserialize)]
pub struct CountRow {
    pub count: i64,
}
