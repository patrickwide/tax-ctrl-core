# Mapping `etims-vscu-wrapper` types to SeaORM SQL column types

`etims-vscu-wrapper` (added as a dependency in `Cargo.toml`) defines the
Rust structs used to talk to KRA's ETIMS VSCU/OSCU API — request DTOs
under `etims_vscu_wrapper::data::request` (e.g.
`BranchCustomerInformation`, `SalesInformation`) and response DTOs under
`etims_vscu_wrapper::data::response` (e.g. `ApiResponse`). None of these
are SeaORM entities themselves; they're wire-format types for the ETIMS
API. This doc is the reference for the day a feature needs to persist
some of that data (a branch cache, an outbox of submitted sales, an
audit log of API responses, etc.) and a migration has to pick SQL column
types for it.

The mechanical part of this mapping — for the primitive field types —
is also encoded as a compiler-checked trait,
[`EtimsColumnType`](../src/persistence/etims_types.rs), rather than kept
only as prose here. Use it to look up a type's `ColumnType` instead of
copying values out of this table by hand:

```rust
use tax_ctrl_core::persistence::EtimsColumnType;

assert_eq!(String::sea_orm_column_type(), sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::None));
```

## Field survey

The table below is the result of surveying every field across
`etims_vscu_wrapper::data::request::*` and `::data::response::*` (18
files, matching the `data/request` and `data/response` layout in that
crate).

| Rust type | Frequency | SeaORM `ColumnType` | Notes |
| --- | --- | --- | --- |
| `String` | 172 | `String(StringLen::None)` | The dominant type. KRA enforces a per-field max length at the API layer (`etims_vscu_wrapper::utils::validate_fields::validate_field`), not on the Rust type — give a real bound (`ColumnDef::new(col).string_len(n)`) per field where it matters, using that same `max_length`, rather than leaving every string unbounded. |
| `Option<String>` | 217 | same as `String`, nullable | The majority shape — most request fields are optional (e.g. `adrs`, `telNo`, `remark`). |
| `i64` | 74 | `BigInteger` | Used for both true identifiers/counts (`BranchInsuranceInformation::isrc_rt`) and, inconsistently across the wrapper, some monetary amounts (`SalesInformationItem::amount`, `unit_price` are `i64`, not `f64` — check the specific struct rather than assuming). |
| `Option<i64>` | 32 | `BigInteger`, nullable | |
| `f64` | 66 | `Double` (see caveat below) | Monetary amounts, prices, and quantities in `purchase_information`, `item_information`, `stock_information` (e.g. `PurchaseInformationItem::unit_price`, `taxable_amount`). |
| `Option<f64>` | 81 | `Double`, nullable | |
| `bool` | 0 | `Boolean` | Not used by the wrapper today — its `useYn`-style flags are `"Y"`/`"N"` strings, matching the ported Python source. Mapped for completeness since hand-written entities elsewhere in this crate may still need a real `bool` field. |
| `serde_json::Value` (1 occurrence, response side) | 1 | `Json` | Would require adding `serde_json` as a direct dependency of this crate to use in an entity `Model`; not added speculatively — add it if/when a concrete entity actually needs this field. |
| Nested struct (e.g. `TaxClassAmounts`, `UserRef`, `SalesInformationReceipt`, `InitInfoDataVscu`) | ~10 occurrences | **not mechanically mapped** | See below. |
| `Vec<T>` of a nested struct (e.g. `SalesInformation::item_list: Vec<SalesInformationItem>`, `PurchaseInformation::item_list: Vec<PurchaseListLineItem>`) | ~14 occurrences | **not mechanically mapped** | See below. |
| `Vec<T>` of a primitive (hypothetical, none currently in the wrapper) | 0 | `Json` | Covered by `EtimsColumnType`'s blanket `Vec<T>` impl for completeness, but nothing in the wrapper currently needs it. |

Frequency counts are field occurrences across every request/response
struct, not distinct field names.

## Why nested structs and `Vec<Struct>` aren't auto-mapped

A `String`, `i64`, or `f64` field maps to exactly one column, unambiguously.
A nested struct or a `Vec` of one doesn't — there are three legitimate
ways to persist it, and which one is right depends on the table being
designed, not on the Rust type:

1. **Flatten into the owning table's columns** (good for a 1:1 struct
   like `UserRef` inside `AuditInfo` — `registered_by_id`,
   `registered_by_name`, etc. as plain columns).
2. **A foreign key to a separate table** (good for `Vec<SalesInformationItem>`
   in a normalized schema — one row per line item, a `sales_id` FK back
   to the parent).
3. **A single `Json`/`JsonBinary` column holding the raw substructure**
   (pragmatic for tables that store an API payload mostly verbatim, e.g.
   an audit/log table of exactly what was sent to or received from ETIMS).

`EtimsColumnType` intentionally only covers the primitive fields (option
1's leaf columns, essentially) so it can't silently pick one of these
for you — `Vec<SalesInformationItem>` doesn't implement
`EtimsColumnType` because `SalesInformationItem` doesn't, and that's by
design, not an oversight.

## Dates

ETIMS date/time fields (e.g. `ApiResponse::result_dt`, KRA's
`yyyyMMddHHmmss`-style timestamps) are typed `String` in the wrapper,
matching the wire format exactly — there's no `chrono` dependency in
`etims-vscu-wrapper`. At the Rust-type level the direct match is still
`String`/`TEXT`. If a table needs to filter or sort by one of these
dates, it's usually worth parsing it into a real `DateTime`/`Timestamp`
column at the point the entity is written to, rather than persisting the
raw string and losing queryability — but that's a per-field judgment
call for whoever designs that migration, same as the nested-struct cases
above.

## Worked example

`BranchInsuranceInformation` is no longer just a hypothetical — it's the
first real table built on this mapping, following the pattern in
[`tutorial-todo-list.md`](tutorial-todo-list.md):

- Migration: [`m20260826_000001_create_branch_insurance_table`](../src/persistence/migrations/m20260826_000001_create_branch_insurance_table.rs)
- Entity: [`entities::branch_insurance`](../src/persistence/entities/branch_insurance.rs)
- Repository: [`BranchInsuranceRepository`](../src/persistence/branch_insurance_repository.rs)

The migration's `string_len(10)`, `string_len(100)`, `string_len(1)`,
`string_len(60)`, `string_len(20)` bounds come directly from the
`validate_field(&mut errors, "isrccCd", ..., 10)`-style calls in
`BranchInsuranceInformation::validate` — that's the per-field-max-length
source of truth mentioned in the table above. `isrcRt` is `big_integer`,
matching `i64`'s `EtimsColumnType` mapping.

`BranchInsuranceRepository::create` takes a
`&BranchInsuranceInformation` directly and calls its `Validate` impl
before inserting — so a record that wouldn't pass ETIMS's own
field-length rules is rejected before it reaches the database, rather
than the two validations silently drifting apart over time.
