#!/usr/bin/env python3
"""Cocktail menu seeder.

Reads data/ingredients.json and data/cocktails.json, then calls the admin API
to create all missing ingredients and cocktails. Safe to re-run: items that
already exist in the database are skipped automatically.

Usage:
  python scripts/seed.py --url http://localhost:8787 --user organizador --password test123
  python scripts/seed.py --url https://api.example.com --user admin --password secret --dry-run
  python scripts/seed.py --url http://localhost:8787 --user admin --password pw --ingredients-only
"""
from __future__ import annotations

import argparse
import base64
import json
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Optional, TypedDict

DATA_DIR = Path(__file__).parent.parent / "data"


# ── Data shapes (from JSON files) ─────────────────────────────────────────────

class IngredientData(TypedDict):
    name: str
    category: str


class CocktailIngredientData(TypedDict):
    ingredient_name: str
    amount: str
    note: Optional[str]
    required: bool


class CocktailData(TypedDict):
    name: str
    base: str
    taste: list[str]
    glass: str
    description: str
    garnish: str
    is_adapted: bool
    adaptation_note: Optional[str]
    ingredients: list[CocktailIngredientData]
    steps: list[str]


# ── API response shapes ────────────────────────────────────────────────────────

class PaginationMeta(TypedDict):
    page: int
    limit: int
    total: int
    total_pages: int


class IngredientsPage(TypedDict):
    ingredients: list[dict[str, Any]]
    pagination: PaginationMeta


class CocktailsResponse(TypedDict):
    cocktails: list[dict[str, Any]]
    stats: dict[str, Any]


# ── HTTP helpers ───────────────────────────────────────────────────────────────

def _basic_auth(user: str, password: str) -> str:
    token = base64.b64encode(f"{user}:{password}".encode()).decode()
    return f"Basic {token}"


_UA = "Mozilla/5.0 (X11; Linux x86_64; rv:124.0) Gecko/20100101 Firefox/124.0"


def get_json(url: str, user: str, password: str) -> Any:
    req = urllib.request.Request(
        url,
        headers={"Authorization": _basic_auth(user, password), "User-Agent": _UA},
    )
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode())


def post_json(
    url: str,
    user: str,
    password: str,
    payload: dict[str, Any],
) -> tuple[int, Any]:
    data = json.dumps(payload).encode()
    req = urllib.request.Request(
        url,
        data=data,
        method="POST",
        headers={
            "Authorization": _basic_auth(user, password),
            "Content-Type": "application/json",
            "User-Agent": _UA,
        },
    )
    try:
        with urllib.request.urlopen(req) as resp:
            return resp.status, json.loads(resp.read().decode())
    except urllib.error.HTTPError as exc:
        body: Any = {}
        try:
            body = json.loads(exc.read().decode())
        except Exception:
            pass
        return exc.code, body


# ── API helpers ────────────────────────────────────────────────────────────────

def fetch_all_ingredients(
    base_url: str, user: str, password: str
) -> list[dict[str, Any]]:
    """Fetches all ingredients via the paginated public endpoint."""
    result: list[dict[str, Any]] = []
    page = 1
    while True:
        data: IngredientsPage = get_json(
            f"{base_url}/api/ingredients?page={page}&limit=50", user, password
        )
        result.extend(data["ingredients"])
        if page >= data["pagination"]["total_pages"]:
            break
        page += 1
    return result


def fetch_existing_cocktail_names(base_url: str, user: str, password: str) -> set[str]:
    """Returns the set of cocktail names already in the database."""
    data: CocktailsResponse = get_json(f"{base_url}/api/cocktails", user, password)
    return {c["name"] for c in data["cocktails"]}


# ── Seed functions ─────────────────────────────────────────────────────────────

def seed_ingredients(
    ingredients: list[IngredientData],
    base_url: str,
    user: str,
    password: str,
    *,
    dry_run: bool,
) -> bool:
    """Creates missing ingredients. Returns True if no errors occurred."""
    prefix = "[DRY RUN] " if dry_run else ""
    print(f"\n{prefix}Seeding {len(ingredients)} ingredients...")

    existing_names = {i["name"] for i in fetch_all_ingredients(base_url, user, password)}
    to_create = [i for i in ingredients if i["name"] not in existing_names]

    print(f"  {len(existing_names)} already in database, {len(to_create)} to create")

    success = True
    for ing in to_create:
        if dry_run:
            print(f"  + would create: {ing['name']} ({ing['category']})")
            continue
        status, _ = post_json(
            f"{base_url}/api/admin/ingredients",
            user,
            password,
            {"name": ing["name"], "category": ing["category"]},
        )
        if status in (200, 201):
            print(f"  + created: {ing['name']}")
        else:
            print(f"  ! failed (HTTP {status}): {ing['name']}", file=sys.stderr)
            success = False

    return success


def seed_cocktails(
    cocktails: list[CocktailData],
    base_url: str,
    user: str,
    password: str,
    *,
    dry_run: bool,
) -> bool:
    """Creates missing cocktails, resolving ingredient names to UUIDs.
    Returns True if no errors occurred.
    """
    prefix = "[DRY RUN] " if dry_run else ""
    print(f"\n{prefix}Seeding {len(cocktails)} cocktails...")

    all_ingredients = fetch_all_ingredients(base_url, user, password)
    name_to_id: dict[str, str] = {i["name"]: i["id"] for i in all_ingredients}
    existing_names = fetch_existing_cocktail_names(base_url, user, password)
    to_create = [c for c in cocktails if c["name"] not in existing_names]

    print(f"  {len(existing_names)} already in database, {len(to_create)} to create")

    success = True
    for cocktail in to_create:
        missing = [
            ing["ingredient_name"]
            for ing in cocktail["ingredients"]
            if ing["ingredient_name"] not in name_to_id
        ]
        if missing:
            print(
                f"  ! skipping '{cocktail['name']}' — unknown ingredients: {', '.join(missing)}",
                file=sys.stderr,
            )
            success = False
            continue

        ingredient_payloads: list[dict[str, Any]] = [
            {
                "ingredient_id": name_to_id[ing["ingredient_name"]],
                "amount": ing["amount"],
                "note": ing["note"],
            }
            for ing in cocktail["ingredients"]
        ]
        required_ids: list[str] = [
            name_to_id[ing["ingredient_name"]]
            for ing in cocktail["ingredients"]
            if ing["required"]
        ]
        payload: dict[str, Any] = {
            "name": cocktail["name"],
            "base": cocktail["base"],
            "taste": cocktail["taste"],
            "glass": cocktail["glass"],
            "description": cocktail["description"],
            "garnish": cocktail["garnish"],
            "is_adapted": cocktail["is_adapted"],
            "adaptation_note": cocktail["adaptation_note"],
            "ingredients": ingredient_payloads,
            "steps": cocktail["steps"],
            "required_ingredients": required_ids,
        }

        if dry_run:
            print(f"  + would create: {cocktail['name']} ({cocktail['base']})")
            continue

        status, _ = post_json(
            f"{base_url}/api/admin/cocktails", user, password, payload
        )
        if status in (200, 201):
            print(f"  + created: {cocktail['name']}")
        else:
            print(f"  ! failed (HTTP {status}): {cocktail['name']}", file=sys.stderr)
            success = False

    return success


# ── CLI ───────────────────────────────────────────────────────────────────────

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Seed ingredients and cocktails via the admin API.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "Examples:\n"
            "  %(prog)s --url http://localhost:8787 --user organizador --password test123\n"
            "  %(prog)s --url https://api.example.com --user admin --password secret --dry-run\n"
            "  %(prog)s --url http://localhost:8787 --user admin --password pw --ingredients-only\n"
        ),
    )
    parser.add_argument(
        "--url",
        required=True,
        help="Base URL of the API, e.g. http://localhost:8787",
    )
    parser.add_argument("--user", required=True, help="Admin username")
    parser.add_argument("--password", required=True, help="Admin password")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be created without sending any write requests",
    )
    scope = parser.add_mutually_exclusive_group()
    scope.add_argument(
        "--ingredients-only",
        action="store_true",
        help="Only seed ingredients, skip cocktails",
    )
    scope.add_argument(
        "--cocktails-only",
        action="store_true",
        help="Only seed cocktails (ingredients must already exist in the database)",
    )
    return parser.parse_args()


def load_json_file(path: Path) -> Any:
    if not path.exists():
        print(f"Error: {path} not found", file=sys.stderr)
        sys.exit(1)
    with path.open(encoding="utf-8") as fh:
        return json.load(fh)


def main() -> int:
    args = parse_args()
    base_url = args.url.rstrip("/")
    all_ok = True

    if not args.cocktails_only:
        ingredients: list[IngredientData] = load_json_file(DATA_DIR / "ingredients.json")
        ok = seed_ingredients(
            ingredients, base_url, args.user, args.password, dry_run=args.dry_run
        )
        all_ok = all_ok and ok

    if not args.ingredients_only:
        cocktails: list[CocktailData] = load_json_file(DATA_DIR / "cocktails.json")
        ok = seed_cocktails(
            cocktails, base_url, args.user, args.password, dry_run=args.dry_run
        )
        all_ok = all_ok and ok

    if all_ok:
        print("\nDone.")
    else:
        print("\nCompleted with errors — see lines marked '!'.", file=sys.stderr)

    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())
