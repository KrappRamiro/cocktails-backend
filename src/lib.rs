//! Entry point del Cloudflare Worker.
//!
//! Define el router principal con todas las rutas públicas y de admin,
//! y expone helpers compartidos para CORS y serialización JSON.

use worker::*;

mod db;
mod models;
mod routes;

/// Handler principal del Worker. Recibe todos los requests HTTP.
///
/// El router despacha cada request a su handler según método + path.
/// Las rutas `/api/admin/*` requieren Basic Auth (validada en cada handler).
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();
    router
        // CORS preflight — responder a OPTIONS en cualquier ruta
        .options_async("/*path", |_req, _ctx| async move {
            cors_response(Response::empty()?)
        })
        // API info
        .get_async("/", api_info)
        .get_async("/api", api_info)
        .get_async("/api/", api_info)
        // Health check
        .get_async("/api/health", |_req, _ctx| async move {
            json_response(r#"{"status":"ok"}"#)
        })
        // Rutas públicas (sin autenticación)
        .get_async("/api/cocktails", routes::cocktails::list_cocktails)
        .get_async("/api/cocktails/:id", routes::cocktails::get_cocktail)
        .get_async("/api/ingredients", routes::ingredients::list_ingredients)
        // Rutas admin — ingredientes
        .post_async("/api/admin/ingredients", routes::admin::create_ingredient)
        .put_async(
            "/api/admin/ingredients/:id",
            routes::admin::update_ingredient,
        )
        .delete_async(
            "/api/admin/ingredients/:id",
            routes::admin::delete_ingredient,
        )
        .patch_async(
            "/api/admin/ingredients/:id/available",
            routes::admin::toggle_ingredient,
        )
        // Rutas admin — cocktails
        .get_async("/api/admin/cocktails", routes::admin::list_cocktails_admin)
        .post_async("/api/admin/cocktails", routes::admin::create_cocktail)
        .put_async("/api/admin/cocktails/:id", routes::admin::update_cocktail)
        .delete_async("/api/admin/cocktails/:id", routes::admin::delete_cocktail)
        .run(req, env)
        .await
}

/// Retorna información básica de la API: nombre y lista de endpoints públicos.
async fn api_info(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    json_response(r#"{"name":"Coctelería API","endpoints":["/api/health","/api/cocktails","/api/cocktails/:id","/api/ingredients"]}"#)
}

/// Agrega los headers CORS estándar a un Response existente.
///
/// Se usa para las respuestas normales. Para respuestas de error que también
/// necesitan CORS, llamar `cors_response(Response::error("msg", code)?)`.
pub fn cors_response(res: Response) -> Result<Response> {
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
    Ok(res.with_headers(headers))
}

/// Serializa un body JSON y retorna un Response con Content-Type, CORS headers,
/// y cache de 10 segundos para las rutas públicas.
///
/// El `max-age=10` permite que el edge de Cloudflare sirva respuestas cacheadas
/// durante 10 segundos, reduciendo la carga sobre D1 cuando muchos invitados
/// consultan el menú simultáneamente.
pub fn json_response(body: &str) -> Result<Response> {
    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Cache-Control", "public, max-age=10")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, PATCH, OPTIONS",
    )?;
    headers.set(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization",
    )?;
    Response::ok(body).map(|r| r.with_headers(headers))
}

/// Igual que `json_response` pero sin cache (`no-store`).
/// Para respuestas de rutas admin que deben reflejar el estado más reciente.
pub fn json_response_no_cache(body: &str) -> Result<Response> {
    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Cache-Control", "no-store")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, PATCH, OPTIONS",
    )?;
    headers.set(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization",
    )?;
    Response::ok(body).map(|r| r.with_headers(headers))
}
