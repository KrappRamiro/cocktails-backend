//! Unit tests para los modelos del dominio.
//!
//! Cubre serialización/deserialización con serde_json para todos los enums y structs,
//! round-trips JSON, y patrones de manejo de errores con payloads inválidos.

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::models::{
        cocktail::{
            Cocktail, CocktailBase, CocktailIngredient, CocktailTaste, CocktailWithAvailability,
            CocktailsResponse, GlassType, Stats,
        },
        ingredient::{Ingredient, IngredientCategory, IngredientsResponse},
        payloads::{
            CocktailIngredientPayload, CocktailPayload, IngredientAvailabilityPayload,
            IngredientPayload,
        },
    };

    // ─── Helpers ──────────────────────────────────────────────────────────────

    /// Construye un `Cocktail` mínimo válido para usar en varios tests.
    fn make_cocktail(id: Uuid, required_ingredients: Vec<Uuid>) -> Cocktail {
        Cocktail {
            id,
            name: "Negroni".to_string(),
            base: CocktailBase::Gin,
            taste: vec![CocktailTaste::Amargo, CocktailTaste::Clasico],
            glass: GlassType::VasoBajo,
            description: "Cocktail clásico italiano".to_string(),
            ingredients: vec![CocktailIngredient {
                ingredient_id: Uuid::new_v4(),
                name: "Gin (seco/dry)".to_string(),
                amount: "30ml".to_string(),
                note: Some("bien frío".to_string()),
            }],
            steps: vec!["Mezclar con hielo".to_string(), "Colar en vaso".to_string()],
            garnish: "Rodaja de naranja".to_string(),
            is_adapted: false,
            adaptation_note: None,
            required_ingredients,
        }
    }

    // ─── CocktailBase: serde snake_case ───────────────────────────────────────

    #[test]
    fn cocktail_base_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&CocktailBase::Gin).unwrap(),
            "\"gin\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::Vodka).unwrap(),
            "\"vodka\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::Tequila).unwrap(),
            "\"tequila\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::Ron).unwrap(),
            "\"ron\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::Pisco).unwrap(),
            "\"pisco\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::Whisky).unwrap(),
            "\"whisky\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::Brandy).unwrap(),
            "\"brandy\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::Caipiroska).unwrap(),
            "\"caipiroska\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::Caipirinha).unwrap(),
            "\"caipirinha\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::GinTonics).unwrap(),
            "\"gin_tonics\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::Mocktail).unwrap(),
            "\"mocktail\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::Espumante).unwrap(),
            "\"espumante\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailBase::Licores).unwrap(),
            "\"licores\""
        );
    }

    #[test]
    fn cocktail_base_deserializes_from_snake_case() {
        assert_eq!(
            serde_json::from_str::<CocktailBase>("\"gin\"").unwrap(),
            CocktailBase::Gin
        );
        assert_eq!(
            serde_json::from_str::<CocktailBase>("\"gin_tonics\"").unwrap(),
            CocktailBase::GinTonics
        );
        assert_eq!(
            serde_json::from_str::<CocktailBase>("\"mocktail\"").unwrap(),
            CocktailBase::Mocktail
        );
    }

    #[test]
    fn cocktail_base_invalid_value_fails_deserialization() {
        assert!(serde_json::from_str::<CocktailBase>("\"vodkas\"").is_err());
        assert!(serde_json::from_str::<CocktailBase>("\"Gin\"").is_err());
        assert!(serde_json::from_str::<CocktailBase>("\"GIN\"").is_err());
        assert!(serde_json::from_str::<CocktailBase>("\"bourbon\"").is_err());
    }

    // ─── CocktailTaste: serde snake_case ──────────────────────────────────────

    #[test]
    fn cocktail_taste_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&CocktailTaste::Fresco).unwrap(),
            "\"fresco\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailTaste::Frutal).unwrap(),
            "\"frutal\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailTaste::Tropical).unwrap(),
            "\"tropical\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailTaste::Clasico).unwrap(),
            "\"clasico\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailTaste::Amargo).unwrap(),
            "\"amargo\""
        );
        assert_eq!(
            serde_json::to_string(&CocktailTaste::SinAlcohol).unwrap(),
            "\"sin_alcohol\""
        );
    }

    #[test]
    fn cocktail_taste_deserializes_from_snake_case() {
        assert_eq!(
            serde_json::from_str::<CocktailTaste>("\"sin_alcohol\"").unwrap(),
            CocktailTaste::SinAlcohol
        );
        assert_eq!(
            serde_json::from_str::<CocktailTaste>("\"fresco\"").unwrap(),
            CocktailTaste::Fresco
        );
    }

    #[test]
    fn cocktail_taste_invalid_value_fails_deserialization() {
        assert!(serde_json::from_str::<CocktailTaste>("\"dulce\"").is_err());
        assert!(serde_json::from_str::<CocktailTaste>("\"Fresco\"").is_err());
        assert!(serde_json::from_str::<CocktailTaste>("\"sin_alcohol_\"").is_err());
    }

    // ─── GlassType: serde snake_case ──────────────────────────────────────────

    #[test]
    fn glass_type_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&GlassType::CopaMartini).unwrap(),
            "\"copa_martini\""
        );
        assert_eq!(
            serde_json::to_string(&GlassType::CopaBalon).unwrap(),
            "\"copa_balon\""
        );
        assert_eq!(
            serde_json::to_string(&GlassType::VasoAlto).unwrap(),
            "\"vaso_alto\""
        );
        assert_eq!(
            serde_json::to_string(&GlassType::VasoBajo).unwrap(),
            "\"vaso_bajo\""
        );
        assert_eq!(
            serde_json::to_string(&GlassType::CopaVino).unwrap(),
            "\"copa_vino\""
        );
    }

    #[test]
    fn glass_type_deserializes_from_snake_case() {
        assert_eq!(
            serde_json::from_str::<GlassType>("\"copa_martini\"").unwrap(),
            GlassType::CopaMartini
        );
        assert_eq!(
            serde_json::from_str::<GlassType>("\"vaso_alto\"").unwrap(),
            GlassType::VasoAlto
        );
    }

    #[test]
    fn glass_type_invalid_value_fails_deserialization() {
        assert!(serde_json::from_str::<GlassType>("\"copa\"").is_err());
        assert!(serde_json::from_str::<GlassType>("\"CopaMartini\"").is_err());
    }

    // ─── IngredientCategory: serde snake_case ─────────────────────────────────

    #[test]
    fn ingredient_category_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&IngredientCategory::BasesAlcoholicas).unwrap(),
            "\"bases_alcoholicas\""
        );
        assert_eq!(
            serde_json::to_string(&IngredientCategory::Licores).unwrap(),
            "\"licores\""
        );
        assert_eq!(
            serde_json::to_string(&IngredientCategory::VermutsAperitivos).unwrap(),
            "\"vermuts_aperitivos\""
        );
        assert_eq!(
            serde_json::to_string(&IngredientCategory::Amargos).unwrap(),
            "\"amargos\""
        );
        assert_eq!(
            serde_json::to_string(&IngredientCategory::MixersGaseosas).unwrap(),
            "\"mixers_gaseosas\""
        );
        assert_eq!(
            serde_json::to_string(&IngredientCategory::Jugos).unwrap(),
            "\"jugos\""
        );
        assert_eq!(
            serde_json::to_string(&IngredientCategory::FrescosYBotanicos).unwrap(),
            "\"frescos_y_botanicos\""
        );
        assert_eq!(
            serde_json::to_string(&IngredientCategory::BasicosAlacena).unwrap(),
            "\"basicos_alacena\""
        );
        assert_eq!(
            serde_json::to_string(&IngredientCategory::Decoracion).unwrap(),
            "\"decoracion\""
        );
    }

    #[test]
    fn ingredient_category_deserializes_from_snake_case() {
        assert_eq!(
            serde_json::from_str::<IngredientCategory>("\"bases_alcoholicas\"").unwrap(),
            IngredientCategory::BasesAlcoholicas
        );
        assert_eq!(
            serde_json::from_str::<IngredientCategory>("\"frescos_y_botanicos\"").unwrap(),
            IngredientCategory::FrescosYBotanicos
        );
        assert_eq!(
            serde_json::from_str::<IngredientCategory>("\"decoracion\"").unwrap(),
            IngredientCategory::Decoracion
        );
    }

    #[test]
    fn ingredient_category_invalid_value_fails_deserialization() {
        assert!(serde_json::from_str::<IngredientCategory>("\"alcohol\"").is_err());
        assert!(serde_json::from_str::<IngredientCategory>("\"BasesAlcoholicas\"").is_err());
    }

    // ─── Ingredient: round-trip ───────────────────────────────────────────────

    #[test]
    fn ingredient_round_trip() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let ingredient = Ingredient {
            id,
            name: "Gin (seco/dry)".to_string(),
            category: IngredientCategory::BasesAlcoholicas,
            is_available: true,
        };

        let json = serde_json::to_string(&ingredient).unwrap();
        let parsed: Ingredient = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, ingredient.id);
        assert_eq!(parsed.name, ingredient.name);
        assert_eq!(parsed.category, ingredient.category);
        assert_eq!(parsed.is_available, ingredient.is_available);
    }

    #[test]
    fn ingredient_json_shape() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let ingredient = Ingredient {
            id,
            name: "Campari".to_string(),
            category: IngredientCategory::Amargos,
            is_available: false,
        };

        let value: serde_json::Value = serde_json::to_value(&ingredient).unwrap();

        assert_eq!(value["id"], "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(value["name"], "Campari");
        assert_eq!(value["category"], "amargos");
        assert_eq!(value["is_available"], false);
    }

    // ─── Cocktail: round-trip ─────────────────────────────────────────────────

    #[test]
    fn cocktail_round_trip() {
        let id = Uuid::new_v4();
        let req_id = Uuid::new_v4();
        let cocktail = make_cocktail(id, vec![req_id]);

        let json = serde_json::to_string(&cocktail).unwrap();
        let parsed: Cocktail = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, cocktail.id);
        assert_eq!(parsed.name, cocktail.name);
        assert_eq!(parsed.base, CocktailBase::Gin);
        assert_eq!(
            parsed.taste,
            vec![CocktailTaste::Amargo, CocktailTaste::Clasico]
        );
        assert_eq!(parsed.glass, GlassType::VasoBajo);
        assert_eq!(parsed.required_ingredients, vec![req_id]);
        assert!(!parsed.is_adapted);
        assert!(parsed.adaptation_note.is_none());
    }

    #[test]
    fn cocktail_json_field_names() {
        let id = Uuid::new_v4();
        let cocktail = make_cocktail(id, vec![]);

        let value: serde_json::Value = serde_json::to_value(&cocktail).unwrap();

        // Verificar que las claves snake_case están presentes
        assert!(value.get("id").is_some());
        assert!(value.get("name").is_some());
        assert!(value.get("base").is_some());
        assert!(value.get("taste").is_some());
        assert!(value.get("glass").is_some());
        assert!(value.get("description").is_some());
        assert!(value.get("ingredients").is_some());
        assert!(value.get("steps").is_some());
        assert!(value.get("garnish").is_some());
        assert!(value.get("is_adapted").is_some());
        assert!(value.get("adaptation_note").is_some());
        assert!(value.get("required_ingredients").is_some());
        // Los valores de los enums deben ser snake_case
        assert_eq!(value["base"], "gin");
        assert_eq!(value["glass"], "vaso_bajo");
        assert_eq!(value["taste"][0], "amargo");
        assert_eq!(value["taste"][1], "clasico");
    }

    #[test]
    fn cocktail_with_adaptation_note_round_trip() {
        let id = Uuid::new_v4();
        let mut cocktail = make_cocktail(id, vec![]);
        cocktail.is_adapted = true;
        cocktail.adaptation_note = Some("Sin bitter, usar Angostura".to_string());

        let json = serde_json::to_string(&cocktail).unwrap();
        let parsed: Cocktail = serde_json::from_str(&json).unwrap();

        assert!(parsed.is_adapted);
        assert_eq!(
            parsed.adaptation_note,
            Some("Sin bitter, usar Angostura".to_string())
        );
    }

    // ─── CocktailWithAvailability: round-trip y flatten ───────────────────────

    #[test]
    fn cocktail_with_availability_flatten() {
        let id = Uuid::new_v4();
        let cocktail = make_cocktail(id, vec![]);
        let cwa = CocktailWithAvailability {
            cocktail,
            is_available: true,
        };

        let value: serde_json::Value = serde_json::to_value(&cwa).unwrap();

        // Gracias a #[serde(flatten)], los campos de Cocktail están al nivel raíz
        assert!(value.get("id").is_some(), "id should be at root level");
        assert!(value.get("name").is_some(), "name should be at root level");
        assert!(value.get("base").is_some(), "base should be at root level");
        assert_eq!(value["is_available"], true);
        // No debe existir una clave "cocktail" anidada
        assert!(
            value.get("cocktail").is_none(),
            "should not have nested 'cocktail' key"
        );
    }

    #[test]
    fn cocktail_with_availability_round_trip() {
        let id = Uuid::new_v4();
        let cocktail = make_cocktail(id, vec![]);
        let cwa = CocktailWithAvailability {
            cocktail,
            is_available: false,
        };

        let json = serde_json::to_string(&cwa).unwrap();
        let parsed: CocktailWithAvailability = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.cocktail.id, id);
        assert!(!parsed.is_available);
    }

    // ─── Stats: serialización ─────────────────────────────────────────────────

    #[test]
    fn stats_serializes_correctly() {
        let stats = Stats {
            total: 10,
            available: 7,
        };
        let value: serde_json::Value = serde_json::to_value(&stats).unwrap();
        assert_eq!(value["total"], 10);
        assert_eq!(value["available"], 7);
    }

    #[test]
    fn stats_round_trip() {
        let stats = Stats {
            total: 0,
            available: 0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let parsed: Stats = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.total, 0);
        assert_eq!(parsed.available, 0);
    }

    // ─── CocktailsResponse: serialización ────────────────────────────────────

    #[test]
    fn cocktails_response_serializes_correctly() {
        let id = Uuid::new_v4();
        let cocktail = make_cocktail(id, vec![]);
        let response = CocktailsResponse {
            cocktails: vec![CocktailWithAvailability {
                cocktail,
                is_available: true,
            }],
            stats: Stats {
                total: 1,
                available: 1,
            },
        };

        let value: serde_json::Value = serde_json::to_value(&response).unwrap();

        assert!(value["cocktails"].is_array());
        assert_eq!(value["cocktails"].as_array().unwrap().len(), 1);
        assert_eq!(value["stats"]["total"], 1);
        assert_eq!(value["stats"]["available"], 1);
        // El cocktail dentro del array debe tener los campos aplanados
        assert!(value["cocktails"][0].get("id").is_some());
        assert!(value["cocktails"][0].get("is_available").is_some());
    }

    #[test]
    fn cocktails_response_empty() {
        let response = CocktailsResponse {
            cocktails: vec![],
            stats: Stats {
                total: 0,
                available: 0,
            },
        };
        let value: serde_json::Value = serde_json::to_value(&response).unwrap();
        assert_eq!(value["cocktails"].as_array().unwrap().len(), 0);
        assert_eq!(value["stats"]["total"], 0);
    }

    // ─── CocktailPayload: deserialización ────────────────────────────────────

    #[test]
    fn cocktail_payload_valid_json() {
        let ing_id = Uuid::new_v4();
        let req_id = Uuid::new_v4();
        let json = format!(
            r#"{{
                "name": "Negroni",
                "base": "gin",
                "taste": ["amargo", "clasico"],
                "glass": "vaso_bajo",
                "description": "Clásico italiano",
                "ingredients": [{{
                    "ingredient_id": "{}",
                    "amount": "30ml",
                    "note": "bien frío"
                }}],
                "steps": ["Mezclar con hielo", "Colar"],
                "garnish": "Naranja",
                "is_adapted": false,
                "adaptation_note": null,
                "required_ingredients": ["{}"]
            }}"#,
            ing_id, req_id
        );

        let payload: CocktailPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload.name, "Negroni");
        assert_eq!(payload.base, CocktailBase::Gin);
        assert_eq!(
            payload.taste,
            vec![CocktailTaste::Amargo, CocktailTaste::Clasico]
        );
        assert_eq!(payload.glass, GlassType::VasoBajo);
        assert_eq!(payload.ingredients.len(), 1);
        assert_eq!(payload.ingredients[0].amount, "30ml");
        assert_eq!(payload.ingredients[0].note, Some("bien frío".to_string()));
        assert_eq!(payload.steps, vec!["Mezclar con hielo", "Colar"]);
        assert!(!payload.is_adapted);
        assert!(payload.adaptation_note.is_none());
        assert_eq!(payload.required_ingredients, vec![req_id]);
    }

    #[test]
    fn cocktail_payload_invalid_base_fails() {
        let json = r#"{
            "name": "Test",
            "base": "bourbon",
            "taste": ["fresco"],
            "glass": "vaso_alto",
            "description": "desc",
            "ingredients": [],
            "steps": [],
            "garnish": "none",
            "is_adapted": false,
            "adaptation_note": null,
            "required_ingredients": []
        }"#;
        assert!(serde_json::from_str::<CocktailPayload>(json).is_err());
    }

    #[test]
    fn cocktail_payload_invalid_taste_fails() {
        let json = r#"{
            "name": "Test",
            "base": "gin",
            "taste": ["dulce"],
            "glass": "vaso_alto",
            "description": "desc",
            "ingredients": [],
            "steps": [],
            "garnish": "none",
            "is_adapted": false,
            "adaptation_note": null,
            "required_ingredients": []
        }"#;
        assert!(serde_json::from_str::<CocktailPayload>(json).is_err());
    }

    #[test]
    fn cocktail_payload_invalid_glass_fails() {
        let json = r#"{
            "name": "Test",
            "base": "vodka",
            "taste": ["frutal"],
            "glass": "copa_grande",
            "description": "desc",
            "ingredients": [],
            "steps": [],
            "garnish": "none",
            "is_adapted": false,
            "adaptation_note": null,
            "required_ingredients": []
        }"#;
        assert!(serde_json::from_str::<CocktailPayload>(json).is_err());
    }

    #[test]
    fn cocktail_payload_with_adaptation_note() {
        let json = r#"{
            "name": "Negroni Adaptado",
            "base": "gin",
            "taste": ["amargo"],
            "glass": "vaso_bajo",
            "description": "Versión sin Campari",
            "ingredients": [],
            "steps": [],
            "garnish": "naranja",
            "is_adapted": true,
            "adaptation_note": "Reemplazar Campari por Aperol",
            "required_ingredients": []
        }"#;

        let payload: CocktailPayload = serde_json::from_str(json).unwrap();
        assert!(payload.is_adapted);
        assert_eq!(
            payload.adaptation_note,
            Some("Reemplazar Campari por Aperol".to_string())
        );
    }

    // ─── IngredientPayload: deserialización ───────────────────────────────────

    #[test]
    fn ingredient_payload_valid_json() {
        let json = r#"{"name": "Gin (seco/dry)", "category": "bases_alcoholicas"}"#;
        let payload: IngredientPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Gin (seco/dry)");
        assert_eq!(payload.category, IngredientCategory::BasesAlcoholicas);
    }

    #[test]
    fn ingredient_payload_invalid_category_fails() {
        let json = r#"{"name": "Cerveza", "category": "cervezas"}"#;
        assert!(serde_json::from_str::<IngredientPayload>(json).is_err());
    }

    #[test]
    fn ingredient_payload_missing_field_fails() {
        let json = r#"{"name": "Gin"}"#;
        assert!(serde_json::from_str::<IngredientPayload>(json).is_err());
    }

    #[test]
    fn ingredient_payload_all_categories() {
        let categories = [
            ("bases_alcoholicas", IngredientCategory::BasesAlcoholicas),
            ("licores", IngredientCategory::Licores),
            ("vermuts_aperitivos", IngredientCategory::VermutsAperitivos),
            ("amargos", IngredientCategory::Amargos),
            ("mixers_gaseosas", IngredientCategory::MixersGaseosas),
            ("jugos", IngredientCategory::Jugos),
            ("frescos_y_botanicos", IngredientCategory::FrescosYBotanicos),
            ("basicos_alacena", IngredientCategory::BasicosAlacena),
            ("decoracion", IngredientCategory::Decoracion),
        ];
        for (snake, variant) in &categories {
            let json = format!(r#"{{"name": "X", "category": "{}"}}"#, snake);
            let payload: IngredientPayload = serde_json::from_str(&json).unwrap();
            assert_eq!(payload.category, *variant, "failed for category: {}", snake);
        }
    }

    // ─── IngredientAvailabilityPayload: deserialización ───────────────────────

    #[test]
    fn ingredient_availability_payload_true() {
        let json = r#"{"available": true}"#;
        let payload: IngredientAvailabilityPayload = serde_json::from_str(json).unwrap();
        assert!(payload.available);
    }

    #[test]
    fn ingredient_availability_payload_false() {
        let json = r#"{"available": false}"#;
        let payload: IngredientAvailabilityPayload = serde_json::from_str(json).unwrap();
        assert!(!payload.available);
    }

    #[test]
    fn ingredient_availability_payload_missing_field_fails() {
        let json = r#"{}"#;
        assert!(serde_json::from_str::<IngredientAvailabilityPayload>(json).is_err());
    }

    #[test]
    fn ingredient_availability_payload_wrong_type_fails() {
        // "available" debe ser un booleano, no un string
        let json = r#"{"available": "true"}"#;
        assert!(serde_json::from_str::<IngredientAvailabilityPayload>(json).is_err());
    }

    // ─── IngredientsResponse: serialización ──────────────────────────────────

    #[test]
    fn ingredients_response_serializes_correctly() {
        let id = Uuid::new_v4();
        let response = IngredientsResponse {
            ingredients: vec![Ingredient {
                id,
                name: "Campari".to_string(),
                category: IngredientCategory::Amargos,
                is_available: true,
            }],
        };

        let value: serde_json::Value = serde_json::to_value(&response).unwrap();
        assert!(value["ingredients"].is_array());
        assert_eq!(value["ingredients"].as_array().unwrap().len(), 1);
        assert_eq!(value["ingredients"][0]["name"], "Campari");
        assert_eq!(value["ingredients"][0]["category"], "amargos");
        assert_eq!(value["ingredients"][0]["is_available"], true);
    }

    // ─── uuid::Uuid: serialización como string en JSON ────────────────────────

    #[test]
    fn uuid_serializes_as_string() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"550e8400-e29b-41d4-a716-446655440000\"");
    }

    #[test]
    fn uuid_deserializes_from_string() {
        let json = "\"550e8400-e29b-41d4-a716-446655440000\"";
        let id: Uuid = serde_json::from_str(json).unwrap();
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn uuid_invalid_string_fails_deserialization() {
        assert!(serde_json::from_str::<Uuid>("\"not-a-uuid\"").is_err());
        assert!(serde_json::from_str::<Uuid>("\"\"").is_err());
    }

    #[test]
    fn uuid_in_ingredient_serializes_as_string() {
        let id = Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap();
        let ingredient = Ingredient {
            id,
            name: "Test".to_string(),
            category: IngredientCategory::Jugos,
            is_available: false,
        };
        let value: serde_json::Value = serde_json::to_value(&ingredient).unwrap();
        // El UUID debe estar como string, no como objeto
        assert!(value["id"].is_string());
        assert_eq!(value["id"], "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
    }

    // ─── CocktailIngredientPayload: note opcional ─────────────────────────────

    #[test]
    fn cocktail_ingredient_payload_with_null_note() {
        let id = Uuid::new_v4();
        let json = format!(
            r#"{{"ingredient_id": "{}", "amount": "50ml", "note": null}}"#,
            id
        );
        let payload: CocktailIngredientPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload.ingredient_id, id);
        assert_eq!(payload.amount, "50ml");
        assert!(payload.note.is_none());
    }

    #[test]
    fn cocktail_ingredient_payload_without_note_field_defaults_to_none() {
        // serde trata campos Option<T> ausentes como None por defecto
        let id = Uuid::new_v4();
        let json = format!(r#"{{"ingredient_id": "{}", "amount": "50ml"}}"#, id);
        let payload = serde_json::from_str::<CocktailIngredientPayload>(&json).unwrap();
        assert!(payload.note.is_none());
        assert_eq!(payload.amount, "50ml");
    }

    #[test]
    fn cocktail_ingredient_payload_with_note() {
        let id = Uuid::new_v4();
        let json = format!(
            r#"{{"ingredient_id": "{}", "amount": "2 cdas", "note": "o Ramazotti"}}"#,
            id
        );
        let payload: CocktailIngredientPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload.note, Some("o Ramazotti".to_string()));
    }
}
