# tax_ctrl_core

Core Rust engine for the tax_ctrl app. This crate is embedded into the
Flutter application via [Flutter Rust Bridge](https://cjycode.com/flutter_rust_bridge/)
and owns the business logic and persistence layer.

## Architecture

`tax_ctrl_core` is **runtime-swappable**: it does not hardcode a single
database. It connects to whatever backend you configure via a
`DATABASE_URL` connection string at startup, resolved through
[SeaORM](https://www.sea-ql.org/SeaORM/):

| Environment | Backend | Why |
| --- | --- | --- |
| Local-first / development | SQLite | No external service, embeds a single file, ideal for offline-first use |
| Shared office / production | PostgreSQL | Standard for multi-user, server-backed deployments |

No code changes are needed to switch — only the connection string. See
[`src/persistence/connection.rs`](src/persistence/connection.rs) for the
implementation.

```
src/
└── persistence/
    ├── connection.rs   # DbConfig + connect() — resolves SQLite vs PostgreSQL at runtime
    ├── schema.rs        # Migrator — where migrations get registered
    ├── entities/        # SeaORM entity definitions (generated or hand-written)
    └── repository.rs    # Generic CRUD repository built on top of entities
```

## Requirements

- Rust, installed via [rustup](https://rustup.rs/). The exact toolchain is
  pinned in [`rust-toolchain.toml`](rust-toolchain.toml) — `rustup` will
  install it automatically the first time you run `cargo` in this repo.
- Optional: [Docker](https://www.docker.com/), if you want to test against
  a real PostgreSQL instance locally (see below).

## Getting started

```bash
cp example.env .env
```

By default `.env` is configured for the local-first SQLite backend — no
further setup required:

```bash
cargo build
cargo test
```

`cargo test` always exercises the SQLite path. It also includes a
PostgreSQL connection test that is **skipped unless** `TEST_DATABASE_URL`
is set (see [Testing against PostgreSQL](#testing-against-postgresql)
below) — it isn't a hard dependency for everyday development.

### Switching to PostgreSQL

Comment out the SQLite line in `.env` and uncomment/configure the
PostgreSQL line instead:

```env
DATABASE_URL=postgres://user:password@localhost:5432/tax_ctrl
```

Everything else — `connect()`, `connect_from_env()`, repositories,
migrations — works unmodified against either backend.

### Testing against PostgreSQL

```bash
docker run --rm -d --name tax-ctrl-postgres \
  -e POSTGRES_USER=tax_ctrl -e POSTGRES_PASSWORD=tax_ctrl -e POSTGRES_DB=tax_ctrl \
  -p 5432:5432 postgres:16

export TEST_DATABASE_URL=postgres://tax_ctrl:tax_ctrl@localhost:5432/tax_ctrl
cargo test
```

## Continuous Integration

Every push and pull request runs, via GitHub Actions
([`.github/workflows/ci.yml`](.github/workflows/ci.yml)):

1. **Format check** — `cargo fmt --all -- --check`
2. **Clippy** — `cargo clippy --all-targets --all-features -- -D warnings`
3. **Build & test (SQLite)** — the default, always-on path
4. **Test (PostgreSQL)** — runs the same test suite against a real
   `postgres:16` service container, so the "runtime-swappable" claim is
   actually verified on every push, not just asserted

A pull request is expected to pass all four before merging.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full development workflow:
running checks locally before pushing, adding migrations/entities, commit
and branch conventions, and project layout details.

## License

Not yet decided — to be added before this crate is shared outside the
team.
