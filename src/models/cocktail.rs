//! Modelos relacionados con los cocktails (recetas).

use serde::{Deserialize, Serialize};

/// Base alcohólica del cocktail.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CocktailBase {
    Gin,
    Vodka,
    Tequila,
    Ron,
    Whisky,
    Brandy,
    Pisco,
    Caipiroska,
    Caipirinha,
    GinTonics,
    Mocktail,
    Espumante,
    Licores,
}

/// Perfil de sabor del cocktail. Un cocktail puede tener varios.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CocktailTaste {
    Fresco,
    Frutal,
    Tropical,
    Clasico,
    Amargo,
    SinAlcohol,
}

/// Tipo de vaso para servir el cocktail.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GlassType {
    CopaMartini,
    CopaBalon,
    VasoAlto,
    VasoBajo,
    CopaVino,
}

/// Un ingrediente dentro de una receta, con cantidad y nota opcional.
/// Se usa para mostrar la receta completa al invitado.
///
/// Incluye el `name` del ingrediente para que el frontend pueda renderizar
/// la receta directamente sin necesidad de cruzar con `GET /api/ingredients`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CocktailIngredient {
    /// UUID del ingrediente referenciado.
    pub ingredient_id: uuid::Uuid,
    /// Nombre legible del ingrediente. Ej: `"Gin (seco/dry)"`.
    /// Obtenido via JOIN con `ingredients` al ensamblar el cocktail.
    pub name: String,
    /// Cantidad expresada como string. Ej: `"50ml"`, `"2 cdas"`, `"completar"`.
    pub amount: String,
    /// Nota opcional de preparación. Ej: `"bien frío"`, `"o Ramazotti"`.
    pub note: Option<String>,
}

/// Receta completa con toda la información necesaria para mostrarla.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cocktail {
    /// UUID v4 generado al insertar. Inmutable después de la creación.
    pub id: uuid::Uuid,
    pub name: String,
    /// Base alcohólica del cocktail. Validada en tiempo de compilación.
    pub base: CocktailBase,
    /// Perfiles de sabor del cocktail. Puede tener varios simultáneamente.
    pub taste: Vec<CocktailTaste>,
    /// Tipo de vaso para servir. Validado en tiempo de compilación.
    pub glass: GlassType,
    pub description: String,
    /// Lista de ingredientes con cantidades para mostrar la receta.
    pub ingredients: Vec<CocktailIngredient>,
    /// Pasos de preparación en orden.
    pub steps: Vec<String>,
    pub garnish: String,
    /// `true` si la receta fue adaptada respecto al original.
    pub is_adapted: bool,
    /// Descripción de la adaptación. Solo presente si `is_adapted = true`.
    pub adaptation_note: Option<String>,
    /// UUIDs de ingredientes indispensables para calcular disponibilidad.
    /// Un trago está disponible SOLO si TODOS estos UUIDs tienen `is_available = 1`.
    /// Es un subconjunto de `ingredients` — excluye decoraciones opcionales.
    pub required_ingredients: Vec<uuid::Uuid>,
}

/// Cocktail con el flag de disponibilidad calculado en runtime.
/// Es el tipo que retorna `GET /api/cocktails`.
#[derive(Serialize, Deserialize)]
pub struct CocktailWithAvailability {
    /// Todos los campos de Cocktail aplanados en el JSON (serde flatten).
    #[serde(flatten)]
    pub cocktail: Cocktail,
    /// `true` si todos los `required_ingredients` tienen `is_available = 1`
    /// en la tabla `ingredients`.
    pub is_available: bool,
}

/// Response del endpoint `GET /api/cocktails`.
#[derive(Serialize, Deserialize)]
pub struct CocktailsResponse {
    pub cocktails: Vec<CocktailWithAvailability>,
    /// Stats calculadas sobre el total de recetas (sin considerar filtros activos).
    pub stats: Stats,
}

/// Conteo de disponibilidad del menú.
#[derive(Serialize, Deserialize)]
pub struct Stats {
    /// Total de recetas en la DB.
    pub total: i64,
    /// Recetas con `is_available = true`.
    pub available: i64,
}
