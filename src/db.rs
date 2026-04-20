//! Helpers para queries a D1 reutilizables entre handlers.
//!
//! Centraliza la lógica de ensamblado de `Cocktail` desde múltiples tablas
//! y el cálculo de disponibilidad en memoria.

use crate::models::*;
use std::collections::HashMap;
use worker::*;

/// Convierte un listado de filas planas de `cocktails` en structs `Cocktail` completos,
/// cargando todos sus datos relacionados desde las tablas secundarias.
///
/// ## Por qué existe esta función
///
/// La tabla `cocktails` solo contiene los campos escalares de cada receta (nombre, base, vaso, etc.).
/// Los gustos, ingredientes, pasos y required_ingredients viven en tablas separadas con FK.
/// Para construir un `Cocktail` usable hay que unir esas 4 tablas.
///
/// Esta función centraliza ese ensamblado para que ningún handler lo repita.
/// La usan handlers como `list_cocktails`, `get_cocktail` y `list_cocktails_admin`.
///
/// ## Estrategia anti N+1
///
/// En lugar de hacer una query por cada cocktail (N+1), hace exactamente 4 queries
/// adicionales usando `WHERE cocktail_id IN (uuid1, uuid2, ...)` sin importar cuántos
/// cocktails haya en el listado. Luego agrupa los resultados en HashMaps en memoria
/// y ensambla los structs finales en una sola pasada.
///
/// ## Orden de los resultados
///
/// El vector retornado mantiene el mismo orden que `cocktail_rows`.
///
/// # Argumentos
/// * `db` - Referencia a la D1 database del Worker
/// * `cocktail_rows` - Filas base de la tabla `cocktails`, ya filtradas/ordenadas por el handler
///
/// # Retorna
/// `Vec<Cocktail>` completamente ensamblados, listos para serializar a JSON.
pub async fn assemble_cocktails(
    db: &D1Database,
    cocktail_rows: Vec<CocktailRow>,
) -> Result<Vec<Cocktail>> {
    if cocktail_rows.is_empty() {
        return Ok(vec![]);
    }

    // Convierte [uuid1, uuid2, uuid3] en "'uuid1','uuid2','uuid3'" para el IN clause.
    // Seguro contra inyección: los valores provienen de uuid::Uuid,
    // que garantiza el formato xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.
    let cocktail_ids_sql = cocktail_rows
        .iter()
        .map(|row| format!("'{}'", row.id))
        .collect::<Vec<_>>()
        .join(",");

    // Lanzar las 4 queries en paralelo, cada una trae los datos de una tabla relacionada
    // para el conjunto completo de cocktails. Así el tiempo total es aprox la query más lenta,
    // no la suma de todas.
    //
    // Las queries se preparan primero en variables locales para que los &str que les pasamos
    // tengan lifetime suficiente dentro del futures::join!.
    let tastes_sql = format!(
        "SELECT cocktail_id, taste FROM cocktail_tastes
         WHERE cocktail_id IN ({})",
        cocktail_ids_sql
    );
    let ingredients_sql = format!(
        "SELECT ci.cocktail_id, ci.ingredient_id, i.name, ci.amount, ci.note, ci.sort_order
         FROM cocktail_ingredients ci
         JOIN ingredients i ON i.id = ci.ingredient_id
         WHERE ci.cocktail_id IN ({})
         ORDER BY ci.sort_order",
        cocktail_ids_sql
    );
    let steps_sql = format!(
        "SELECT cocktail_id, step_order, description
         FROM cocktail_steps
         WHERE cocktail_id IN ({})
         ORDER BY step_order",
        cocktail_ids_sql
    );
    let required_sql = format!(
        "SELECT cocktail_id, ingredient_id
         FROM cocktail_required_ingredients
         WHERE cocktail_id IN ({})",
        cocktail_ids_sql
    );
    let tastes_stmt = db.prepare(&tastes_sql);
    let ingredients_stmt = db.prepare(&ingredients_sql);
    let steps_stmt = db.prepare(&steps_sql);
    let required_stmt = db.prepare(&required_sql);
    let (tastes_query_result, ingredients_query_result, steps_query_result, required_query_result) = futures::join!(
        tastes_stmt.all(),
        ingredients_stmt.all(),
        steps_stmt.all(),
        required_stmt.all(),
    );

    // Deserializar cada resultado en su tipo de fila correspondiente
    let taste_rows = tastes_query_result?.results::<TasteRow>()?;
    let ingredient_rows = ingredients_query_result?.results::<CocktailIngredientRow>()?;
    let step_rows = steps_query_result?.results::<StepRow>()?;
    let required_rows = required_query_result?.results::<RequiredIngredientRow>()?;

    // Agrupar cada tabla secundaria en un HashMap<cocktail_id, Vec<dato>>
    // para acceso O(1) al ensamblar cada Cocktail.

    let mut tastes_by_cocktail: HashMap<uuid::Uuid, Vec<CocktailTaste>> = HashMap::new();
    for taste_row in taste_rows {
        tastes_by_cocktail
            .entry(taste_row.cocktail_id)
            .or_default()
            .push(taste_row.taste);
    }

    let mut ingredients_by_cocktail: HashMap<uuid::Uuid, Vec<CocktailIngredient>> = HashMap::new();
    for ingredient_row in ingredient_rows {
        ingredients_by_cocktail
            .entry(ingredient_row.cocktail_id)
            .or_default()
            .push(CocktailIngredient {
                ingredient_id: ingredient_row.ingredient_id,
                name: ingredient_row.name,
                amount: ingredient_row.amount,
                note: ingredient_row.note,
            });
    }

    let mut steps_by_cocktail: HashMap<uuid::Uuid, Vec<String>> = HashMap::new();
    for step_row in step_rows {
        steps_by_cocktail
            .entry(step_row.cocktail_id)
            .or_default()
            .push(step_row.description);
    }

    let mut required_ingredients_by_cocktail: HashMap<uuid::Uuid, Vec<uuid::Uuid>> = HashMap::new();
    for required_row in required_rows {
        required_ingredients_by_cocktail
            .entry(required_row.cocktail_id)
            .or_default()
            .push(required_row.ingredient_id);
    }

    // Ensamblar los Cocktail finales uniendo la fila base con los datos agrupados.
    // `base`, `glass` y `taste` ya vienen tipados como enums — serde los validó al deserializar.
    Ok(cocktail_rows
        .into_iter()
        .map(|row| {
            let id = row.id;
            Cocktail {
                id,
                name: row.name,
                base: row.base,
                taste: tastes_by_cocktail.remove(&id).unwrap_or_default(),
                glass: row.glass,
                description: row.description,
                ingredients: ingredients_by_cocktail.remove(&id).unwrap_or_default(),
                steps: steps_by_cocktail.remove(&id).unwrap_or_default(),
                garnish: row.garnish,
                is_adapted: row.is_adapted != 0,
                adaptation_note: row.adaptation_note,
                required_ingredients: required_ingredients_by_cocktail
                    .remove(&id)
                    .unwrap_or_default(),
            }
        })
        .collect())
}

/// Dado un cocktail ya ensamblado, calcula si está disponible
/// consultando en memoria si sus required_ingredients tienen is_available = 1.
///
/// Usar cuando ya tenés los ingredientes cargados en memoria para evitar
/// queries adicionales a D1.
///
/// # Argumentos
/// * `cocktail` - Receta con su lista de required_ingredients
/// * `available_ids` - UUIDs de ingredientes con is_available = 1
///   (obtener con `SELECT id FROM ingredients WHERE is_available = 1`)
pub fn is_cocktail_available(cocktail: &Cocktail, available_ids: &[uuid::Uuid]) -> bool {
    cocktail
        .required_ingredients
        .iter()
        .all(|id| available_ids.contains(id))
}

#[cfg(test)]
mod tests {
    use super::is_cocktail_available;
    use crate::models::cocktail::{Cocktail, CocktailBase, CocktailTaste, GlassType};
    use uuid::Uuid;

    /// Construye un `Cocktail` mínimo para tests de disponibilidad.
    fn make_cocktail(required_ingredients: Vec<Uuid>) -> Cocktail {
        Cocktail {
            id: Uuid::new_v4(),
            name: "Test Cocktail".to_string(),
            base: CocktailBase::Vodka,
            taste: vec![CocktailTaste::Fresco],
            glass: GlassType::VasoAlto,
            description: "Test".to_string(),
            ingredients: vec![],
            steps: vec![],
            garnish: "None".to_string(),
            is_adapted: false,
            adaptation_note: None,
            required_ingredients,
        }
    }

    // ─── is_cocktail_available ────────────────────────────────────────────────

    #[test]
    fn available_when_all_required_are_in_available_ids() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let cocktail = make_cocktail(vec![id1, id2]);
        // available_ids contiene id1, id2 y un extra (id3)
        let available = vec![id1, id2, id3];
        assert!(is_cocktail_available(&cocktail, &available));
    }

    #[test]
    fn available_when_all_required_are_exactly_available_ids() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let cocktail = make_cocktail(vec![id1, id2]);
        let available = vec![id1, id2];
        assert!(is_cocktail_available(&cocktail, &available));
    }

    #[test]
    fn not_available_when_one_required_is_missing() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let missing = Uuid::new_v4();
        let cocktail = make_cocktail(vec![id1, id2, missing]);
        // available_ids tiene id1 e id2 pero NO missing
        let available = vec![id1, id2];
        assert!(!is_cocktail_available(&cocktail, &available));
    }

    #[test]
    fn not_available_when_all_required_are_missing() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let cocktail = make_cocktail(vec![id1, id2]);
        // available_ids vacío
        let available: Vec<Uuid> = vec![];
        assert!(!is_cocktail_available(&cocktail, &available));
    }

    #[test]
    fn available_when_required_ingredients_is_empty() {
        // Un cocktail sin required_ingredients siempre está disponible
        let cocktail = make_cocktail(vec![]);
        let available = vec![Uuid::new_v4(), Uuid::new_v4()];
        assert!(is_cocktail_available(&cocktail, &available));
    }

    #[test]
    fn available_when_both_required_and_available_are_empty() {
        // Vacío + vacío: .all() sobre iterador vacío retorna true
        let cocktail = make_cocktail(vec![]);
        let available: Vec<Uuid> = vec![];
        assert!(is_cocktail_available(&cocktail, &available));
    }

    #[test]
    fn not_available_when_available_is_empty_but_required_is_not() {
        let id1 = Uuid::new_v4();
        let cocktail = make_cocktail(vec![id1]);
        let available: Vec<Uuid> = vec![];
        assert!(!is_cocktail_available(&cocktail, &available));
    }

    #[test]
    fn available_with_single_required_present() {
        let id = Uuid::new_v4();
        let cocktail = make_cocktail(vec![id]);
        let available = vec![id];
        assert!(is_cocktail_available(&cocktail, &available));
    }

    #[test]
    fn not_available_with_single_required_absent() {
        let id = Uuid::new_v4();
        let other = Uuid::new_v4();
        let cocktail = make_cocktail(vec![id]);
        // available tiene 'other', no 'id'
        let available = vec![other];
        assert!(!is_cocktail_available(&cocktail, &available));
    }

    #[test]
    fn not_available_when_only_last_required_is_missing() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let missing = Uuid::new_v4();
        let cocktail = make_cocktail(vec![id1, id2, id3, missing]);
        let available = vec![id1, id2, id3];
        assert!(!is_cocktail_available(&cocktail, &available));
    }
}
