//! Handlers para `/api/config` (público) y `/api/admin/config` (admin).
//!
//! El `event_config` guarda el nombre del evento actual, editable por admin
//! y leído público para mostrarlo como título del menú.

use crate::{cors_response, json_response, json_response_no_cache, routes::admin::check_auth};
use serde::{Deserialize, Serialize};
use worker::*;

const EVENT_NAME_MAX_LEN: usize = 80;

#[derive(Serialize, Deserialize)]
struct ConfigRow {
    event_name: String,
}

#[derive(Deserialize)]
struct ConfigPayload {
    event_name: String,
}

#[derive(Serialize)]
struct ConfigResponse {
    event_name: String,
}

/// GET /api/config — retorna la configuración pública del evento.
pub async fn get_config(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db = ctx.env.d1("DB")?;
    let row = db
        .prepare("SELECT event_name FROM event_config WHERE id = 1")
        .first::<ConfigRow>(None)
        .await?;

    let event_name = row.map(|r| r.event_name).unwrap_or_else(|| "Evento".into());
    let response = ConfigResponse { event_name };
    json_response(&serde_json::to_string(&response)?)
}

/// PUT /api/admin/config — actualiza el nombre del evento.
/// Requires Basic Auth. Validates length (1..=80) and trims whitespace.
pub async fn update_config(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if !check_auth(&req, &ctx)? {
        let headers = Headers::new();
        headers.set("Access-Control-Allow-Origin", "*")?;
        headers.set(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, PATCH, OPTIONS",
        )?;
        headers.set(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )?;
        headers.set("Retry-After", "2")?;
        return Ok(Response::error("Unauthorized", 401)?.with_headers(headers));
    }

    let payload: ConfigPayload = req.json().await?;
    let trimmed = payload.event_name.trim().to_string();

    if trimmed.is_empty() {
        return cors_response(Response::error(
            "El nombre del evento no puede estar vacío",
            400,
        )?);
    }

    if trimmed.chars().count() > EVENT_NAME_MAX_LEN {
        return cors_response(Response::error(
            format!("El nombre del evento no puede tener más de {EVENT_NAME_MAX_LEN} caracteres"),
            400,
        )?);
    }

    let db = ctx.env.d1("DB")?;
    db.prepare("UPDATE event_config SET event_name = ?1, updated_at = unixepoch() WHERE id = 1")
        .bind(&[trimmed.clone().into()])?
        .run()
        .await?;

    let response = ConfigResponse {
        event_name: trimmed,
    };
    json_response_no_cache(&serde_json::to_string(&response)?)
}
