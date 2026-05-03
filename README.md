# Web Service Boilerplate

A production-grade Rust web service boilerplate built around strict architectural boundaries, testability, and long-term maintainability. The reference implementation is a task-management service, but the structure is designed to be reused across services.

---

## Architecture

### Layer Model

```
┌─────────────────────────────────────┐
│              http/                  │  Axum handlers, schemas, extractors,
│         (entry point)               │  middleware — no business logic
└──────────────┬──────────────────────┘
               │ calls
┌──────────────▼──────────────────────┐
│           application/              │  Use-case services — orchestrates domain
│       (orchestration layer)         │  objects and calls ports; owns UoW lifecycle
└──────────────┬──────────────────────┘
               │ depends on
┌──────────────▼──────────────────────┐
│              domain/                │  Pure business types, invariants, port
│          (pure core)                │  traits — zero infrastructure dependencies
└──────────────▲──────────────────────┘
               │ implements
┌──────────────┴──────────────────────┐
│           adapters/                 │  PostgreSQL repositories, telemetry wiring
│       (infrastructure)              │  — plugged in at the composition root
└─────────────────────────────────────┘
               ▲
               │ wires
┌──────────────┴──────────────────────┐
│              src/                   │  Composition root: config loading,
│     (composition root)              │  AppState construction, server startup
└─────────────────────────────────────┘
```

**Dependency rule**: each layer may depend only on layers to its right in the original description. No infrastructure types leak into domain or application; no HTTP types leak into domain or application.

### Workspace Crates

| Crate | Role |
|---|---|
| `domain/` | Entities, value objects, `DomainError`, port traits (`UserRepository`, `TaskRepository`, `UnitOfWork`, …) |
| `application/` | `TaskService`, `UserService`, `AppError` — pure use-case logic, no I/O |
| `http/` | Axum router, handlers, request/response schemas, extractors (`AuthenticatedUser`, `ValidatedJson`), middleware |
| `adapters/postgres/` | `PostgresTaskRepository`, `PostgresUnitOfWork`, `PostgresUowFactory` via SQLx |
| `adapters/telemetry/` | Tracing-subscriber initialization |
| `forma/` | Local validation crate used by HTTP schemas |
| `src/` | `main.rs`, `AppState`, `Settings` — wires all crates together |

### Key Structural Decisions

**Domain is the source of truth.** Port traits live in `domain/ports/`. Adapters implement them; the application layer calls them through the trait. Nothing in the domain knows about Postgres, HTTP, or any framework.

**Ports and adapters through the Unit of Work.** `UnitOfWork` owns the transaction and hands out repositories. `commit` / `rollback` are called exclusively in the application layer — never in handlers, never in adapters directly.

**Error chain without leaking layers.**
```
DomainError  →  AppError  →  ProblemDetails (RFC 7807)
   domain       application      http/src/error.rs
```
HTTP-to-status mapping lives entirely in `http/src/error.rs`.

**Request validation at the HTTP boundary.** `ValidatedJson<T>` and `ValidatedQuery<T>` reject bad input before it reaches the application layer. All request DTOs carry `#[serde(deny_unknown_fields)]`.

**Middleware layering.**
- `TraceLayer` + `trace_id` extraction: applied globally — every request, including public routes.
- `authenticate` (Bearer token → `Subject`): applied per route group. Public routes have no auth middleware. Protected routes require it. Never applied globally.

**AppState holds services only.** Settings are consumed at wiring time; only `Arc<XService>` fields remain.

---

## Directory Layout

Each domain concept (entity) gets its own file or module at every layer. The pattern repeats consistently so that adding a new entity is a matter of following the same structure, not making structural decisions.

```
web-service-boilerplate/
├── domain/
│   └── src/
│       ├── {entity}.rs         # Aggregate or entity: types, invariants, state-transition guards
│       ├── ...                 # One file per domain concept
│       ├── error.rs            # DomainError — all domain-level failure cases
│       └── ports/mod.rs        # Repository and UoW port traits — one trait per entity
├── application/
│   └── src/
│       ├── services/
│       │   └── {entity}_service.rs  # Use-case methods for the entity; owns UoW lifecycle
│       └── error.rs                 # AppError — maps domain errors + application-level failures
├── http/
│   └── src/
│       ├── routes/
│       │   ├── mod.rs               # Route groups: public / authenticated; composes entity routers
│       │   └── {entity}/
│       │       ├── mod.rs           # Port trait the handler depends on + router()
│       │       └── handlers.rs      # Handler functions (private to the module)
│       ├── schemas/
│       │   └── {entity}/
│       │       ├── requests.rs      # Request DTOs (deserialization + validation)
│       │       └── response.rs      # Response DTOs (serialization)
│       ├── extractors/              # Shared extractors: AuthenticatedUser, ValidatedJson, TraceId
│       ├── middleware/              # trace (global), authenticate (per route group)
│       └── error.rs                 # AppError → ProblemDetails (RFC 7807) mapping
├── adapters/
│   ├── postgres/src/
│   │   ├── repos/                   # {Db}{Entity}Repository — one file per entity
│   │   ├── uow.rs                   # UnitOfWork impl: owns transaction, hands out repositories
│   │   └── pool.rs                  # Connection pool construction
│   └── telemetry/src/               # Tracing subscriber initialization
├── forma/                           # Local normalization + validation crate
├── migrations/                      # SQLx migrations — run automatically on startup
└── src/
    ├── config/mod.rs                # Settings, per-adapter config structs
    ├── app/state.rs                 # AppState: wires repos → UoW factory → services
    └── main.rs                      # Entry point: load config, init telemetry, serve
```

The `{entity}` placeholder stands for any domain concept you introduce. The reference implementation ships with a pair of entities used to exercise and stabilise the architecture patterns — they are not the target domain. When you add your own entities, each one follows the same layout: one aggregate in `domain/`, one service in `application/`, one repository in `adapters/postgres/repos/`, one route directory in `http/routes/`, and one schema directory in `http/schemas/`.

---

## API

Routes are introduced under `http/src/routes/`. Each entity gets its own subdirectory with a `mod.rs` that declares the port trait the handlers depend on and a `router()` function, and a `handlers.rs` that contains the handler functions. `routes/mod.rs` composes entity routers and assigns them to access groups.

**Access groups** control which middleware stack applies:

- **Public** — no authentication required (e.g. health check, login). Added directly to the router.
- **Authenticated** — requires a valid Bearer token. Routers merged under the `authenticate` middleware layer.
- **Role-scoped** — authenticated plus an additional authorization check. Nested further inside the authenticated group.

All routes sit under a versioned path prefix (`/api/v1/…`). Path segments must not expose internal crate or module names.

Every non-2xx response follows [RFC 7807](https://www.rfc-editor.org/rfc/rfc7807) Problem Details and includes a `trace_id` field for log correlation. The mapping from `AppError` to status code and problem type lives exclusively in `http/src/error.rs`.

---

## Configuration

Settings are loaded in order (later sources override earlier ones):

1. `Settings.toml` — base defaults checked into the repo
2. `Settings.local.toml` — local overrides (gitignored)
3. Environment variables prefixed `APP__` (double underscore as separator)

```toml
[server]
addr = "0.0.0.0"
port = 3000

[database]
provider = "postgres"

[database.config]
url = "postgres://user:pass@localhost/db"
max_connections = 10
connection_timeout_seconds = 5

[telemetry]
format = "pretty"   # json | pretty | compact
level  = "debug"
```

---

## Running Locally

```bash
# start postgres (adjust url in Settings.local.toml)
cargo run
```

Migrations run automatically on startup via `sqlx::migrate!`.

---

## Quality Checks

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

---

## Adding a New Entity

Follow the vertical-slice pattern: domain types and port traits → application service → postgres adapter → HTTP schemas and handlers → wire in `AppState` and `routes/mod.rs`. The [`vertical-slice` skill](.claude/skills/vertical-slice.md) documents each step in detail.

---

## Hard Rules (non-negotiable)

- No `#[async_trait]` — use native `async fn` in traits (RPITIT)
- No `.unwrap()` in production code — use `?`
- No DB access from HTTP handlers — call application services only
- No HTTP types in `domain/` or `application/`
- `commit` / `rollback` called in the application layer only
- All request DTOs must have `#[serde(deny_unknown_fields)]`
- `update()` in every repository must include all mutable domain fields
