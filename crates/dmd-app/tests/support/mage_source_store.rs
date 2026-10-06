//! Full typed rows for package/replay refusal checks; read-only SQL.
use std::collections::BTreeMap;

// Retain each storage type and value. SQL quote(TEXT) truncates at embedded NUL;
// hex(CAST(... AS BLOB)) does not. SQLite's 26-digit alternate-form REAL text
// preserves a round-trippable numeric value. Include every table and column.
pub(super) async fn typed_rows(pool: &sqlx::SqlitePool) -> BTreeMap<String, Vec<Vec<String>>> {
    use sqlx::Row;
    let names: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap();
    let mut result = BTreeMap::new();
    for name in names {
        let columns: Vec<String> =
            sqlx::query_scalar("SELECT name FROM pragma_table_info(?) ORDER BY cid")
                .bind(&name)
                .fetch_all(pool)
                .await
                .unwrap();
        let quote = |value: &str| format!("\"{}\"", value.replace('"', "\"\""));
        let statement = format!(
            "SELECT {} FROM {}",
            columns
                .iter()
                .map(|column| {
                    let quoted = quote(column);
                    format!("json_array(typeof({quoted}),CASE typeof({quoted}) WHEN 'real' THEN printf('%!.26g',{quoted}) ELSE hex(CAST({quoted} AS BLOB)) END)")
                })
                .collect::<Vec<_>>()
                .join(","),
            quote(&name)
        );
        let mut rows = sqlx::query(&statement)
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(|row| {
                (0..columns.len())
                    .map(|i| row.get::<String, _>(i))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        rows.sort();
        result.insert(name, rows);
    }
    result
}
