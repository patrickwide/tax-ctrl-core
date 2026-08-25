# Contributing to tax_ctrl_core

## Prerequisites

Install Rust via [rustup](https://rustup.rs/). You don't need to pick a
version — the toolchain and components (`rustfmt`, `clippy`) are pinned
in [`rust-toolchain.toml`](rust-toolchain.toml) and rustup will install
them automatically the first time you run any `cargo` command here. This
keeps your local checks aligned with what CI runs.

## Day-to-day commands

Run these before pushing — they're exactly what CI checks, so running
them locally means no surprises on the PR:

```bash
cargo fmt --all              # auto-format
cargo fmt --all -- --check   # verify formatting without changing files
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

If `cargo fmt -- --check` fails on your PR, just run `cargo fmt --all`
and commit the result — don't hand-edit whitespace to match.

## Testing against both database backends

`cargo test` alone only exercises SQLite (in-memory, no setup needed).
To also exercise the PostgreSQL path locally:

```bash
docker run --rm -d --name tax-ctrl-postgres \
  -e POSTGRES_USER=tax_ctrl -e POSTGRES_PASSWORD=tax_ctrl -e POSTGRES_DB=tax_ctrl \
  -p 5432:5432 postgres:16

export TEST_DATABASE_URL=postgres://tax_ctrl:tax_ctrl@localhost:5432/tax_ctrl
cargo test
```

CI always runs both (see [`.github/workflows/ci.yml`](.github/workflows/ci.yml)),
so this is optional locally but useful when touching anything in
`src/persistence/`.

## Adding a migration

Migrations are registered in [`src/persistence/schema.rs`](src/persistence/schema.rs).

1. Add a new migration module (by convention, timestamp-prefixed), e.g.
   `src/persistence/migrations/m20240101_000001_create_example_table.rs`,
   implementing `sea_orm_migration::MigrationTrait`.
2. Register it in `Migrator::migrations()` in `schema.rs`, in
   chronological order.
3. Apply it with:
   ```rust
   use sea_orm_migration::MigratorTrait;
   Migrator::up(&db, None).await?;
   ```

Because `Migrator` runs against whatever connection you pass it, the
same migration code applies to both SQLite and PostgreSQL — no
backend-specific branching.

## Adding an entity

Once a table exists via a migration, generate the SeaORM entity for it:

```bash
cargo install sea-orm-cli   # one-time
sea-orm-cli generate entity \
    --database-url "$DATABASE_URL" \
    --output-dir src/persistence/entities
```

Then declare the new module in `src/persistence/entities/mod.rs`
(`pub mod <table_name>;`). Hand-writing an entity following the same
`DeriveEntityModel` shape (documented in that file) is also fine for
small, stable tables.

## Repositories

Prefer composing the generic `Repository<E>` in
[`src/persistence/repository.rs`](src/persistence/repository.rs) for
basic find/list operations rather than re-implementing them per entity.
Add domain-specific query methods on top of it as needed once real
entities exist.

## Branching and commits

- Branch names: `feat/<short-description>`, `fix/<short-description>`,
  `chore/<short-description>`.
- Commit messages: a short imperative summary line, with a body
  explaining *why* when the change isn't self-evident from the diff.
- Keep PRs focused — setup/infrastructure changes separate from feature
  work where practical.

## Before opening a PR

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test --all-features` passes (SQLite; run the PostgreSQL
      steps above too if you touched `src/persistence/`)
- [ ] New public items have doc comments (`///`) explaining what they do
      and why, following the existing style in `src/persistence/`
