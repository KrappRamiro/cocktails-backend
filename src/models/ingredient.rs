//! Modelos relacionados con los ingredientes.

use serde::{Deserialize, Serialize};

/// Categoría de un ingrediente. Conjunto cerrado — si la DB devuelve un valor
/// que no existe en este enum, la deserialización falla inmediatamente.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IngredientCategory {
    BasesAlcoholicas,
    Licores,
    VermutsAperitivos,
    Amargos,
    MixersGaseosas,
    Jugos,
    FrescosYBotanicos,
    BasicosAlacena,
    Decoracion,
}

/// Representa un ingrediente de la lista maestra, con su estado de disponibilidad actual.
///
/// `is_available` refleja la columna `ingredients.is_available` de D1.
/// Se actualiza vía `PATCH /api/admin/ingredients/:id/available`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ingredient {
    /// UUID v4 generado al insertar. Inmutable después de la creación.
    /// Serde serializa/deserializa automáticamente desde el TEXT de D1.
    pub id: uuid::Uuid,
    /// Nombre legible para mostrar al usuario. Ej: `"Gin (seco/dry)"`.
    pub name: String,
    /// Categoría del ingrediente. Validada en tiempo de compilación.
    pub category: IngredientCategory,
    /// `true` si `is_available = 1` en la tabla `ingredients` (activado por el admin).
    pub is_available: bool,
}

/// Response del endpoint `GET /api/ingredients`.
#[derive(Serialize, Deserialize)]
pub struct IngredientsResponse {
    /// Lista completa de ingredientes con su estado de disponibilidad.
    pub ingredients: Vec<Ingredient>,
}
