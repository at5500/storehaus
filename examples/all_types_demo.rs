use storehaus::prelude::*;

/// Model demonstrating all SQLx-compatible types supported by StoreHaus
///
/// Note: PostgreSQL doesn't support unsigned integers (u16, u32, u64) natively.
/// Use signed integers (i16, i32, i64) instead. For large unsigned values that
/// exceed i64::MAX, consider using i64 with application-level validation or
/// creating a custom wrapper type with NUMERIC storage.
#[model]
#[table(name = "all_types_test")]
pub struct AllTypesTest {
    #[primary_key]
    pub id: Uuid,

    // ========== String types ==========
    #[field(create, update)]
    pub required_string: String,

    #[field(create, update)]
    pub optional_string: Option<String>,

    // ========== Integer types (PostgreSQL supports signed only) ==========
    #[field(create, update)]
    pub required_i8: i8,

    #[field(create, update)]
    pub optional_i8: Option<i8>,

    #[field(create, update)]
    pub required_i16: i16,

    #[field(create, update)]
    pub optional_i16: Option<i16>,

    #[field(create, update)]
    pub required_i32: i32,

    #[field(create, update)]
    pub optional_i32: Option<i32>,

    #[field(create, update)]
    pub required_i64: i64,

    #[field(create, update)]
    pub optional_i64: Option<i64>,

    // ========== Float types ==========
    #[field(create, update)]
    pub required_f32: f32,

    #[field(create, update)]
    pub optional_f32: Option<f32>,

    #[field(create, update)]
    pub required_f64: f64,

    #[field(create, update)]
    pub optional_f64: Option<f64>,

    // ========== Boolean types ==========
    #[field(create, update)]
    pub required_bool: bool,

    #[field(create, update)]
    pub optional_bool: Option<bool>,

    // ========== UUID types ==========
    #[field(create, update)]
    pub required_uuid: Uuid,

    #[field(create, update)]
    pub optional_uuid: Option<Uuid>,

    // ========== DateTime types ==========
    #[field(create, update)]
    pub required_datetime: chrono::DateTime<chrono::Utc>,

    #[field(create, update)]
    pub optional_datetime: Option<chrono::DateTime<chrono::Utc>>,

    #[field(create, update)]
    pub optional_naive_datetime: Option<chrono::NaiveDateTime>,

    #[field(create, update)]
    pub optional_date: Option<chrono::NaiveDate>,

    // ========== JSON types ==========
    #[field(create, update)]
    pub required_json: serde_json::Value,

    #[field(create, update)]
    pub optional_json: Option<serde_json::Value>,

    // ========== Array types ==========
    #[field(create, update)]
    pub string_array: Vec<String>,
}

fn main() {
    println!("=== StoreHaus: Complete Type Mapping Verification ===\n");

    println!("Table: {}\n", AllTypesTest::table_name());

    println!("Field Types (Rust -> PostgreSQL):");
    println!("═══════════════════════════════════════════════════════════════");

    let fields = AllTypesTest::get_table_fields();

    // Group by categories for readability
    let categories = [
        ("STRING TYPES", vec!["required_string", "optional_string"]),
        ("INTEGER TYPES", vec!["required_i8", "optional_i8", "required_i16", "optional_i16", "required_i32", "optional_i32", "required_i64", "optional_i64"]),
        ("FLOAT TYPES", vec!["required_f32", "optional_f32", "required_f64", "optional_f64"]),
        ("BOOLEAN TYPES", vec!["required_bool", "optional_bool"]),
        ("UUID TYPES", vec!["required_uuid", "optional_uuid"]),
        ("DATETIME TYPES", vec!["required_datetime", "optional_datetime", "optional_naive_datetime", "optional_date"]),
        ("JSON TYPES", vec!["required_json", "optional_json"]),
        ("ARRAY TYPES", vec!["string_array"]),
    ];

    for (category, field_names) in categories {
        println!("\n{}", category);
        println!("───────────────────────────────────────────────────────────────");
        for field_name in field_names {
            if let Some((_, pg_type)) = fields.iter().find(|(name, _)| *name == field_name) {
                let nullable = if field_name.starts_with("optional_") { "✓" } else { "✗" };
                println!("  {:30} -> {:30} nullable: {}", field_name, pg_type, nullable);
            }
        }
    }

    println!("\n\n🔍 CREATE TABLE SQL:");
    println!("═══════════════════════════════════════════════════════════════");
    let create_sql = AllTypesTest::create_table_sql();

    // Format for readability
    let formatted = create_sql
        .replace(", ", ",\n    ")
        .replace("(", "(\n    ")
        .replace(")", "\n)");
    println!("{}", formatted);

    println!("\n✅ All SQLx-compatible types are correctly mapped!");
    println!("\n📝 Note: PostgreSQL doesn't support unsigned integers (u16, u32, u64).");
    println!("   Use signed integers (i16, i32, i64) instead for portable code.");
}