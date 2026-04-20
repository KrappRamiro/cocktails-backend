//! Handlers para las rutas públicas de cocktails.

use crate::{db, json_response, models::*};
use worker::*;

/// GET /api/cocktails
///
/// Lista cocktails con filtros opcionales: `?base=gin`, `?taste=fresco`, `?available=true`.
/// Calcula disponibilidad cruzando `required_ingredients` con `ingredients.is_available`.
pub async fn list_cocktails(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db = ctx.env.d1("DB")?;

    // Obtener IDs de ingredientes disponibles para calcular disponibilidad
    let available_rows = db
        .prepare("SELECT id FROM ingredients WHERE is_available = 1")
        .all()
        .await?
        .results::<serde_json::Value>()?;
    let available_ids: Vec<uuid::Uuid> = available_rows
        .iter()
        .filter_map(|r| r.get("id").and_then(|v| v.as_str()))
        .filter_map(|s| s.parse().ok())
        .collect();

    // Parsear query params
    let url = req.url()?;
    let params: std::collections::HashMap<String, String> = url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let base_filter = params.get("base");
    let taste_filter = params.get("taste");
    let available_filter = params.get("available").map(|v| v == "true");

    // Query base: todos los cocktails o filtrados por base
    let cocktail_rows = if let Some(base) = base_filter {
        db.prepare("SELECT * FROM cocktails WHERE base = ?1")
            .bind(&[base.into()])?
            .all()
            .await?
            .results::<CocktailRow>()?
    } else {
        db.prepare("SELECT * FROM cocktails ORDER BY base, name")
            .all()
            .await?
            .results::<CocktailRow>()?
    };

    // Ensamblar cocktails completos
    let mut cocktails = db::assemble_cocktails(&db, cocktail_rows).await?;

    // Filtrar por taste si se pidió (post-proceso en Rust)
    if let Some(taste) = taste_filter {
        if let Ok(taste_enum) = serde_json::from_str::<CocktailTaste>(&format!("\"{}\"", taste)) {
            cocktails.retain(|c| c.taste.contains(&taste_enum));
        }
    }

    // Calcular disponibilidad
    let cocktails_with_availability: Vec<CocktailWithAvailability> = cocktails
        .into_iter()
        .map(|c| {
            let is_available = db::is_cocktail_available(&c, &available_ids);
            CocktailWithAvailability {
                cocktail: c,
                is_available,
            }
        })
        .collect();

    // Stats sobre el total (antes de filtrar por available)
    let total = cocktails_with_availability.len() as i64;
    let available = cocktails_with_availability
        .iter()
        .filter(|c| c.is_available)
        .count() as i64;

    // Filtrar por disponibilidad si se pidió
    let final_cocktails = if available_filter == Some(true) {
        cocktails_with_availability
            .into_iter()
            .filter(|c| c.is_available)
            .collect()
    } else {
        cocktails_with_availability
    };

    let response = CocktailsResponse {
        cocktails: final_cocktails,
        stats: Stats { total, available },
    };
    json_response(&serde_json::to_string(&response)?)
}

/// GET /api/cocktails/:id
///
/// Retorna un cocktail específico por UUID con su estado de disponibilidad.
pub async fn get_cocktail(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap();

    let row = db
        .prepare("SELECT * FROM cocktails WHERE id = ?1")
        .bind(&[id.into()])?
        .first::<CocktailRow>(None)
        .await?;

    let Some(row) = row else {
        return crate::cors_response(Response::error("Not Found", 404)?);
    };

    let cocktails = db::assemble_cocktails(&db, vec![row]).await?;
    let cocktail = cocktails.into_iter().next().unwrap();

    // Calcular disponibilidad
    let available_rows = db
        .prepare("SELECT id FROM ingredients WHERE is_available = 1")
        .all()
        .await?
        .results::<serde_json::Value>()?;
    let available_ids: Vec<uuid::Uuid> = available_rows
        .iter()
        .filter_map(|r| r.get("id").and_then(|v| v.as_str()))
        .filter_map(|s| s.parse().ok())
        .collect();

    let is_available = db::is_cocktail_available(&cocktail, &available_ids);
    let response = CocktailWithAvailability {
        cocktail,
        is_available,
    };
    json_response(&serde_json::to_string(&response)?)
}
