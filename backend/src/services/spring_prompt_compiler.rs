use crate::domain::{CrudOperation, SpringIntent, to_camel_case};
use crate::models::_entities::{company_rules, prompt_templates};
use anyhow::Result;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

/// Compiled prompt for Spring code generation
#[derive(Debug, Clone)]
pub struct SpringCompiledPrompt {
    pub system: String,
    pub user: String,
}

impl SpringCompiledPrompt {
    pub fn full(&self) -> String {
        format!("{}\n\n{}", self.system, self.user)
    }
}

/// Service for compiling SpringIntent into LLM prompts
pub struct SpringPromptCompiler;

impl SpringPromptCompiler {
    /// Compile a SpringIntent into a prompt using templates from database
    pub async fn compile(
        db: &DatabaseConnection,
        intent: &SpringIntent,
        company_id: Option<&str>,
    ) -> Result<SpringCompiledPrompt> {
        // 1. Load template from DB (or use defaults)
        let template = Self::load_template(db, "spring-backend", "crud").await;

        // 2. Load company rules if provided
        let rules = if let Some(cid) = company_id {
            Self::load_company_rules(db, cid).await.ok()
        } else {
            None
        };

        // 3. Build prompts
        let system = Self::build_system_prompt(&template, &rules, intent);
        let user = Self::build_user_prompt(&template, intent, &rules);

        Ok(SpringCompiledPrompt { system, user })
    }

    /// Compile using default templates (no database)
    pub fn compile_with_defaults(intent: &SpringIntent, company_rules: Option<&str>) -> SpringCompiledPrompt {
        let system = Self::get_default_system_prompt(intent);
        let user = Self::build_user_prompt_from_intent(intent, company_rules);

        SpringCompiledPrompt { system, user }
    }

    /// Load template from database
    async fn load_template(
        db: &DatabaseConnection,
        product: &str,
        screen_type: &str,
    ) -> Option<prompt_templates::Model> {
        prompt_templates::Entity::find()
            .filter(prompt_templates::Column::Product.eq(product))
            .filter(prompt_templates::Column::ScreenType.eq(Some(screen_type.to_string())))
            .filter(prompt_templates::Column::IsActive.eq(Some(true)))
            .one(db)
            .await
            .ok()
            .flatten()
    }

    /// Load company rules from database
    /// In single-company on-premise mode, we load rules by name
    async fn load_company_rules(
        db: &DatabaseConnection,
        rule_name: &str,
    ) -> Result<company_rules::Model> {
        company_rules::Entity::find()
            .filter(company_rules::Column::Name.eq(rule_name))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Company rules not found for: {}", rule_name))
    }

    /// Get default system prompt for Spring code generation
    fn get_default_system_prompt(intent: &SpringIntent) -> String {
        let mut prompt = String::from(
r#"You are a Spring Framework code generator. Your task is to generate clean, production-ready Java code following Spring best practices.

GENERAL RULES:
1. Use @RestController with @RequestMapping for controllers
2. Use @Service annotation for service implementations
3. Use @Mapper annotation for MyBatis mappers
4. All method and variable names should follow Java camelCase convention
5. Use Lombok annotations (@Data, @Builder, @NoArgsConstructor, @AllArgsConstructor) for DTOs
6. Add validation annotations (@NotNull, @NotBlank, @Size) for required fields
7. Use proper exception handling with @ControllerAdvice pattern (reference only, don't generate)
8. Follow RESTful API conventions for endpoint paths and HTTP methods

MYBATIS RULES:
1. Use #{paramName} for parameter binding (NEVER use ${} to prevent SQL injection)
2. Define proper resultMap for complex mappings
3. Use <if> for dynamic SQL conditions
4. Add proper typeHandler for Java 8 date types if needed

OUTPUT FORMAT:
You must output exactly 6 sections with these markers:
--- CONTROLLER ---
[Complete Controller class with all annotations and methods]

--- SERVICE ---
[Complete Service interface]

--- SERVICE_IMPL ---
[Complete Service implementation class]

--- DTO ---
[Complete DTO class with Lombok and validation annotations]

--- MAPPER ---
[Complete MyBatis Mapper interface]

--- MAPPER_XML ---
[Complete MyBatis Mapper XML with namespace, resultMap, and all CRUD statements]

IMPORTANT:
- Each section must be complete and compilable Java/XML code
- Include all necessary imports
- Use package names as specified
- Generate TODO comments for any unclear or configurable parts
"#);

        // Add Lombok usage note
        if intent.options.use_lombok {
            prompt.push_str("\nLOMBOK: Use @Data, @Builder, @NoArgsConstructor, @AllArgsConstructor on DTOs.\n");
        }

        // Add validation note
        if intent.options.use_validation {
            prompt.push_str("VALIDATION: Add @NotNull, @NotBlank, @Size for required/sized fields.\n");
        }

        prompt
    }

    /// Build system prompt from template and rules
    fn build_system_prompt(
        template: &Option<prompt_templates::Model>,
        rules: &Option<company_rules::Model>,
        intent: &SpringIntent,
    ) -> String {
        let base_prompt = template
            .as_ref()
            .map(|t| t.system_prompt.clone())
            .unwrap_or_else(|| Self::get_default_system_prompt(intent));

        // Append company rules if available
        if let Some(r) = rules {
            if let Some(ref additional) = r.additional_rules {
                if !additional.is_empty() {
                    return format!("{}\n\nCOMPANY-SPECIFIC RULES:\n{}", base_prompt, additional);
                }
            }
        }

        base_prompt
    }

    /// Build user prompt from template and intent
    fn build_user_prompt(
        template: &Option<prompt_templates::Model>,
        intent: &SpringIntent,
        rules: &Option<company_rules::Model>,
    ) -> String {
        let company_rules_str = rules
            .as_ref()
            .and_then(|r| r.additional_rules.clone())
            .unwrap_or_default();

        if let Some(t) = template {
            Self::render_template(&t.user_prompt_template, intent, &company_rules_str)
        } else {
            let rules_ref = if company_rules_str.is_empty() {
                None
            } else {
                Some(company_rules_str.as_str())
            };
            Self::build_user_prompt_from_intent(intent, rules_ref)
        }
    }

    /// Render a template with intent data
    fn render_template(template: &str, intent: &SpringIntent, company_rules: &str) -> String {
        template
            .replace("{{entity_name}}", &intent.entity_name)
            .replace("{{table_name}}", &intent.table_name)
            .replace("{{package_base}}", &intent.package_base)
            .replace("{{columns}}", &Self::describe_columns(intent))
            .replace("{{crud_operations}}", &Self::describe_operations(intent))
            .replace("{{company_rules}}", company_rules)
    }

    /// Build user prompt directly from intent
    fn build_user_prompt_from_intent(intent: &SpringIntent, company_rules: Option<&str>) -> String {
        let mut prompt = format!(
            "Generate Spring Framework backend code for the '{}' entity.\n\n",
            intent.entity_name
        );

        // Package information
        prompt.push_str("PACKAGE STRUCTURE:\n");
        prompt.push_str(&format!("- Base package: {}\n", intent.package_base));
        prompt.push_str(&format!("- Controller: {}.controller.{}\n", intent.package_base, intent.controller_name()));
        prompt.push_str(&format!("- Service: {}.service.{}\n", intent.package_base, intent.service_name()));
        prompt.push_str(&format!("- ServiceImpl: {}.service.impl.{}\n", intent.package_base, intent.service_impl_name()));
        prompt.push_str(&format!("- DTO: {}.dto.{}\n", intent.package_base, intent.dto_name()));
        prompt.push_str(&format!("- Mapper: {}.mapper.{}\n", intent.package_base, intent.mapper_name()));

        // Entity information
        prompt.push_str("\nENTITY INFORMATION:\n");
        prompt.push_str(&format!("- Entity name: {}\n", intent.entity_name));
        prompt.push_str(&format!("- Table name: {}\n", intent.table_name));
        prompt.push_str(&format!("- API path: /api/{}\n", intent.path_name()));

        // Column definitions
        prompt.push_str("\nCOLUMN DEFINITIONS:\n");
        for col in &intent.columns {
            let java_type = Self::infer_java_type(&col.data_type);
            let field_name = to_camel_case(&col.name);
            prompt.push_str(&format!(
                "- {} {} ({}){}{}",
                java_type,
                field_name,
                col.label,
                if col.is_pk { " [PK]" } else { "" },
                if col.required { " [REQUIRED]" } else { "" }
            ));
            if let Some(len) = col.max_length {
                prompt.push_str(&format!(" [MAX:{}]", len));
            }
            prompt.push('\n');
        }

        // CRUD operations
        prompt.push_str("\nCRUD OPERATIONS TO GENERATE:\n");
        for op in &intent.crud_operations {
            prompt.push_str(&format!("- {:?}: {} {}\n", op, op.http_method(), Self::describe_operation(op, intent)));
        }

        // Response wrapper
        if let Some(ref wrapper) = intent.options.response_wrapper {
            prompt.push_str(&format!("\nRESPONSE WRAPPER: Use {} for all responses\n", wrapper));
        }

        // Company rules
        if let Some(rules) = company_rules {
            if !rules.is_empty() {
                prompt.push_str(&format!("\nCOMPANY-SPECIFIC RULES:\n{}\n", rules));
            }
        }

        prompt.push_str("\nGenerate the complete code for all 6 sections (Controller, Service, ServiceImpl, DTO, Mapper, MapperXML).");

        prompt
    }

    /// Describe columns for template
    fn describe_columns(intent: &SpringIntent) -> String {
        intent.columns.iter()
            .map(|col| {
                let java_type = Self::infer_java_type(&col.data_type);
                format!("{} {} ({})", java_type, to_camel_case(&col.name), col.label)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Describe operations for template
    fn describe_operations(intent: &SpringIntent) -> String {
        intent.crud_operations.iter()
            .map(|op| format!("{:?}", op))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Describe a single operation
    fn describe_operation(op: &CrudOperation, intent: &SpringIntent) -> String {
        match op {
            CrudOperation::Create => format!("/api/{}", intent.path_name()),
            CrudOperation::Read => format!("/api/{}/{{id}}", intent.path_name()),
            CrudOperation::ReadList => format!("/api/{}", intent.path_name()),
            CrudOperation::Update => format!("/api/{}/{{id}}", intent.path_name()),
            CrudOperation::Delete => format!("/api/{}/{{id}}", intent.path_name()),
        }
    }

    /// Infer Java type from DataType
    fn infer_java_type(data_type: &crate::domain::DataType) -> &'static str {
        match data_type {
            crate::domain::DataType::String => "String",
            crate::domain::DataType::Integer => "Long",
            crate::domain::DataType::Decimal => "BigDecimal",
            crate::domain::DataType::Boolean => "Boolean",
            crate::domain::DataType::Date => "LocalDate",
            crate::domain::DataType::DateTime => "LocalDateTime",
            crate::domain::DataType::Text => "String",
            crate::domain::DataType::Binary => "byte[]",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ColumnIntent, DataType, UiType};

    fn create_test_intent() -> SpringIntent {
        SpringIntent::new("Member", "TB_MEMBER", "com.company.project")
            .with_column(
                ColumnIntent::new("member_id", "회원ID")
                    .with_ui_type(UiType::Hidden)
                    .with_data_type(DataType::Integer)
                    .primary_key()
            )
            .with_column(
                ColumnIntent::new("member_name", "회원명")
                    .with_ui_type(UiType::Input)
                    .with_data_type(DataType::String)
                    .required()
                    .with_max_length(100)
            )
            .with_column(
                ColumnIntent::new("email", "이메일")
                    .with_ui_type(UiType::Input)
                    .with_data_type(DataType::String)
            )
    }

    #[test]
    fn test_compile_with_defaults() {
        let intent = create_test_intent();
        let prompt = SpringPromptCompiler::compile_with_defaults(&intent, None);

        assert!(!prompt.system.is_empty());
        assert!(prompt.user.contains("Member"));
        assert!(prompt.user.contains("TB_MEMBER"));
        assert!(prompt.user.contains("com.company.project"));
    }

    #[test]
    fn test_user_prompt_contains_columns() {
        let intent = create_test_intent();
        let prompt = SpringPromptCompiler::compile_with_defaults(&intent, None);

        assert!(prompt.user.contains("memberId"));
        assert!(prompt.user.contains("memberName"));
        assert!(prompt.user.contains("email"));
        assert!(prompt.user.contains("[PK]"));
        assert!(prompt.user.contains("[REQUIRED]"));
    }

    #[test]
    fn test_user_prompt_contains_operations() {
        let intent = create_test_intent();
        let prompt = SpringPromptCompiler::compile_with_defaults(&intent, None);

        assert!(prompt.user.contains("Create"));
        assert!(prompt.user.contains("Read"));
        assert!(prompt.user.contains("ReadList"));
        assert!(prompt.user.contains("Update"));
        assert!(prompt.user.contains("Delete"));
    }

    #[test]
    fn test_full_prompt() {
        let intent = create_test_intent();
        let prompt = SpringPromptCompiler::compile_with_defaults(&intent, None);
        let full = prompt.full();

        assert!(full.contains("@RestController"));
        assert!(full.contains("@Service"));
        assert!(full.contains("MyBatis"));
        assert!(full.contains("Member"));
    }
}
