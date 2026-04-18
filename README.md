# Worker — Cloudflare Worker (Rust)

The backend is a Cloudflare Worker written in Rust, compiled to WebAssembly. It handles all HTTP routing, authentication, database access, and business logic.

---

## Prerequisites

| Tool | Version | Install |
|---|---|---|
| Rust | stable (1.75+) | [rustup.rs](https://rustup.rs/) |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| Node.js | 20+ | [nodejs.org](https://nodejs.org/) (needed for wrangler) |
| wrangler | 4.x | `npm install -g wrangler` |

## Setup (local development)

```bash
# 1. Install wasm target (one time)
rustup target add wasm32-unknown-unknown

# 2. Install wrangler CLI (one time)
npm install -g wrangler

# 3. Verify the project compiles
cargo check

# 4. Run unit tests
cargo test

# 5. Run the D1 migrations on the local database
wrangler d1 execute cocktail-db --local --file=migrations/0001_schema.sql
wrangler d1 execute cocktail-db --local --file=migrations/0002_seed_ingredients.sql
wrangler d1 execute cocktail-db --local --file=migrations/0003_seed_cocktails_1.sql
wrangler d1 execute cocktail-db --local --file=migrations/0004_seed_cocktails_2.sql

# 6. Start the dev server (builds Wasm + starts local Workers runtime)
wrangler dev --local
# → http://localhost:8787

# 7. Test it
curl http://localhost:8787/api/health
curl http://localhost:8787/api/cocktails
curl http://localhost:8787/api/ingredients
```

> **Note:** The first `wrangler dev` build takes ~15 seconds because it compiles all crates to Wasm. Subsequent builds are incremental (~1 second).

> **Admin secrets:** For local dev, wrangler uses `.dev.vars` for secrets. Create `worker/.dev.vars`:
> ```
> ADMIN_USER=organizador
> ADMIN_PASSWORD=test123
> ```

---

## Project Structure

```
worker/src/
├── lib.rs          # Entry point: Worker event handler + router + shared helpers
├── db.rs           # Shared DB logic: cocktail assembly and availability calculation
├── models/
│   ├── mod.rs      # Re-exports all model types
│   ├── cocktail.rs # Domain types: Cocktail, CocktailIngredient, enums, response wrappers
│   ├── ingredient.rs # Ingredient, IngredientCategory enum
│   ├── payloads.rs # Request body types for admin write operations
│   └── rows.rs     # Raw D1 row types (Deserialize only — never serialized to JSON)
└── routes/
    ├── mod.rs      # Re-exports route modules
    ├── cocktails.rs # GET /api/cocktails, GET /api/cocktails/:id
    ├── ingredients.rs # GET /api/ingredients
    └── admin.rs    # All /api/admin/* routes (auth required)
```

### Why the separation between `models/` and `routes/`

The models directory contains the data layer: what things *are*. The routes directory contains the HTTP layer: how requests map to operations. The `db.rs` file sits between them — it contains logic that multiple routes reuse but that is not part of the model definitions.

This prevents the common antipattern of each handler containing its own query logic, which quickly leads to N+1 queries and duplicated assembly code.

---

## D1 Database Schema

The schema is defined in `migrations/0001_schema.sql` and applied with Wrangler.

### Tables and Relationships

```
ingredients
  id           TEXT PRIMARY KEY    (UUID v4)
  name         TEXT UNIQUE
  category     TEXT                (snake_case enum value)
  is_available INTEGER DEFAULT 0   (0 = unavailable, 1 = available)

cocktails
  id               TEXT PRIMARY KEY  (UUID v4)
  name             TEXT UNIQUE
  base             TEXT              (snake_case enum value)
  glass            TEXT              (snake_case enum value)
  description      TEXT
  garnish          TEXT
  is_adapted       INTEGER DEFAULT 0
  adaptation_note  TEXT

cocktail_tastes                        ← one cocktail → many tastes
  cocktail_id  TEXT FK → cocktails(id) CASCADE DELETE
  taste        TEXT                   (snake_case enum value)
  PRIMARY KEY (cocktail_id, taste)

cocktail_ingredients                   ← recipe ingredient list (what to show guests)
  id             INTEGER AUTOINCREMENT
  cocktail_id    TEXT FK → cocktails(id) CASCADE DELETE
  ingredient_id  TEXT FK → ingredients(id)
  amount         TEXT
  note           TEXT
  sort_order     INTEGER DEFAULT 0

cocktail_steps                         ← preparation instructions
  id           INTEGER AUTOINCREMENT
  cocktail_id  TEXT FK → cocktails(id) CASCADE DELETE
  step_order   INTEGER
  description  TEXT

cocktail_required_ingredients          ← availability gate (subset of ingredients)
  cocktail_id    TEXT FK → cocktails(id) CASCADE DELETE
  ingredient_id  TEXT FK → ingredients(id)
  PRIMARY KEY (cocktail_id, ingredient_id)
```

### The Two Ingredient Tables

`cocktail_ingredients` and `cocktail_required_ingredients` serve different purposes.

`cocktail_ingredients` is the full recipe list — everything you put in the glass, including optional garnishes and decorations. It is used to display the recipe card to guests.

`cocktail_required_ingredients` is the availability gate — only the ingredients that are truly indispensable to make the drink. If gin is unavailable but the recipe also calls for a lime wedge decoration, the cocktail should still be considered unavailable. But if rosemary is just a garnish, its absence should not block the cocktail.

The organizer decides which ingredients belong in `required_ingredients` when creating a recipe.

### SQLite Booleans

D1/SQLite has no native boolean type. `is_available` and `is_adapted` are stored as `INTEGER` (0/1). The `rows.rs` types reflect this (`i32`), and the conversion to `bool` happens explicitly when mapping rows to domain types.

---

## How Cocktail Availability is Calculated

Availability is not stored in the database. It is computed at query time from two pieces of information:

1. The set of ingredient UUIDs that currently have `is_available = 1`.
2. The `required_ingredients` list for each cocktail.

A cocktail is available if and only if every UUID in its `required_ingredients` list appears in the available ingredients set:

```rust
// db.rs
pub fn is_cocktail_available(cocktail: &Cocktail, available_ids: &[uuid::Uuid]) -> bool {
    cocktail
        .required_ingredients
        .iter()
        .all(|id| available_ids.contains(id))
}
```

This function takes a `Cocktail` struct and a slice of available UUIDs — it does not touch the database. The caller is responsible for fetching the available IDs first.

This design means availability is always computed fresh, never stale. When the admin toggles an ingredient, the next `GET /api/cocktails` reflects the change immediately.

---

## The `assemble_cocktails` Function

`db::assemble_cocktails` is the central function for loading complete cocktail data. It takes a `Vec<CocktailRow>` (rows from the `cocktails` table) and returns `Vec<Cocktail>` with all related data loaded.

### The N+1 Problem (and why it's avoided here)

The naive approach would be: for each cocktail, run 4 separate queries (tastes, ingredients, steps, required). With 20 cocktails, that is 80 extra queries.

`assemble_cocktails` instead runs exactly 4 queries regardless of how many cocktails there are, using `WHERE cocktail_id IN (uuid1, uuid2, ...)`. The results are grouped into `HashMap<Uuid, Vec<...>>` in memory, then each cocktail struct is assembled in a single pass:

```
Step 1: Build a comma-separated list of all cocktail IDs
Step 2: Launch 4 queries in parallel using futures::join!
Step 3: Group each result into a HashMap by cocktail_id
Step 4: Iterate the original rows, pulling from each HashMap
```

The queries run in parallel (not sequentially), so the total time is approximately the slowest single query rather than the sum of all four.

---

## Authentication

Admin routes use HTTP Basic Auth. The implementation is in `routes/admin.rs`:

1. The `check_auth` function reads the `Authorization` header, decodes the Base64 credentials, and compares them against the `ADMIN_USER` and `ADMIN_PASSWORD` Worker secrets.
2. The `require_auth!` macro is called at the top of each admin handler. If auth fails, it immediately returns `401 Unauthorized` with CORS headers and stops execution.

Secrets are never hardcoded. They are injected into the Worker at deploy time via Cloudflare's secret management (created by OpenTofu's `cloudflare_worker_secret` resources).

---

## How to Add a New Endpoint

1. **Decide which route file it belongs to.** Public, read-only routes go in `routes/cocktails.rs` or `routes/ingredients.rs`. Anything that requires auth goes in `routes/admin.rs`.

2. **Add the handler function.** Follow the signature convention:
   ```rust
   pub async fn my_handler(req: Request, ctx: RouteContext<()>) -> Result<Response> {
       require_auth!(req, ctx); // if admin
       // ...
       json_response(&serde_json::to_string(&my_struct)?)
   }
   ```

3. **Register the route in `lib.rs`.** Add a line to the router in `main`:
   ```rust
   .get_async("/api/my-resource", routes::cocktails::my_handler)
   ```

4. **Add any new model types.** If the endpoint needs new request/response shapes, add them to the appropriate file in `models/`. Request body types (payloads) go in `payloads.rs`. Raw DB row types go in `rows.rs`. Domain response types go in `cocktail.rs` or `ingredient.rs`.

5. **Re-export if needed.** `models/mod.rs` uses `pub use` to make all types available via `use crate::models::*`. If you add a new file under `models/`, add a `pub mod` and `pub use` there.

---

## How to Run Migrations

Migrations live in `worker/migrations/`. The filename prefix determines execution order.

**Apply to remote D1 (production):**
```bash
wrangler d1 execute cocktail-db --file=migrations/0001_schema.sql
```

**Apply to local dev D1 (created automatically by `wrangler dev`):**
```bash
wrangler d1 execute cocktail-db --local --file=migrations/0001_schema.sql
```

**Create a new migration:**
1. Create a new file: `migrations/0002_your_change.sql`
2. Write the SQL (use `IF NOT EXISTS` and `IF EXISTS` guards where appropriate)
3. Apply it with the commands above

There is no automatic migration runner — you apply migrations manually. This is intentional for a small project where migrations are infrequent and the risk of accidental data loss is real.

---

## How to Test

The Worker does not currently have unit tests (Cloudflare Workers wasm runtime makes standard Rust test harnesses complex to set up). Testing is done through:

**Local integration testing with `wrangler dev`:**
```bash
# Terminal 1
cd worker && wrangler dev

# Terminal 2 — test with curl
curl http://localhost:8787/api/cocktails
curl -X POST http://localhost:8787/api/admin/ingredients \
  -H "Authorization: Basic $(echo -n 'organizador:yourpass' | base64)" \
  -H "Content-Type: application/json" \
  -d '{"name":"Gin de prueba","category":"bases_alcoholicas"}'
```

**End-to-end testing via the frontend:**
Start both `wrangler dev` and `npm run dev` in the frontend, and use the admin panel normally. The frontend's Vitest tests mock the API layer.
