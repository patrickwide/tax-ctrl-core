//! SeaORM column-type mapping for `etims_vscu_wrapper`'s data types.
//!
//! `etims_vscu_wrapper::data::{request, response}` defines the Rust
//! structs used to talk to KRA's ETIMS VSCU/OSCU API (e.g.
//! [`BranchCustomerInformation`], [`SalesInformation`],
//! [`ApiResponse`]). None of those structs are SeaORM entities — they're
//! wire-format DTOs — but once a feature needs to persist ETIMS data
//! (e.g. an outbox/audit table of submitted sales, or cached branch
//! records), a migration for that table needs to know which SQL column
//! type corresponds to each Rust field type it's built from.
//!
//! [`EtimsColumnType`] is that mapping, expressed as a small trait
//! instead of a lookup table so it's checked by the compiler rather than
//! kept in sync by hand: `String::sea_orm_column_type()` and
//! `<Option<i64>>::sea_orm_column_type()` are guaranteed to compile only
//! for types this module has actually mapped.
//!
//! [`BranchCustomerInformation`]: etims_vscu_wrapper::data::request::BranchCustomerInformation
//! [`SalesInformation`]: etims_vscu_wrapper::data::request::SalesInformation
//! [`ApiResponse`]: etims_vscu_wrapper::data::response::ApiResponse
//!
//! Only the primitive field types that actually appear across
//! `etims_vscu_wrapper`'s request/response structs are covered — plain
//! `String`, `i64`, `f64`, their `Option<_>` forms, plus `bool` (not
//! currently used by the wrapper, included for completeness) and `Vec<_>`
//! of those primitives. Composite fields (nested structs like
//! `TaxClassAmounts` or `UserRef`, and `Vec<_>` of them, like
//! `SalesInformation::item_list: Vec<SalesInformationItem>`) are
//! deliberately **not** given a mechanical mapping here — whether a
//! nested struct becomes flattened columns on the same table, a foreign
//! key to a child table, or a JSON blob is a schema design decision for
//! whoever writes that migration, not something this trait should decide
//! for them. See `docs/etims-type-mapping.md` for the full reference
//! table and the reasoning behind each choice.
use sea_orm::sea_query::{ColumnType, StringLen};

/// Maps a Rust type used by `etims_vscu_wrapper`'s data types onto the
/// [`ColumnType`] a SeaORM migration should declare for it.
///
/// Implemented for the primitive types that appear as struct fields in
/// `etims_vscu_wrapper::data::{request, response}` — see the module docs
/// for which composite types are intentionally left out.
pub trait EtimsColumnType {
    /// The SeaORM/`sea_query` column type a migration should use for
    /// this Rust type.
    fn sea_orm_column_type() -> ColumnType;

    /// Whether a column of this Rust type should be declared nullable.
    /// `false` for every type here except the blanket `Option<T>` impl,
    /// which overrides it to `true`.
    fn is_nullable() -> bool {
        false
    }
}

// `String` is by far the most common field type in the wrapper's data
// module (e.g. `BranchCustomerInformation::cust_no`,
// `ApiResponse::result_dt`). KRA enforces a per-field maximum length at
// the API layer (see `etims_vscu_wrapper::utils::validate_fields`), not
// on the Rust type itself, so there's no single correct bound to bake in
// here — `StringLen::None` (an unbounded VARCHAR/TEXT) is the safe
// default. A migration for a specific table should give the real,
// field-specific length from `validate_fields` where that limit matters,
// e.g. `ColumnDef::new(Column::CustNo).string_len(9)` for `custNo`.
impl EtimsColumnType for String {
    fn sea_orm_column_type() -> ColumnType {
        ColumnType::String(StringLen::None)
    }
}

// `i64` is used both for numeric identifiers/counts (e.g.
// `BranchInsuranceInformation::isrc_rt`) and quantities. `BigInteger` is
// the direct match for Rust's 64-bit integer and avoids the silent
// truncation risk of mapping down to `Integer` (32-bit).
impl EtimsColumnType for i64 {
    fn sea_orm_column_type() -> ColumnType {
        ColumnType::BigInteger
    }
}

// `f64` is used for monetary amounts, prices and quantities in
// `data::request::purchase_information`, `item_information`, and
// `stock_information` (e.g. `PurchaseInformationItem::unit_price`).
// `Double` is the direct type-level match. Note for anyone writing a
// real migration off this: floating point is a poor fit for money, so
// once a specific ETIMS-backed table is being designed, prefer
// `ColumnType::Decimal(Some((p, s)))` (paired with `rust_decimal::Decimal`
// on the entity side) over persisting `f64` verbatim — see
// `docs/etims-type-mapping.md`.
impl EtimsColumnType for f64 {
    fn sea_orm_column_type() -> ColumnType {
        ColumnType::Double
    }
}

// Not currently produced by `etims_vscu_wrapper` (its data types have no
// `bool` fields — `useYn`/flags travel as `"Y"`/`"N"` strings), but
// mapped here for completeness since it's a type SeaORM entities commonly
// need.
impl EtimsColumnType for bool {
    fn sea_orm_column_type() -> ColumnType {
        ColumnType::Boolean
    }
}

// Every optional field in the wrapper (the majority of them — e.g.
// `BranchCustomerInformation::adrs: Option<String>`) maps to the same
// column type as its inner type, just nullable.
impl<T: EtimsColumnType> EtimsColumnType for Option<T> {
    fn sea_orm_column_type() -> ColumnType {
        T::sea_orm_column_type()
    }

    fn is_nullable() -> bool {
        true
    }
}

// A `Vec` of a primitive (e.g. a hypothetical `Vec<String>`) is stored as
// a single JSON column. This intentionally does NOT cover `Vec<Struct>`
// fields like `SalesInformation::item_list: Vec<SalesInformationItem>` —
// `SalesInformationItem` isn't `EtimsColumnType`, so that won't compile,
// which is the point: a repeated substructure is a one-to-many relation
// to a child table in a normalized schema, not a single column, and
// choosing to flatten it into JSON instead should be a deliberate choice
// made when that table is designed, not an automatic default here.
impl<T: EtimsColumnType> EtimsColumnType for Vec<T> {
    fn sea_orm_column_type() -> ColumnType {
        ColumnType::Json
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_maps_to_unbounded_varchar() {
        assert!(matches!(
            String::sea_orm_column_type(),
            ColumnType::String(StringLen::None)
        ));
        assert!(!String::is_nullable());
    }

    #[test]
    fn i64_maps_to_big_integer() {
        assert!(matches!(i64::sea_orm_column_type(), ColumnType::BigInteger));
        assert!(!i64::is_nullable());
    }

    #[test]
    fn f64_maps_to_double() {
        assert!(matches!(f64::sea_orm_column_type(), ColumnType::Double));
        assert!(!f64::is_nullable());
    }

    #[test]
    fn bool_maps_to_boolean() {
        assert!(matches!(bool::sea_orm_column_type(), ColumnType::Boolean));
        assert!(!bool::is_nullable());
    }

    #[test]
    fn option_is_nullable_but_keeps_inner_column_type() {
        assert!(matches!(
            <Option<String>>::sea_orm_column_type(),
            ColumnType::String(StringLen::None)
        ));
        assert!(<Option<String>>::is_nullable());

        assert!(matches!(
            <Option<i64>>::sea_orm_column_type(),
            ColumnType::BigInteger
        ));
        assert!(<Option<i64>>::is_nullable());

        assert!(matches!(
            <Option<f64>>::sea_orm_column_type(),
            ColumnType::Double
        ));
        assert!(<Option<f64>>::is_nullable());
    }

    #[test]
    fn vec_of_primitive_maps_to_json() {
        assert!(matches!(<Vec<String>>::sea_orm_column_type(), ColumnType::Json));
        assert!(matches!(<Vec<i64>>::sea_orm_column_type(), ColumnType::Json));
    }
}
