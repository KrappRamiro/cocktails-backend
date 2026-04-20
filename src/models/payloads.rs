//! Structs para los bodies de los requests de las rutas admin.

use crate::models::cocktail::{CocktailBase, CocktailTaste, GlassType};
use crate::models::ingredient::IngredientCategory;
use serde::{Deserialize, Serialize};

/// Body para `POST /api/admin/ingredients` y `PUT /api/admin/ingredients/:id`.
/// Crea o reemplaza los datos de un ingrediente.
///
/// `is_available` no se incluye — se gestiona exclusivamente
/// con `PATCH /api/admin/ingredients/:id/available`.
#[derive(Serialize, Deserialize)]
pub struct IngredientPayload {
    pub name: String,
    pub category: IngredientCategory,
}

/// Body para `PATCH /api/admin/ingredients/:id/available`.
/// Activa o desactiva la disponibilidad de un ingrediente individual.
#[derive(Serialize, Deserialize)]
pub struct IngredientAvailabilityPayload {
    /// `true` para marcar como disponible, `false` para no disponible.
    pub available: bool,
}

/// Body para `POST /api/admin/cocktails` y `PUT /api/admin/cocktails/:id`.
/// Contiene todos los datos de una receta para crear o reemplazar.
///
/// Usa los mismos enums que `Cocktail` — si el frontend manda un valor inválido
/// (ej: `"vodkas"` en lugar de `"vodka"`), serde falla al deserializar con un
/// error 400 antes de que el handler llegue a ejecutar ninguna lógica.
#[derive(Serialize, Deserialize, Clone)]
pub struct CocktailPayload {
    pub name: String,
    pub base: CocktailBase,
    pub taste: Vec<CocktailTaste>,
    pub glass: GlassType,
    pub description: String,
    pub ingredients: Vec<CocktailIngredientPayload>,
    pub steps: Vec<String>,
    pub garnish: String,
    pub is_adapted: bool,
    pub adaptation_note: Option<String>,
    /// UUIDs que deben existir en la tabla `ingredients`.
    pub required_ingredients: Vec<uuid::Uuid>,
}

/// Ingrediente dentro del payload de creación/edición de receta.
#[derive(Serialize, Deserialize, Clone)]
pub struct CocktailIngredientPayload {
    /// UUID del ingrediente. Debe existir en la tabla `ingredients`.
    pub ingredient_id: uuid::Uuid,
    pub amount: String,
    pub note: Option<String>,
}
