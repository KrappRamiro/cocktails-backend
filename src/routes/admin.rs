//! Handlers para las rutas admin (requieren Basic Auth).
//!
//! Todas las rutas de este módulo validan credenciales contra los secrets
//! `ADMIN_USER` y `ADMIN_PASSWORD` del Worker.

use base64::Engine as _;
use worker::*;
use crate::{cors_response, db, json_response_no_cache, models::*};

/// Reads an env binding by name.
/// Tries every access method worker-rs provides: .secret(), .var(), and raw JsValue.
fn get_env_value(ctx: &RouteContext<()>, name: &str) -> Result<String> {
    // Method 1: .secret()
    if let Ok(val) = ctx.env.secret(name) {
        return Ok(val.to_string());
    }
    // Method 2: .var()
    if let Ok(val) = ctx.env.var(name) {
        return Ok(val.to_string());
    }
    Err(worker::Error::RustError(format!("Binding `{}` is undefined. Set it in .dev.vars or wrangler secret put.", name)))
}

/// Valida Basic Auth contra los secrets del Worker.
/// Retorna `true` si las credenciales coinciden.
fn check_auth(req: &Request, ctx: &RouteContext<()>) -> Result<bool> {
    let auth_header = req.headers().get("Authorization")?.unwrap_or_default();
    if !auth_header.starts_with("Basic ") {
        return Ok(false);
    }
    let encoded = &auth_header["Basic ".len()..];
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .unwrap_or_default();
    let decoded = String::from_utf8_lossy(&decoded_bytes);
    let parts: Vec<&str> = decoded.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Ok(false);
    }

    let expected_user = get_env_value(ctx, "ADMIN_USER")?;
    let expected_pass = get_env_value(ctx, "ADMIN_PASSWORD")?;
    Ok(parts[0] == expected_user && parts[1] == expected_pass)
}

/// Helper: retorna 401 con CORS headers y Retry-After si la auth falla.
/// `Retry-After: 2` le indica al cliente que espere 2 segundos antes de reintentar,
/// y sirve como señal para configurar Cloudflare WAF Rate Limiting rules
/// en el dashboard: limitar requests a `/api/admin/*` que retornen 401 a N por minuto.
macro_rules! require_auth {
    ($req:expr, $ctx:expr) => {
        if !check_auth(&$req, &$ctx)? {
            let headers = Headers::new();
            headers.set("Access-Control-Allow-Origin", "*")?;
            headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS")?;
            headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization")?;
            headers.set("Retry-After", "2")?;
            return Ok(Response::error("Unauthorized", 401)?.with_headers(headers));
        }
    };
}

// ─── Ingredientes ─────────────────────────────────────────────────────────────

/// POST /api/admin/ingredients — Crear ingrediente
pub async fn create_ingredient(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_auth!(req, ctx);
    let db = ctx.env.d1("DB")?;
    let payload: IngredientPayload = req.json().await?;
    let id = uuid::Uuid::new_v4();
    let category_str = serde_json::to_value(&payload.category)?
        .as_str()
        .unwrap_or_default()
        .to_string();

    db.prepare("INSERT INTO ingredients (id, name, category) VALUES (?1, ?2, ?3)")
        .bind(&[id.to_string().into(), payload.name.clone().into(), category_str.into()])?
        .run()
        .await?;

    let ingredient = Ingredient {
        id,
        name: payload.name,
        category: payload.category,
        is_available: false,
    };
    let body = serde_json::to_string(&ingredient)?;
    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization")?;
    let resp = Response::ok(body)?.with_headers(headers).with_status(201);
    Ok(resp)
}

/// PUT /api/admin/ingredients/:id — Editar ingrediente
pub async fn update_ingredient(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_auth!(req, ctx);
    let db = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap().to_string();
    let payload: IngredientPayload = req.json().await?;
    let category_str = serde_json::to_value(&payload.category)?
        .as_str()
        .unwrap_or_default()
        .to_string();

    // Verificar que existe
    let existing = db
        .prepare("SELECT COUNT(*) as count FROM ingredients WHERE id = ?1")
        .bind(&[id.clone().into()])?
        .first::<CountRow>(None)
        .await?;
    if existing.map(|r| r.count).unwrap_or(0) == 0 {
        return cors_response(Response::error("Not Found", 404)?);
    }

    db.prepare("UPDATE ingredients SET name = ?1, category = ?2 WHERE id = ?3")
        .bind(&[payload.name.clone().into(), category_str.into(), id.clone().into()])?
        .run()
        .await?;

    // Retornar ingrediente actualizado
    let row = db
        .prepare("SELECT id, name, category, is_available FROM ingredients WHERE id = ?1")
        .bind(&[id.into()])?
        .first::<IngredientRow>(None)
        .await?
        .unwrap();

    let ingredient = Ingredient {
        id: row.id,
        name: row.name,
        category: row.category,
        is_available: row.is_available != 0,
    };
    json_response_no_cache(&serde_json::to_string(&ingredient)?)
}

/// DELETE /api/admin/ingredients/:id — Eliminar ingrediente
pub async fn delete_ingredient(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_auth!(req, ctx);
    let db = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap().to_string();

    // Verificar que existe
    let existing = db
        .prepare("SELECT COUNT(*) as count FROM ingredients WHERE id = ?1")
        .bind(&[id.clone().into()])?
        .first::<CountRow>(None)
        .await?;
    if existing.map(|r| r.count).unwrap_or(0) == 0 {
        return cors_response(Response::error("Not Found", 404)?);
    }

    // Verificar que no está en uso
    let in_use = db
        .prepare("SELECT COUNT(*) as count FROM cocktail_required_ingredients WHERE ingredient_id = ?1")
        .bind(&[id.clone().into()])?
        .first::<CountRow>(None)
        .await?;
    let in_use_count = in_use.map(|r| r.count).unwrap_or(0);
    if in_use_count > 0 {
        let count = in_use_count;
        return cors_response(Response::error(
            format!("Este ingrediente está en uso por {} receta(s). Eliminalo de las recetas primero.", count),
            409,
        )?);
    }

    db.prepare("DELETE FROM ingredients WHERE id = ?1")
        .bind(&[id.into()])?
        .run()
        .await?;

    cors_response(Response::empty()?.with_status(204))
}

/// PATCH /api/admin/ingredients/:id/available — Toggle disponibilidad
pub async fn toggle_ingredient(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_auth!(req, ctx);
    let db = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap().to_string();
    let payload: IngredientAvailabilityPayload = req.json().await?;

    // Verificar que existe
    let existing = db
        .prepare("SELECT COUNT(*) as count FROM ingredients WHERE id = ?1")
        .bind(&[id.clone().into()])?
        .first::<CountRow>(None)
        .await?;
    if existing.map(|r| r.count).unwrap_or(0) == 0 {
        return cors_response(Response::error("Not Found", 404)?);
    }

    let value: i32 = if payload.available { 1 } else { 0 };
    db.prepare("UPDATE ingredients SET is_available = ?1 WHERE id = ?2")
        .bind(&[value.into(), id.clone().into()])?
        .run()
        .await?;

    let row = db
        .prepare("SELECT id, name, category, is_available FROM ingredients WHERE id = ?1")
        .bind(&[id.into()])?
        .first::<IngredientRow>(None)
        .await?
        .unwrap();

    let ingredient = Ingredient {
        id: row.id,
        name: row.name,
        category: row.category,
        is_available: row.is_available != 0,
    };
    json_response_no_cache(&serde_json::to_string(&ingredient)?)
}

// ─── Cocktails ────────────────────────────────────────────────────────────────

/// GET /api/admin/cocktails — Listar todos (incluyendo no disponibles)
pub async fn list_cocktails_admin(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_auth!(req, ctx);
    let db = ctx.env.d1("DB")?;

    let rows = db
        .prepare("SELECT * FROM cocktails ORDER BY base, name")
        .all()
        .await?
        .results::<CocktailRow>()?;

    let cocktails = db::assemble_cocktails(&db, rows).await?;

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

    let total = cocktails_with_availability.len() as i64;
    let available = cocktails_with_availability
        .iter()
        .filter(|c| c.is_available)
        .count() as i64;

    let response = CocktailsResponse {
        cocktails: cocktails_with_availability,
        stats: Stats { total, available },
    };
    json_response_no_cache(&serde_json::to_string(&response)?)
}

/// POST /api/admin/cocktails — Crear receta
pub async fn create_cocktail(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_auth!(req, ctx);
    let db = ctx.env.d1("DB")?;
    let payload: CocktailPayload = req.json().await?;
    let id = uuid::Uuid::new_v4();

    let base_str = serde_json::to_value(&payload.base)?.as_str().unwrap_or_default().to_string();
    let glass_str = serde_json::to_value(&payload.glass)?.as_str().unwrap_or_default().to_string();
    let is_adapted: i32 = if payload.is_adapted { 1 } else { 0 };

    // Build batch de statements
    let mut statements = vec![
        db.prepare("INSERT INTO cocktails (id, name, base, glass, description, garnish, is_adapted, adaptation_note) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")
            .bind(&[
                id.to_string().into(),
                payload.name.clone().into(),
                base_str.into(),
                glass_str.into(),
                payload.description.clone().into(),
                payload.garnish.clone().into(),
                is_adapted.into(),
                payload.adaptation_note.clone().unwrap_or_default().into(),
            ])?,
    ];

    // Tastes
    for taste in &payload.taste {
        let taste_str = serde_json::to_value(taste)?.as_str().unwrap_or_default().to_string();
        statements.push(
            db.prepare("INSERT INTO cocktail_tastes (cocktail_id, taste) VALUES (?1, ?2)")
                .bind(&[id.to_string().into(), taste_str.into()])?,
        );
    }

    // Ingredients
    for (i, ing) in payload.ingredients.iter().enumerate() {
        statements.push(
            db.prepare("INSERT INTO cocktail_ingredients (cocktail_id, ingredient_id, amount, note, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)")
                .bind(&[
                    id.to_string().into(),
                    ing.ingredient_id.to_string().into(),
                    ing.amount.clone().into(),
                    ing.note.clone().unwrap_or_default().into(),
                    (i as i32).into(),
                ])?,
        );
    }

    // Steps
    for (i, step) in payload.steps.iter().enumerate() {
        statements.push(
            db.prepare("INSERT INTO cocktail_steps (cocktail_id, step_order, description) VALUES (?1, ?2, ?3)")
                .bind(&[id.to_string().into(), (i as i32).into(), step.clone().into()])?,
        );
    }

    // Required ingredients
    for req_id in &payload.required_ingredients {
        statements.push(
            db.prepare("INSERT INTO cocktail_required_ingredients (cocktail_id, ingredient_id) VALUES (?1, ?2)")
                .bind(&[id.to_string().into(), req_id.to_string().into()])?,
        );
    }

    db.batch(statements).await?;

    let body = serde_json::to_string(&serde_json::json!({ "id": id, "name": payload.name }))?;
    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization")?;
    Ok(Response::ok(body)?.with_headers(headers).with_status(201))
}

/// PUT /api/admin/cocktails/:id — Editar receta (reemplazo completo)
pub async fn update_cocktail(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_auth!(req, ctx);
    let db = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap().to_string();
    let payload: CocktailPayload = req.json().await?;

    // Verificar que existe
    let existing = db
        .prepare("SELECT COUNT(*) as count FROM cocktails WHERE id = ?1")
        .bind(&[id.clone().into()])?
        .first::<CountRow>(None)
        .await?;
    if existing.map(|r| r.count).unwrap_or(0) == 0 {
        return cors_response(Response::error("Not Found", 404)?);
    }

    let base_str = serde_json::to_value(&payload.base)?.as_str().unwrap_or_default().to_string();
    let glass_str = serde_json::to_value(&payload.glass)?.as_str().unwrap_or_default().to_string();
    let is_adapted: i32 = if payload.is_adapted { 1 } else { 0 };

    let mut statements = vec![
        // Update base row
        db.prepare("UPDATE cocktails SET name=?1, base=?2, glass=?3, description=?4, garnish=?5, is_adapted=?6, adaptation_note=?7 WHERE id=?8")
            .bind(&[
                payload.name.clone().into(),
                base_str.into(),
                glass_str.into(),
                payload.description.clone().into(),
                payload.garnish.clone().into(),
                is_adapted.into(),
                payload.adaptation_note.clone().unwrap_or_default().into(),
                id.clone().into(),
            ])?,
        // Delete related data (CASCADE would handle this on DELETE, but for UPDATE we do it manually)
        db.prepare("DELETE FROM cocktail_tastes WHERE cocktail_id = ?1").bind(&[id.clone().into()])?,
        db.prepare("DELETE FROM cocktail_ingredients WHERE cocktail_id = ?1").bind(&[id.clone().into()])?,
        db.prepare("DELETE FROM cocktail_steps WHERE cocktail_id = ?1").bind(&[id.clone().into()])?,
        db.prepare("DELETE FROM cocktail_required_ingredients WHERE cocktail_id = ?1").bind(&[id.clone().into()])?,
    ];

    // Re-insert related data
    for taste in &payload.taste {
        let taste_str = serde_json::to_value(taste)?.as_str().unwrap_or_default().to_string();
        statements.push(
            db.prepare("INSERT INTO cocktail_tastes (cocktail_id, taste) VALUES (?1, ?2)")
                .bind(&[id.clone().into(), taste_str.into()])?,
        );
    }

    for (i, ing) in payload.ingredients.iter().enumerate() {
        statements.push(
            db.prepare("INSERT INTO cocktail_ingredients (cocktail_id, ingredient_id, amount, note, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)")
                .bind(&[
                    id.clone().into(),
                    ing.ingredient_id.to_string().into(),
                    ing.amount.clone().into(),
                    ing.note.clone().unwrap_or_default().into(),
                    (i as i32).into(),
                ])?,
        );
    }

    for (i, step) in payload.steps.iter().enumerate() {
        statements.push(
            db.prepare("INSERT INTO cocktail_steps (cocktail_id, step_order, description) VALUES (?1, ?2, ?3)")
                .bind(&[id.clone().into(), (i as i32).into(), step.clone().into()])?,
        );
    }

    for req_id in &payload.required_ingredients {
        statements.push(
            db.prepare("INSERT INTO cocktail_required_ingredients (cocktail_id, ingredient_id) VALUES (?1, ?2)")
                .bind(&[id.clone().into(), req_id.to_string().into()])?,
        );
    }

    db.batch(statements).await?;
    json_response_no_cache(&serde_json::to_string(&serde_json::json!({ "id": id, "name": payload.name }))?)
}

/// DELETE /api/admin/cocktails/:id — Eliminar receta
pub async fn delete_cocktail(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_auth!(req, ctx);
    let db = ctx.env.d1("DB")?;
    let id = ctx.param("id").unwrap().to_string();

    let existing = db
        .prepare("SELECT COUNT(*) as count FROM cocktails WHERE id = ?1")
        .bind(&[id.clone().into()])?
        .first::<CountRow>(None)
        .await?;
    if existing.map(|r| r.count).unwrap_or(0) == 0 {
        return cors_response(Response::error("Not Found", 404)?);
    }

    // ON DELETE CASCADE limpia todas las tablas relacionadas
    db.prepare("DELETE FROM cocktails WHERE id = ?1")
        .bind(&[id.into()])?
        .run()
        .await?;

    cors_response(Response::empty()?.with_status(204))
}
