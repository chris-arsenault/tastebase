# Futures

Backlog of features to design for but not build yet. These inform architectural
decisions (extensible types, flexible schemas) without adding implementation scope.

## Recipe Rating & Review

Extend the recipe model with the same media-capture pattern used for tastings:
photograph the finished dish, record a voice-over review, run through the
enrichment pipeline to extract scores and notes. Requires:
- `recipe_reviews` table (or polymorphic review model shared with tastings)
- Processing pipeline generalized beyond tasting-specific field names
- Frontend: camera + voice capture on recipe detail page

## Tasting <-> Recipe Linking

A tasting can reference a recipe ("used this hot sauce in this recipe") and a
recipe can link to tastings of its results. Requires:
- Join table or FK from tastings to recipes (nullable)
- UI for linking during tasting creation
- Recipe detail page showing linked tastings

## `save_tasting` MCP Tool

Expose a second MCP tool so Claude can save tasting notes directly. Mirrors
`save_recipe` but targets the tasting data model. Lower priority since tastings
are photo/voice-driven, but useful for text-based tasting notes from
conversations.

## Recipe Editing In-App

Users may want to edit Claude-sourced recipes. Add:
- `dirty` flag or version lineage on recipes
- Edit UI respecting round-trip fidelity (ingredient tokens in steps)
- Conflict detection if Claude re-saves the same recipe

## Public Sharing / Visibility

Recipes and tastings are currently public-read. When per-item visibility controls
are needed:
- Add `visibility` enum (`public`, `private`, `unlisted`) to both tables
- Update read-path queries to filter by visibility + ownership
- Shareable links for unlisted items

## Dynamic Client Registration (DCR)

Eliminate the Claude.ai copy-paste setup for MCP connector:
- Lambda-backed `POST /register` endpoint
- Calls Cognito `CreateUserPoolClient` dynamically
- Returns client credentials to Claude.ai automatically
- Only needed if copy-paste UX becomes a real friction point

## Duplicate Recipe Detection

Claude can call `save_recipe` twice for the same recipe. Options:
- Upsert on `(user_id, title)` hash
- Surface a conflict response to Claude for user resolution
- Content-hash-based dedup

## Collection Management UI

Recipes can belong to collections (schema supports it). Build:
- Collection CRUD in frontend
- Drag-and-drop recipe organization
- Collection sharing
