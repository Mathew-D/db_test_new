/*
Made by: Mathew Dusome
April 2 2026
Turso (libSQL) database config + schema for Rust

================================
CUSTOMIZE YOUR DATABASE SCHEMA:
================================
1. Modify the DatabaseTable struct below.
    This struct is app-owned and safe to customize without touching database.rs.
   - Add/remove fields to match your table columns
   - Use appropriate Rust types: i32 for INTEGER, String for TEXT, bool for BOOLEAN, f64 for REAL
   - Keep id: i32 (0 for INSERT means auto-generate, populated with actual ID for SELECT)
   - Use serde attributes for custom naming if needed

2. Create your table in Turso (via CLI or SQL):

   Using Turso CLI:
     turso db shell my-db
     CREATE TABLE my_table (
       id INTEGER PRIMARY KEY AUTOINCREMENT,
       column1 TEXT NOT NULL,
       column2 INTEGER,
       ...
     );

3. Column type mapping:
   - INTEGER -> i32, i64
   - TEXT -> String
   - REAL -> f64
   - BOOLEAN -> bool
   - NUMERIC -> f64 or String
*/

use serde::{Deserialize, Serialize};

// Helper function for serde to skip serializing id when it's 0
fn is_zero(num: &i32) -> bool {
    *num == 0
}

// Please replace the libsql:// from the URL with https:
pub const TURSO_URL: &str = "https://testing-mathew-d.aws-us-east-2.turso.io";
pub const TURSO_AUTH_TOKEN: &str = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhIjoicnciLCJpYXQiOjE3NjYwMDM3MzQsImlkIjoiYWJmN2VjMmQtNjI4Yy00NjQ1LTk5YWEtYjJlN2JkYmRlZjBiIiwicmlkIjoiMTc5YjVmZjktZTFlNC00YjdjLWIxYWQtMmJhYmMwOTBjNjhiIn0.BVSKprWC8aRNmi8oh6O8zHM7GsdF01d5miK3a95-UsljE6DtLk4U_iqJfHJkKA2CmvaBS706pes6I2RSUsBoCw";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseTable {
    #[serde(default, skip_serializing_if = "is_zero")]
    pub id: i32,
    pub text: String,
}