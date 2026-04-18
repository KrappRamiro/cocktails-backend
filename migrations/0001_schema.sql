-- ─────────────────────────────────────────────
-- INGREDIENTES
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS ingredients (
    id           TEXT    PRIMARY KEY,
    name         TEXT    NOT NULL UNIQUE,
    category     TEXT    NOT NULL,
    is_available INTEGER NOT NULL DEFAULT 0  -- 0 = false, 1 = true
);

-- ─────────────────────────────────────────────
-- COCKTAILS
-- ─────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS cocktails (
    id               TEXT PRIMARY KEY,
    name             TEXT NOT NULL UNIQUE,
    base             TEXT NOT NULL,
    glass            TEXT NOT NULL,
    description      TEXT NOT NULL,
    garnish          TEXT NOT NULL,
    is_adapted       INTEGER NOT NULL DEFAULT 0,  -- 0 = false, 1 = true
    adaptation_note  TEXT
);

-- Gustos del cocktail (uno a muchos)
CREATE TABLE IF NOT EXISTS cocktail_tastes (
    cocktail_id  TEXT NOT NULL REFERENCES cocktails(id) ON DELETE CASCADE,
    taste        TEXT NOT NULL,
    PRIMARY KEY (cocktail_id, taste)
);

-- Ingredientes con cantidad (para mostrar la receta)
CREATE TABLE IF NOT EXISTS cocktail_ingredients (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    cocktail_id    TEXT NOT NULL REFERENCES cocktails(id) ON DELETE CASCADE,
    ingredient_id  TEXT NOT NULL REFERENCES ingredients(id),
    amount         TEXT NOT NULL,
    note           TEXT,
    sort_order     INTEGER NOT NULL DEFAULT 0
);

-- Pasos de preparación ordenados
CREATE TABLE IF NOT EXISTS cocktail_steps (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    cocktail_id  TEXT NOT NULL REFERENCES cocktails(id) ON DELETE CASCADE,
    step_order   INTEGER NOT NULL,
    description  TEXT NOT NULL
);

-- Ingredientes requeridos para calcular disponibilidad
CREATE TABLE IF NOT EXISTS cocktail_required_ingredients (
    cocktail_id    TEXT NOT NULL REFERENCES cocktails(id) ON DELETE CASCADE,
    ingredient_id  TEXT NOT NULL REFERENCES ingredients(id),
    PRIMARY KEY (cocktail_id, ingredient_id)
);

-- ─────────────────────────────────────────────
-- ÍNDICES
-- ─────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_cocktails_base ON cocktails(base);
CREATE INDEX IF NOT EXISTS idx_ingredients_available ON ingredients(is_available);
CREATE INDEX IF NOT EXISTS idx_cocktail_ingredients_cocktail ON cocktail_ingredients(cocktail_id);
CREATE INDEX IF NOT EXISTS idx_cocktail_steps_cocktail ON cocktail_steps(cocktail_id);
CREATE INDEX IF NOT EXISTS idx_cocktail_required_cocktail ON cocktail_required_ingredients(cocktail_id);
