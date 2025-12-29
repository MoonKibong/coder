use crate::domain::{
    ColumnIntent, DataType, DatasetIntent, GenerateInput, GridColumnIntent,
    GridIntent, NaturalLanguageInput, QuerySampleInput, SchemaColumn, SchemaInput, ScreenType,
    UiIntent, UiType, default_actions_for_screen_type,
};
use anyhow::{anyhow, Result};

/// Service for normalizing various input types to UiIntent DSL
pub struct NormalizerService;

impl NormalizerService {
    /// Normalize any input type to UiIntent
    pub fn normalize(input: &GenerateInput) -> Result<UiIntent> {
        match input {
            GenerateInput::DbSchema(schema) => Self::normalize_schema(schema),
            GenerateInput::QuerySample(query) => Self::normalize_query(query),
            GenerateInput::NaturalLanguage(nl) => Self::normalize_natural_language(nl),
        }
    }

    /// Normalize database schema input to UiIntent
    pub fn normalize_schema(input: &SchemaInput) -> Result<UiIntent> {
        let screen_name = format!("{}_list", input.table.to_lowercase());
        let dataset_id = format!("ds_{}", input.table.to_lowercase());

        // Convert schema columns to column intents
        let columns: Vec<ColumnIntent> = input
            .columns
            .iter()
            .map(|c| Self::schema_column_to_intent(c, &input.primary_keys))
            .collect();

        // Create grid columns from visible columns (exclude hidden PKs)
        let grid_columns: Vec<GridColumnIntent> = columns
            .iter()
            .filter(|c| c.ui_type != UiType::Hidden)
            .map(|c| GridColumnIntent::new(&c.name, &c.label))
            .collect();

        // Build dataset
        let dataset = DatasetIntent::new(&dataset_id)
            .with_table(&input.table)
            .with_columns(columns);

        // Build grid
        let grid = GridIntent::new(format!("grid_{}", input.table.to_lowercase()), &dataset_id)
            .with_columns(grid_columns);

        // Build intent with default actions
        let actions = default_actions_for_screen_type(ScreenType::List);

        let intent = UiIntent::new(screen_name, ScreenType::List)
            .with_dataset(dataset)
            .with_grid(grid);

        // Add actions
        let mut intent = intent;
        for action in actions {
            intent = intent.with_action(action);
        }

        Ok(intent)
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

        // Check for specific types
        if upper.starts_with("VARCHAR") || upper.starts_with("CHAR") || upper == "NVARCHAR" {
            let len = Self::extract_varchar_length(db_type).unwrap_or(255);
            if len > 500 {
                return (UiType::TextArea, DataType::String);
            }
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

        if upper.starts_with("DECIMAL") || upper.starts_with("NUMERIC") || upper == "FLOAT" || upper == "DOUBLE" || upper == "REAL" {
            return (UiType::Number, DataType::Decimal);
        }

        if upper.starts_with("BLOB") || upper == "BINARY" || upper == "VARBINARY" {
            return (UiType::File, DataType::Binary);
        }

        // Default to input
        (UiType::Input, DataType::String)
    }

    /// Extract length from VARCHAR(n) type
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
        // If there's a comment, use it as the label
        if let Some(c) = comment {
            if !c.is_empty() {
                return c.to_string();
            }
        }

        // Common Korean mappings for typical column names
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            "id" => "ID".to_string(),
            "name" | "nm" => "이름".to_string(),
            "member_id" | "user_id" => "회원ID".to_string(),
            "member_name" | "user_name" => "회원명".to_string(),
            "email" => "이메일".to_string(),
            "phone" | "tel" | "phone_no" => "전화번호".to_string(),
            "mobile" | "mobile_no" => "휴대폰".to_string(),
            "address" | "addr" => "주소".to_string(),
            "created_at" | "reg_date" | "reg_dt" => "등록일".to_string(),
            "updated_at" | "mod_date" | "mod_dt" => "수정일".to_string(),
            "created_by" | "reg_id" => "등록자".to_string(),
            "updated_by" | "mod_id" => "수정자".to_string(),
            "status" | "state" => "상태".to_string(),
            "type" | "kind" => "유형".to_string(),
            "description" | "desc" => "설명".to_string(),
            "remarks" | "note" | "notes" => "비고".to_string(),
            "title" => "제목".to_string(),
            "content" | "contents" => "내용".to_string(),
            "amount" | "amt" => "금액".to_string(),
            "price" => "가격".to_string(),
            "quantity" | "qty" => "수량".to_string(),
            "date" | "dt" => "일자".to_string(),
            "start_date" | "from_date" => "시작일".to_string(),
            "end_date" | "to_date" => "종료일".to_string(),
            "use_yn" | "is_active" | "active" => "사용여부".to_string(),
            "del_yn" | "is_deleted" | "deleted" => "삭제여부".to_string(),
            _ => Self::humanize_column_name(name),
        }
    }

    /// Convert snake_case column name to human-readable format
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

    /// Normalize query sample input to UiIntent
    pub fn normalize_query(input: &QuerySampleInput) -> Result<UiIntent> {
        // Parse the query to extract table name and columns
        let table_name = Self::extract_table_from_query(&input.query)?;
        let screen_name = format!("{}_list", table_name.to_lowercase());
        let dataset_id = format!("ds_{}", table_name.to_lowercase());

        // If result_columns are provided, use them
        let columns: Vec<ColumnIntent> = if let Some(ref cols) = input.result_columns {
            cols.iter()
                .map(|c| {
                    let label = c.label.clone().unwrap_or_else(|| Self::infer_label(&c.name, None));
                    let (ui_type, data_type) = c
                        .column_type
                        .as_ref()
                        .map(|t| Self::infer_types(t, false))
                        .unwrap_or((UiType::Input, DataType::String));
                    ColumnIntent::new(&c.name, label)
                        .with_ui_type(ui_type)
                        .with_data_type(data_type)
                })
                .collect()
        } else {
            // Try to extract columns from SELECT clause
            Self::extract_columns_from_query(&input.query)?
        };

        let grid_columns: Vec<GridColumnIntent> = columns
            .iter()
            .filter(|c| c.ui_type != UiType::Hidden)
            .map(|c| GridColumnIntent::new(&c.name, &c.label))
            .collect();

        let dataset = DatasetIntent::new(&dataset_id)
            .with_table(&table_name)
            .with_columns(columns);

        let grid = GridIntent::new(format!("grid_{}", table_name.to_lowercase()), &dataset_id)
            .with_columns(grid_columns);

        let actions = default_actions_for_screen_type(ScreenType::List);

        let mut intent = UiIntent::new(screen_name, ScreenType::List)
            .with_dataset(dataset)
            .with_grid(grid);

        if let Some(ref desc) = input.description {
            intent = intent.with_notes(desc.clone());
        }

        for action in actions {
            intent = intent.with_action(action);
        }

        Ok(intent)
    }

    /// Extract main table name from SELECT query
    fn extract_table_from_query(query: &str) -> Result<String> {
        let upper = query.to_uppercase();

        // Find FROM clause
        if let Some(from_pos) = upper.find(" FROM ") {
            let after_from = &query[from_pos + 6..];
            let table_part = after_from
                .split_whitespace()
                .next()
                .ok_or_else(|| anyhow!("Could not extract table name from query"))?;

            // Remove schema prefix if present
            let table_name = table_part.split('.').last().unwrap_or(table_part);

            // Remove any alias or quotes
            let clean_name = table_name
                .trim_matches(|c| c == '"' || c == '\'' || c == '`' || c == '[' || c == ']');

            return Ok(clean_name.to_string());
        }

        Err(anyhow!("Could not find FROM clause in query"))
    }

    /// Extract columns from SELECT clause
    fn extract_columns_from_query(query: &str) -> Result<Vec<ColumnIntent>> {
        let upper = query.to_uppercase();

        // Find SELECT ... FROM
        let select_pos = upper.find("SELECT").ok_or_else(|| anyhow!("No SELECT found"))?;
        let from_pos = upper.find(" FROM ").ok_or_else(|| anyhow!("No FROM found"))?;

        let select_clause = &query[select_pos + 6..from_pos].trim();

        // Handle SELECT *
        if select_clause.trim() == "*" {
            return Err(anyhow!("SELECT * requires result_columns to be provided"));
        }

        // Split by comma (simple parsing, may not handle all cases)
        let columns: Vec<ColumnIntent> = select_clause
            .split(',')
            .filter_map(|col| {
                let col = col.trim();
                if col.is_empty() {
                    return None;
                }

                // Handle AS alias
                let parts: Vec<&str> = col.split_whitespace().collect();
                let name = if parts.len() >= 3 && parts[parts.len() - 2].to_uppercase() == "AS" {
                    parts.last().unwrap().trim_matches(|c| c == '"' || c == '\'' || c == '`')
                } else if parts.len() >= 2 && !["AS", "AND", "OR"].contains(&parts.last().unwrap().to_uppercase().as_str()) {
                    // Last word might be an alias without AS
                    parts.last().unwrap().trim_matches(|c| c == '"' || c == '\'' || c == '`')
                } else {
                    // No alias, use the column expression
                    let col_name = parts[0].split('.').last().unwrap_or(parts[0]);
                    col_name.trim_matches(|c| c == '"' || c == '\'' || c == '`')
                };

                let label = Self::infer_label(name, None);
                Some(ColumnIntent::new(name, label))
            })
            .collect();

        if columns.is_empty() {
            return Err(anyhow!("No columns found in SELECT clause"));
        }

        Ok(columns)
    }

    /// Normalize natural language input to UiIntent
    pub fn normalize_natural_language(input: &NaturalLanguageInput) -> Result<UiIntent> {
        // For natural language, we create a basic intent and let the LLM fill in details
        let screen_type = input
            .screen_type
            .as_ref()
            .map(|s| match s.to_lowercase().as_str() {
                "list" => ScreenType::List,
                "detail" => ScreenType::Detail,
                "popup" => ScreenType::Popup,
                "list_with_popup" | "listwithpopup" => ScreenType::ListWithPopup,
                _ => ScreenType::List,
            })
            .unwrap_or(ScreenType::List);

        let screen_name = Self::infer_screen_name_from_description(&input.description);
        let actions = default_actions_for_screen_type(screen_type);

        let mut intent = UiIntent::new(screen_name, screen_type)
            .with_notes(&input.description);

        if let Some(ref ctx) = input.context {
            intent.notes = Some(format!(
                "{}\n\nContext: {}",
                intent.notes.as_deref().unwrap_or(""),
                ctx
            ));
        }

        for action in actions {
            intent = intent.with_action(action);
        }

        Ok(intent)
    }

    /// Infer screen name from natural language description
    fn infer_screen_name_from_description(description: &str) -> String {
        let lower = description.to_lowercase();

        // Entity mappings: (keywords, entity_name)
        let entity_patterns: &[(&[&str], &str)] = &[
            // Korean entities
            (&["회원", "사용자"], "member"),
            (&["주문"], "order"),
            (&["상품", "제품"], "product"),
            (&["게시판", "게시물"], "board"),
            (&["고객"], "customer"),
            (&["직원", "사원"], "employee"),
            (&["부서"], "department"),
            (&["프로젝트"], "project"),
            (&["업무", "작업", "태스크"], "task"),
            (&["일정", "스케줄"], "schedule"),
            (&["예약"], "reservation"),
            (&["결제", "payment"], "payment"),
            (&["송장", "인보이스"], "invoice"),
            (&["재고"], "inventory"),
            (&["카테고리", "분류"], "category"),
            (&["공지사항", "공지"], "notice"),
            (&["문의", "질문"], "inquiry"),
            (&["코드", "코드관리"], "code"),
            // English entities
            (&["member", "user", "account"], "member"),
            (&["order", "purchase"], "order"),
            (&["product", "item", "goods"], "product"),
            (&["board", "post", "article"], "board"),
            (&["customer", "client"], "customer"),
            (&["employee", "staff", "worker"], "employee"),
            (&["department", "dept"], "department"),
            (&["project"], "project"),
            (&["task", "todo", "job", "work"], "task"),
            (&["schedule", "calendar", "event"], "schedule"),
            (&["reservation", "booking"], "reservation"),
            (&["payment", "transaction"], "payment"),
            (&["invoice", "bill"], "invoice"),
            (&["inventory", "stock"], "inventory"),
            (&["category"], "category"),
            (&["notice", "announcement"], "notice"),
            (&["inquiry", "question", "support"], "inquiry"),
            (&["code", "master"], "code"),
            (&["setting", "config", "preference"], "setting"),
            (&["log", "history", "audit"], "log"),
            (&["report", "statistics", "analytics"], "report"),
            (&["file", "document", "attachment"], "file"),
            (&["menu", "navigation"], "menu"),
            (&["role", "permission", "authority"], "role"),
            (&["company", "organization", "org"], "company"),
        ];

        // Check each pattern
        for (keywords, entity) in entity_patterns {
            for keyword in *keywords {
                if lower.contains(keyword) {
                    return format!("{}_list", entity);
                }
            }
        }

        // Try to extract entity from common patterns like "X list", "X screen", "X management"
        let extraction_patterns = [
            " list", " screen", " management", " manager", " page", " view",
            " 목록", " 화면", " 관리", " 조회",
        ];

        for pattern in extraction_patterns {
            if let Some(pos) = lower.find(pattern) {
                // Get the word before the pattern
                let before = &lower[..pos];
                let words: Vec<&str> = before.split_whitespace().collect();
                if let Some(last_word) = words.last() {
                    // Clean and use as entity name
                    let entity = last_word
                        .trim_matches(|c: char| !c.is_alphanumeric())
                        .to_lowercase();
                    if !entity.is_empty() && entity.len() > 1 {
                        return format!("{}_list", entity);
                    }
                }
            }
        }

        // Default fallback
        "screen_list".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_schema_basic() {
        let schema = SchemaInput::new("member")
            .with_column(SchemaColumn::new("id", "INTEGER").primary_key())
            .with_column(SchemaColumn::new("name", "VARCHAR(100)").not_null())
            .with_column(SchemaColumn::new("email", "VARCHAR(255)"))
            .with_column(SchemaColumn::new("created_at", "DATETIME"));

        let intent = NormalizerService::normalize_schema(&schema).unwrap();

        assert_eq!(intent.screen_name, "member_list");
        assert_eq!(intent.screen_type, ScreenType::List);
        assert_eq!(intent.datasets.len(), 1);
        assert_eq!(intent.datasets[0].columns.len(), 4);
        assert_eq!(intent.grids.len(), 1);
    }

    #[test]
    fn test_infer_types() {
        assert_eq!(
            NormalizerService::infer_types("VARCHAR(100)", false),
            (UiType::Input, DataType::String)
        );
        assert_eq!(
            NormalizerService::infer_types("TEXT", false),
            (UiType::TextArea, DataType::Text)
        );
        assert_eq!(
            NormalizerService::infer_types("DATE", false),
            (UiType::DatePicker, DataType::Date)
        );
        assert_eq!(
            NormalizerService::infer_types("BOOLEAN", false),
            (UiType::Checkbox, DataType::Boolean)
        );
        assert_eq!(
            NormalizerService::infer_types("INTEGER", false),
            (UiType::Number, DataType::Integer)
        );
    }

    #[test]
    fn test_infer_label() {
        assert_eq!(NormalizerService::infer_label("email", None), "이메일");
        assert_eq!(NormalizerService::infer_label("created_at", None), "등록일");
        assert_eq!(NormalizerService::infer_label("member_name", None), "회원명");
        assert_eq!(NormalizerService::infer_label("custom_field", None), "Custom Field");
    }

    #[test]
    fn test_extract_varchar_length() {
        assert_eq!(NormalizerService::extract_varchar_length("VARCHAR(100)"), Some(100));
        assert_eq!(NormalizerService::extract_varchar_length("CHAR(10)"), Some(10));
        assert_eq!(NormalizerService::extract_varchar_length("TEXT"), None);
    }

    #[test]
    fn test_extract_table_from_query() {
        assert_eq!(
            NormalizerService::extract_table_from_query("SELECT * FROM members WHERE id = 1").unwrap(),
            "members"
        );
        assert_eq!(
            NormalizerService::extract_table_from_query("SELECT id, name FROM schema.users u").unwrap(),
            "users"
        );
    }
}
