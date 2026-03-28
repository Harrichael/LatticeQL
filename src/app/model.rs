/// A db-agnostic representation of a table/entity in the schema.
pub struct SchemaNode {
    pub name: String,
    pub columns: Vec<ColumnDef>,
}

/// A column within a SchemaNode.
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub is_primary_key: bool,
    pub nullable: bool,
}

impl SchemaNode {
    /// Build a SchemaNode from the db-layer TableInfo.
    pub fn from_table_info(info: &crate::db::TableInfo) -> Self {
        Self {
            name: info.name.clone(),
            columns: info.columns.iter().map(|c| ColumnDef {
                name: c.name.clone(),
                data_type: c.data_type.clone(),
                is_primary_key: c.is_primary_key,
                nullable: c.nullable,
            }).collect(),
        }
    }
}
