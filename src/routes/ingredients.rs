//! Handler para la ruta pública de ingredientes.

use crate::{json_response, models::*};
use worker::*;

/// GET /api/ingredients?page=1&limit=20
///
/// Retorna ingredientes paginados. Defaults: page=1, limit=20.
/// Response incluye `total` para que el frontend sepa cuántas páginas hay.
pub async fn list_ingredients(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db = ctx.env.d1("DB")?;

    let url = req.url()?;
    let params: std::collections::HashMap<String, String> = url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let page: i64 = params
        .get("page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
        .max(1);
    let limit: i64 = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
        .clamp(1, 50);
    let offset = (page - 1) * limit;

    // Total count
    let total = db
        .prepare("SELECT COUNT(*) as count FROM ingredients")
        .first::<CountRow>(None)
        .await?
        .map(|r| r.count)
        .unwrap_or(0);

    // Paginated query — LIMIT/OFFSET inlined because D1 bind params
    // don't reliably work with LIMIT/OFFSET clauses.
    let query = format!(
        "SELECT id, name, category, is_available FROM ingredients ORDER BY category, name LIMIT {} OFFSET {}",
        limit, offset
    );
    let rows = db.prepare(&query).all().await?.results::<IngredientRow>()?;

    let ingredients: Vec<Ingredient> = rows
        .into_iter()
        .map(|row| Ingredient {
            id: row.id,
            name: row.name,
            category: row.category,
            is_available: row.is_available != 0,
        })
        .collect();

    let response = serde_json::json!({
        "ingredients": ingredients,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total,
            "total_pages": (total as f64 / limit as f64).ceil() as i64,
        }
    });
    json_response(&serde_json::to_string(&response)?)
}
