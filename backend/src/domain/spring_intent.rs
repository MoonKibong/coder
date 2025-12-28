use serde::{Deserialize, Serialize};

use super::ColumnIntent;

/// Internal DSL for representing Spring Framework code generation intent.
/// This is the normalized representation for generating backend code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringIntent {
    /// Entity name in PascalCase (e.g., "Member", "OrderDetail")
    pub entity_name: String,

    /// Database table name (e.g., "TB_MEMBER", "TB_ORDER_DETAIL")
    pub table_name: String,

    /// Base package name (e.g., "com.company.project")
    pub package_base: String,

    /// Column definitions (shared with UI)
    pub columns: Vec<ColumnIntent>,

    /// CRUD operations to generate
    pub crud_operations: Vec<CrudOperation>,

    /// Additional options
    pub options: SpringOptions,
}

impl SpringIntent {
    pub fn new(
        entity_name: impl Into<String>,
        table_name: impl Into<String>,
        package_base: impl Into<String>,
    ) -> Self {
        Self {
            entity_name: entity_name.into(),
            table_name: table_name.into(),
            package_base: package_base.into(),
            columns: Vec::new(),
            crud_operations: vec![
                CrudOperation::Create,
                CrudOperation::Read,
                CrudOperation::ReadList,
                CrudOperation::Update,
                CrudOperation::Delete,
            ],
            options: SpringOptions::default(),
        }
    }

    pub fn with_column(mut self, column: ColumnIntent) -> Self {
        self.columns.push(column);
        self
    }

    pub fn with_columns(mut self, columns: Vec<ColumnIntent>) -> Self {
        self.columns = columns;
        self
    }

    pub fn with_operations(mut self, operations: Vec<CrudOperation>) -> Self {
        self.crud_operations = operations;
        self
    }

    pub fn with_options(mut self, options: SpringOptions) -> Self {
        self.options = options;
        self
    }

    /// Get the controller class name
    pub fn controller_name(&self) -> String {
        format!("{}Controller", self.entity_name)
    }

    /// Get the service interface name
    pub fn service_name(&self) -> String {
        format!("{}Service", self.entity_name)
    }

    /// Get the service implementation name
    pub fn service_impl_name(&self) -> String {
        format!("{}ServiceImpl", self.entity_name)
    }

    /// Get the DTO class name
    pub fn dto_name(&self) -> String {
        format!("{}DTO", self.entity_name)
    }

    /// Get the mapper interface name
    pub fn mapper_name(&self) -> String {
        format!("{}Mapper", self.entity_name)
    }

    /// Get the entity name in lowercase for URL paths
    pub fn path_name(&self) -> String {
        // Convert PascalCase to kebab-case
        let mut result = String::new();
        for (i, c) in self.entity_name.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('-');
                }
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Get the primary key columns
    pub fn primary_key_columns(&self) -> Vec<&ColumnIntent> {
        self.columns.iter().filter(|c| c.is_pk).collect()
    }
}

/// CRUD operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrudOperation {
    /// Create new record
    Create,
    /// Read single record by ID
    Read,
    /// Read list of records (with pagination)
    ReadList,
    /// Update existing record
    Update,
    /// Delete record
    Delete,
}

impl CrudOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            CrudOperation::Create => "create",
            CrudOperation::Read => "read",
            CrudOperation::ReadList => "read_list",
            CrudOperation::Update => "update",
            CrudOperation::Delete => "delete",
        }
    }

    /// Get the HTTP method for this operation
    pub fn http_method(&self) -> &'static str {
        match self {
            CrudOperation::Create => "POST",
            CrudOperation::Read => "GET",
            CrudOperation::ReadList => "GET",
            CrudOperation::Update => "PUT",
            CrudOperation::Delete => "DELETE",
        }
    }

    /// Get the Spring annotation for this operation
    pub fn spring_annotation(&self) -> &'static str {
        match self {
            CrudOperation::Create => "@PostMapping",
            CrudOperation::Read => "@GetMapping",
            CrudOperation::ReadList => "@GetMapping",
            CrudOperation::Update => "@PutMapping",
            CrudOperation::Delete => "@DeleteMapping",
        }
    }
}

/// Spring generation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringOptions {
    /// Use Lombok annotations (@Data, @Builder, etc.)
    pub use_lombok: bool,

    /// Use validation annotations (@NotNull, @Size, etc.)
    pub use_validation: bool,

    /// Use Swagger/OpenAPI annotations
    pub use_swagger: bool,

    /// Generate MyBatis mapper (vs JPA repository)
    pub use_mybatis: bool,

    /// Include audit fields (created_at, updated_at, created_by, updated_by)
    pub include_audit_fields: bool,

    /// Generate search/filter DTO
    pub generate_search_dto: bool,

    /// Base response wrapper class (e.g., "ApiResponse")
    pub response_wrapper: Option<String>,
}

impl Default for SpringOptions {
    fn default() -> Self {
        Self {
            use_lombok: true,
            use_validation: true,
            use_swagger: false,
            use_mybatis: true, // MyBatis is more common in Korean enterprise
            include_audit_fields: true,
            generate_search_dto: true,
            response_wrapper: Some("ApiResponse".to_string()),
        }
    }
}

/// Generated Spring artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringArtifacts {
    /// Controller class content
    pub controller: String,

    /// Service interface content
    pub service_interface: String,

    /// Service implementation content
    pub service_impl: String,

    /// DTO class content
    pub dto: String,

    /// Search DTO class content (optional)
    pub search_dto: Option<String>,

    /// Mapper interface content (MyBatis)
    pub mapper_interface: String,

    /// Mapper XML content (MyBatis)
    pub mapper_xml: String,

    /// Validation warnings
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl SpringArtifacts {
    pub fn new() -> Self {
        Self {
            controller: String::new(),
            service_interface: String::new(),
            service_impl: String::new(),
            dto: String::new(),
            search_dto: None,
            mapper_interface: String::new(),
            mapper_xml: String::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}

impl Default for SpringArtifacts {
    fn default() -> Self {
        Self::new()
    }
}

/// Java type mapping from database types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaType {
    String,
    Integer,
    Long,
    Double,
    BigDecimal,
    Boolean,
    LocalDate,
    LocalDateTime,
    ByteArray,
}

impl JavaType {
    /// Infer Java type from database column type
    pub fn from_db_type(db_type: &str) -> Self {
        let upper = db_type.to_uppercase();

        if upper.contains("VARCHAR") || upper.contains("CHAR") || upper.contains("TEXT") || upper.contains("CLOB") {
            JavaType::String
        } else if upper.contains("BIGINT") || upper.contains("SERIAL") {
            JavaType::Long
        } else if upper.contains("INT") || upper.contains("SMALLINT") || upper.contains("TINYINT") {
            JavaType::Integer
        } else if upper.contains("DECIMAL") || upper.contains("NUMERIC") {
            JavaType::BigDecimal
        } else if upper.contains("DOUBLE") || upper.contains("FLOAT") || upper.contains("REAL") {
            JavaType::Double
        } else if upper.contains("BOOLEAN") || upper.contains("BIT") {
            JavaType::Boolean
        } else if upper.contains("TIMESTAMP") || upper.contains("DATETIME") {
            JavaType::LocalDateTime
        } else if upper.contains("DATE") {
            JavaType::LocalDate
        } else if upper.contains("BLOB") || upper.contains("BINARY") {
            JavaType::ByteArray
        } else {
            JavaType::String
        }
    }

    /// Get the Java type name
    pub fn as_str(&self) -> &'static str {
        match self {
            JavaType::String => "String",
            JavaType::Integer => "Integer",
            JavaType::Long => "Long",
            JavaType::Double => "Double",
            JavaType::BigDecimal => "BigDecimal",
            JavaType::Boolean => "Boolean",
            JavaType::LocalDate => "LocalDate",
            JavaType::LocalDateTime => "LocalDateTime",
            JavaType::ByteArray => "byte[]",
        }
    }

    /// Get import statement if needed
    pub fn import_statement(&self) -> Option<&'static str> {
        match self {
            JavaType::BigDecimal => Some("java.math.BigDecimal"),
            JavaType::LocalDate => Some("java.time.LocalDate"),
            JavaType::LocalDateTime => Some("java.time.LocalDateTime"),
            _ => None,
        }
    }

    /// Get MyBatis JDBC type
    pub fn jdbc_type(&self) -> &'static str {
        match self {
            JavaType::String => "VARCHAR",
            JavaType::Integer => "INTEGER",
            JavaType::Long => "BIGINT",
            JavaType::Double => "DOUBLE",
            JavaType::BigDecimal => "DECIMAL",
            JavaType::Boolean => "BOOLEAN",
            JavaType::LocalDate => "DATE",
            JavaType::LocalDateTime => "TIMESTAMP",
            JavaType::ByteArray => "BLOB",
        }
    }
}

/// Convert column name to Java field name (snake_case to camelCase)
pub fn to_camel_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in name.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    result
}

/// Convert name to PascalCase
pub fn to_pascal_case(name: &str) -> String {
    let camel = to_camel_case(name);
    let mut chars = camel.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_intent_creation() {
        let intent = SpringIntent::new("Member", "TB_MEMBER", "com.company.project");

        assert_eq!(intent.entity_name, "Member");
        assert_eq!(intent.table_name, "TB_MEMBER");
        assert_eq!(intent.package_base, "com.company.project");
        assert_eq!(intent.crud_operations.len(), 5);
    }

    #[test]
    fn test_naming_conventions() {
        let intent = SpringIntent::new("OrderDetail", "TB_ORDER_DETAIL", "com.company");

        assert_eq!(intent.controller_name(), "OrderDetailController");
        assert_eq!(intent.service_name(), "OrderDetailService");
        assert_eq!(intent.service_impl_name(), "OrderDetailServiceImpl");
        assert_eq!(intent.dto_name(), "OrderDetailDTO");
        assert_eq!(intent.mapper_name(), "OrderDetailMapper");
        assert_eq!(intent.path_name(), "order-detail");
    }

    #[test]
    fn test_java_type_inference() {
        assert_eq!(JavaType::from_db_type("VARCHAR(100)"), JavaType::String);
        assert_eq!(JavaType::from_db_type("INTEGER"), JavaType::Integer);
        assert_eq!(JavaType::from_db_type("BIGINT"), JavaType::Long);
        assert_eq!(JavaType::from_db_type("DECIMAL(10,2)"), JavaType::BigDecimal);
        assert_eq!(JavaType::from_db_type("DATE"), JavaType::LocalDate);
        assert_eq!(JavaType::from_db_type("TIMESTAMP"), JavaType::LocalDateTime);
        assert_eq!(JavaType::from_db_type("BOOLEAN"), JavaType::Boolean);
    }

    #[test]
    fn test_camel_case_conversion() {
        assert_eq!(to_camel_case("member_id"), "memberId");
        assert_eq!(to_camel_case("created_at"), "createdAt");
        assert_eq!(to_camel_case("MEMBER_NAME"), "memberName");
        assert_eq!(to_camel_case("id"), "id");
    }

    #[test]
    fn test_pascal_case_conversion() {
        assert_eq!(to_pascal_case("member_id"), "MemberId");
        assert_eq!(to_pascal_case("created_at"), "CreatedAt");
        assert_eq!(to_pascal_case("member"), "Member");
    }

    #[test]
    fn test_crud_operations() {
        assert_eq!(CrudOperation::Create.http_method(), "POST");
        assert_eq!(CrudOperation::Read.http_method(), "GET");
        assert_eq!(CrudOperation::Update.http_method(), "PUT");
        assert_eq!(CrudOperation::Delete.http_method(), "DELETE");

        assert_eq!(CrudOperation::Create.spring_annotation(), "@PostMapping");
        assert_eq!(CrudOperation::Read.spring_annotation(), "@GetMapping");
    }
}
