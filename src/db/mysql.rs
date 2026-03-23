use super::{ColumnInfo, Database, ForeignKey, Row, TableInfo, Value};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{Column, MySqlPool, Row as SqlxRow, TypeInfo};
use std::collections::HashMap;

pub struct MysqlDb {
    pool: MySqlPool,
}

impl MysqlDb {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = MySqlPool::connect(url).await?;
        Ok(Self { pool })
    }
}

/// Try to decode a column as String; if the DB returns VARBINARY (common in
/// some MySQL collations for information_schema), fall back to UTF-8 bytes.
fn get_string(row: &sqlx::mysql::MySqlRow, col: &str) -> String {
    use sqlx::Row as _;
    if let Ok(s) = row.try_get::<String, _>(col) {
        return s;
    }
    if let Ok(b) = row.try_get::<Vec<u8>, _>(col) {
        return String::from_utf8_lossy(&b).into_owned();
    }
    String::new()
}

fn get_string_idx(row: &sqlx::mysql::MySqlRow, idx: usize) -> String {
    use sqlx::Row as _;
    if let Ok(s) = row.try_get::<String, _>(idx) {
        return s;
    }
    if let Ok(b) = row.try_get::<Vec<u8>, _>(idx) {
        return String::from_utf8_lossy(&b).into_owned();
    }
    String::new()
}

#[async_trait]
impl Database for MysqlDb {
    async fn list_tables(&self) -> Result<Vec<String>> {
        let rows = sqlx::query("SHOW TABLES").fetch_all(&self.pool).await?;
        Ok(rows.iter().map(|r| get_string_idx(r, 0)).collect())
    }

    async fn describe_table(&self, table: &str) -> Result<TableInfo> {
        // Get columns
        let col_sql = format!(
            "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_KEY \
             FROM information_schema.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = '{}' \
             ORDER BY ORDINAL_POSITION",
            table.replace('\'', "''")
        );
        let col_rows = sqlx::query(&col_sql).fetch_all(&self.pool).await?;
        let mut columns = Vec::new();
        for row in &col_rows {
            let name = get_string(row, "COLUMN_NAME");
            let data_type = get_string(row, "DATA_TYPE");
            let is_nullable = get_string(row, "IS_NULLABLE");
            let col_key = get_string(row, "COLUMN_KEY");
            columns.push(ColumnInfo {
                name,
                data_type,
                nullable: is_nullable == "YES",
                is_primary_key: col_key == "PRI",
            });
        }

        // Get foreign keys
        let fk_sql = format!(
            "SELECT COLUMN_NAME, REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME \
             FROM information_schema.KEY_COLUMN_USAGE \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = '{}' \
             AND REFERENCED_TABLE_NAME IS NOT NULL",
            table.replace('\'', "''")
        );
        let fk_rows = sqlx::query(&fk_sql).fetch_all(&self.pool).await?;
        let mut foreign_keys = Vec::new();
        for row in &fk_rows {
            let from_column = get_string(row, "COLUMN_NAME");
            let to_table = get_string(row, "REFERENCED_TABLE_NAME");
            let to_column = get_string(row, "REFERENCED_COLUMN_NAME");
            foreign_keys.push(ForeignKey {
                from_column,
                to_table,
                to_column,
            });
        }

        Ok(TableInfo {
            name: table.to_string(),
            columns,
            foreign_keys,
        })
    }

    async fn describe_all_tables(&self, tables: &[String]) -> Result<Vec<TableInfo>> {
        if tables.is_empty() {
            return Ok(vec![]);
        }

        // Fetch ALL columns for all tables in one query.
        let col_rows = sqlx::query(
            "SELECT TABLE_NAME, COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_KEY \
             FROM information_schema.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() \
             ORDER BY TABLE_NAME, ORDINAL_POSITION",
        )
        .fetch_all(&self.pool)
        .await?;

        // Fetch ALL foreign keys in one query.
        let fk_rows = sqlx::query(
            "SELECT TABLE_NAME, COLUMN_NAME, REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME \
             FROM information_schema.KEY_COLUMN_USAGE \
             WHERE TABLE_SCHEMA = DATABASE() \
             AND REFERENCED_TABLE_NAME IS NOT NULL",
        )
        .fetch_all(&self.pool)
        .await?;

        // Build maps keyed by table name.
        let mut col_map: HashMap<String, Vec<ColumnInfo>> = HashMap::new();
        for row in &col_rows {
            let tname = get_string(row, "TABLE_NAME");
            let name = get_string(row, "COLUMN_NAME");
            let data_type = get_string(row, "DATA_TYPE");
            let is_nullable = get_string(row, "IS_NULLABLE");
            let col_key = get_string(row, "COLUMN_KEY");
            col_map.entry(tname).or_default().push(ColumnInfo {
                name,
                data_type,
                nullable: is_nullable == "YES",
                is_primary_key: col_key == "PRI",
            });
        }

        let mut fk_map: HashMap<String, Vec<ForeignKey>> = HashMap::new();
        for row in &fk_rows {
            let tname = get_string(row, "TABLE_NAME");
            let from_column = get_string(row, "COLUMN_NAME");
            let to_table = get_string(row, "REFERENCED_TABLE_NAME");
            let to_column = get_string(row, "REFERENCED_COLUMN_NAME");
            fk_map.entry(tname).or_default().push(ForeignKey {
                from_column,
                to_table,
                to_column,
            });
        }

        Ok(tables
            .iter()
            .map(|t| TableInfo {
                name: t.clone(),
                columns: col_map.remove(t).unwrap_or_default(),
                foreign_keys: fk_map.remove(t).unwrap_or_default(),
            })
            .collect())
    }

    async fn query(&self, sql: &str) -> Result<Vec<Row>> {
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        let mut result = Vec::new();
        for row in &rows {
            let mut map = Row::new();
            for col in row.columns() {
                let name = col.name().to_string();
                let type_info = col.type_info();
                let val = decode_mysql_value(row, col.ordinal(), type_info.name());
                map.insert(name, val);
            }
            result.push(map);
        }
        Ok(result)
    }
}

fn decode_mysql_value(
    row: &sqlx::mysql::MySqlRow,
    idx: usize,
    type_name: &str,
) -> Value {
    use sqlx::Row as _;
    let upper = type_name.to_uppercase();
    if upper.contains("INT") || upper.contains("BIT") || upper.contains("YEAR") {
        match row.try_get::<i64, _>(idx) {
            Ok(v) => Value::Integer(v),
            Err(_) => Value::Null,
        }
    } else if upper.contains("FLOAT")
        || upper.contains("DOUBLE")
        || upper.contains("DECIMAL")
        || upper.contains("NUMERIC")
    {
        match row.try_get::<f64, _>(idx) {
            Ok(v) => Value::Float(v),
            Err(_) => Value::Null,
        }
    } else if upper.contains("BLOB") || upper.contains("BINARY") {
        // Try String first (covers VARBINARY used for text in some collations)
        match row.try_get::<String, _>(idx) {
            Ok(v) => Value::Text(v),
            Err(_) => match row.try_get::<Vec<u8>, _>(idx) {
                Ok(v) => match String::from_utf8(v.clone()) {
                    Ok(s) => Value::Text(s),
                    Err(_) => Value::Bytes(v),
                },
                Err(_) => Value::Null,
            },
        }
    } else {
        match row.try_get::<String, _>(idx) {
            Ok(v) => Value::Text(v),
            Err(_) => Value::Null,
        }
    }
}
