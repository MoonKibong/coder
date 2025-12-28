use crate::domain::{
    ColumnIntent, CrudOperation, DataType, GenerateInput, SchemaColumn, SchemaInput, SpringIntent,
    SpringOptions, UiType, to_pascal_case,
};
use anyhow::{anyhow, Result};

/// Service for normalizing input to SpringIntent DSL
pub struct SpringNormalizerService;

impl SpringNormalizerService {
    /// Normalize any input type to SpringIntent
    pub fn normalize(input: &GenerateInput, package_base: &str) -> Result<SpringIntent> {
        match input {
            GenerateInput::DbSchema(schema) => Self::normalize_schema(schema, package_base),
            GenerateInput::QuerySample(query) => {
                // For query samples, extract table info and treat as schema
                let table_name = Self::extract_table_from_query(&query.query)?;
                let columns = Self::extract_columns_from_query(&query.query)?;

                let schema = SchemaInput {
                    table: table_name,
                    schema: None,
                    columns,
                    primary_keys: vec![],
                    foreign_keys: vec![],
                };
                Self::normalize_schema(&schema, package_base)
            }
            GenerateInput::NaturalLanguage(nl) => {
                // For natural language, create a basic intent
                // The LLM will need to fill in details
                let entity_name = Self::infer_entity_name(&nl.description);
                let table_name = format!("TB_{}", entity_name.to_uppercase());

                Ok(SpringIntent::new(entity_name, table_name, package_base))
            }
        }
    }

    /// Normalize database schema to SpringIntent
    pub fn normalize_schema(input: &SchemaInput, package_base: &str) -> Result<SpringIntent> {
        // Derive entity name from table name
        let entity_name = Self::table_to_entity_name(&input.table);
        let table_name = input.table.clone();

        // Convert schema columns to column intents
        let columns: Vec<ColumnIntent> = input
            .columns
            .iter()
            .map(|c| Self::schema_column_to_intent(c, &input.primary_keys))
            .collect();

        // Determine CRUD operations (default: all)
        let crud_operations = vec![
            CrudOperation::Create,
            CrudOperation::Read,
            CrudOperation::ReadList,
            CrudOperation::Update,
            CrudOperation::Delete,
        ];

        Ok(SpringIntent {
            entity_name,
            table_name,
            package_base: package_base.to_string(),
            columns,
            crud_operations,
            options: SpringOptions::default(),
        })
    }

    /// Convert table name to entity name (e.g., "TB_MEMBER" -> "Member")
    fn table_to_entity_name(table: &str) -> String {
        let clean = table
            .strip_prefix("TB_")
            .or_else(|| table.strip_prefix("TBL_"))
            .or_else(|| table.strip_prefix("T_"))
            .unwrap_or(table);

        to_pascal_case(clean)
    }

    /// Convert a schema column to column intent
    fn schema_column_to_intent(col: &SchemaColumn, primary_keys: &[String]) -> ColumnIntent {
        let is_pk = col.pk || primary_keys.contains(&col.name);
        let label = Self::infer_label(&col.name, col.comment.as_deref());
        let (ui_type, data_type) = Self::infer_types(&col.column_type, is_pk);

        let mut intent = ColumnIntent::new(&col.name, label)
            .with_ui_type(ui_type)
            .with_data_type(data_type);

        if is_pk {
            intent = intent.primary_key();
        } else if !col.nullable {
            intent = intent.required();
        }

        // Extract max length from VARCHAR(n)
        if let Some(len) = Self::extract_varchar_length(&col.column_type) {
            intent = intent.with_max_length(len);
        }

        intent
    }

    /// Infer UI type and data type from database column type
    fn infer_types(db_type: &str, is_pk: bool) -> (UiType, DataType) {
        if is_pk {
            return (UiType::Hidden, DataType::Integer);
        }

        let upper = db_type.to_uppercase();

        if upper.starts_with("VARCHAR") || upper.starts_with("CHAR") || upper == "NVARCHAR" {
            return (UiType::Input, DataType::String);
        }

        if upper.starts_with("TEXT") || upper.starts_with("CLOB") || upper == "LONGTEXT" {
            return (UiType::TextArea, DataType::Text);
        }

        if upper == "DATE" {
            return (UiType::DatePicker, DataType::Date);
        }

        if upper.starts_with("DATETIME") || upper.starts_with("TIMESTAMP") {
            return (UiType::DateTimePicker, DataType::DateTime);
        }

        if upper == "BOOLEAN" || upper == "BOOL" || upper == "BIT" {
            return (UiType::Checkbox, DataType::Boolean);
        }

        if upper.starts_with("INT") || upper == "BIGINT" || upper == "SMALLINT" || upper == "TINYINT" {
            return (UiType::Number, DataType::Integer);
        }

        if upper.starts_with("DECIMAL") || upper.starts_with("NUMERIC") || upper == "FLOAT" || upper == "DOUBLE" {
            return (UiType::Number, DataType::Decimal);
        }

        (UiType::Input, DataType::String)
    }

    /// Extract length from VARCHAR(n)
    fn extract_varchar_length(db_type: &str) -> Option<u32> {
        let upper = db_type.to_uppercase();
        if let Some(start) = upper.find('(') {
            if let Some(end) = upper.find(')') {
                if let Ok(len) = upper[start + 1..end].trim().parse() {
                    return Some(len);
                }
            }
        }
        None
    }

    /// Infer display label from column name
    fn infer_label(name: &str, comment: Option<&str>) -> String {
        if let Some(c) = comment {
            if !c.is_empty() {
                return c.to_string();
            }
        }

        // Common Korean mappings
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            "id" => "ID".to_string(),
            "name" | "nm" => "이름".to_string(),
            "member_id" | "user_id" => "회원ID".to_string(),
            "member_name" | "user_name" => "회원명".to_string(),
            "email" => "이메일".to_string(),
            "phone" | "tel" | "phone_no" => "전화번호".to_string(),
            "address" | "addr" => "주소".to_string(),
            "created_at" | "reg_date" | "reg_dt" => "등록일".to_string(),
            "updated_at" | "mod_date" | "mod_dt" => "수정일".to_string(),
            "created_by" | "reg_id" => "등록자".to_string(),
            "updated_by" | "mod_id" => "수정자".to_string(),
            "status" | "state" => "상태".to_string(),
            "type" | "kind" => "유형".to_string(),
            "description" | "desc" => "설명".to_string(),
            "remarks" | "note" => "비고".to_string(),
            "title" => "제목".to_string(),
            "content" => "내용".to_string(),
            "amount" | "amt" => "금액".to_string(),
            "price" => "가격".to_string(),
            "quantity" | "qty" => "수량".to_string(),
            "use_yn" | "is_active" => "사용여부".to_string(),
            "del_yn" | "is_deleted" => "삭제여부".to_string(),
            _ => Self::humanize_column_name(name),
        }
    }

    /// Convert snake_case to human-readable format
    fn humanize_column_name(name: &str) -> String {
        name.replace('_', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().to_string() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Extract table name from query
    fn extract_table_from_query(query: &str) -> Result<String> {
        let upper = query.to_uppercase();

        if let Some(from_pos) = upper.find(" FROM ") {
            let after_from = &query[from_pos + 6..];
            let table_part = after_from
                .split_whitespace()
                .next()
                .ok_or_else(|| anyhow!("Could not extract table name from query"))?;

            let table_name = table_part.split('.').last().unwrap_or(table_part);
            let clean_name = table_name.trim_matches(|c| c == '"' || c == '\'' || c == '`');

            return Ok(clean_name.to_string());
        }

        Err(anyhow!("Could not find FROM clause in query"))
    }

    /// Extract columns from query
    fn extract_columns_from_query(query: &str) -> Result<Vec<SchemaColumn>> {
        let upper = query.to_uppercase();

        let select_pos = upper.find("SELECT").ok_or_else(|| anyhow!("No SELECT found"))?;
        let from_pos = upper.find(" FROM ").ok_or_else(|| anyhow!("No FROM found"))?;

        let select_clause = &query[select_pos + 6..from_pos].trim();

        if select_clause.trim() == "*" {
            return Ok(vec![]);
        }

        let columns: Vec<SchemaColumn> = select_clause
            .split(',')
            .filter_map(|col| {
                let col = col.trim();
                if col.is_empty() {
                    return None;
                }

                let parts: Vec<&str> = col.split_whitespace().collect();
                let name = if parts.len() >= 3 && parts[parts.len() - 2].to_uppercase() == "AS" {
                    parts.last().unwrap().trim_matches(|c| c == '"' || c == '\'')
                } else {
                    parts[0].split('.').last().unwrap_or(parts[0])
                };

                Some(SchemaColumn::new(name, "VARCHAR(255)"))
            })
            .collect();

        Ok(columns)
    }

    /// Infer entity name from description
    fn infer_entity_name(description: &str) -> String {
        let lower = description.to_lowercase();

        if lower.contains("회원") || lower.contains("member") || lower.contains("user") {
            return "Member".to_string();
        }
        if lower.contains("주문") || lower.contains("order") {
            return "Order".to_string();
        }
        if lower.contains("상품") || lower.contains("product") {
            return "Product".to_string();
        }
        if lower.contains("게시판") || lower.contains("board") || lower.contains("post") {
            return "Board".to_string();
        }

        "Entity".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SchemaColumn;

    #[test]
    fn test_table_to_entity_name() {
        assert_eq!(SpringNormalizerService::table_to_entity_name("TB_MEMBER"), "Member");
        assert_eq!(SpringNormalizerService::table_to_entity_name("TBL_ORDER_DETAIL"), "OrderDetail");
        assert_eq!(SpringNormalizerService::table_to_entity_name("T_USER"), "User");
        assert_eq!(SpringNormalizerService::table_to_entity_name("PRODUCT"), "Product");
    }

    #[test]
    fn test_normalize_schema() {
        let schema = SchemaInput::new("TB_MEMBER")
            .with_column(SchemaColumn::new("MEMBER_ID", "BIGINT").primary_key())
            .with_column(SchemaColumn::new("MEMBER_NAME", "VARCHAR(100)").not_null())
            .with_column(SchemaColumn::new("EMAIL", "VARCHAR(255)"))
            .with_column(SchemaColumn::new("CREATED_AT", "TIMESTAMP"));

        let intent = SpringNormalizerService::normalize_schema(&schema, "com.company.project").unwrap();

        assert_eq!(intent.entity_name, "Member");
        assert_eq!(intent.table_name, "TB_MEMBER");
        assert_eq!(intent.package_base, "com.company.project");
        assert_eq!(intent.columns.len(), 4);
        assert_eq!(intent.crud_operations.len(), 5);
    }

    #[test]
    fn test_infer_label() {
        assert_eq!(SpringNormalizerService::infer_label("email", None), "이메일");
        assert_eq!(SpringNormalizerService::infer_label("created_at", None), "등록일");
        assert_eq!(SpringNormalizerService::infer_label("member_name", None), "회원명");
    }
}
