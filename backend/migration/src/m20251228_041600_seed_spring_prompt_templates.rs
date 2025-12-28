use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Insert Spring Framework prompt template for CRUD operations
        let insert = Query::insert()
            .into_table(Alias::new("prompt_templates"))
            .columns([
                Alias::new("name"),
                Alias::new("product"),
                Alias::new("screen_type"),
                Alias::new("system_prompt"),
                Alias::new("user_prompt_template"),
                Alias::new("version"),
                Alias::new("is_active"),
            ])
            .values_panic([
                "spring-crud".into(),
                "spring-backend".into(),
                "crud".into(),
                SPRING_SYSTEM_PROMPT.into(),
                SPRING_USER_PROMPT_TEMPLATE.into(),
                1.into(),
                true.into(),
            ])
            .to_owned();

        m.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Delete the Spring CRUD template
        let delete = Query::delete()
            .from_table(Alias::new("prompt_templates"))
            .and_where(Expr::col(Alias::new("product")).eq("spring-backend"))
            .and_where(Expr::col(Alias::new("name")).eq("spring-crud"))
            .to_owned();

        m.exec_stmt(delete).await?;

        Ok(())
    }
}

const SPRING_SYSTEM_PROMPT: &str = r#"You are a Spring Framework code generator. Your task is to generate clean, production-ready Java code following Spring best practices.

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
- Use Lombok @Data, @Builder, @NoArgsConstructor, @AllArgsConstructor on DTOs
- Add @NotNull, @NotBlank, @Size for required/sized fields

{{company_rules}}"#;

const SPRING_USER_PROMPT_TEMPLATE: &str = r#"Generate Spring Framework backend code for the '{{entity_name}}' entity.

PACKAGE STRUCTURE:
- Base package: {{package_base}}
- Controller: {{package_base}}.controller.{{entity_name}}Controller
- Service: {{package_base}}.service.{{entity_name}}Service
- ServiceImpl: {{package_base}}.service.impl.{{entity_name}}ServiceImpl
- DTO: {{package_base}}.dto.{{entity_name}}DTO
- Mapper: {{package_base}}.mapper.{{entity_name}}Mapper

ENTITY INFORMATION:
- Entity name: {{entity_name}}
- Table name: {{table_name}}
- API path: /api/{{path_name}}

COLUMN DEFINITIONS:
{{columns}}

CRUD OPERATIONS TO GENERATE:
{{crud_operations}}

{{#if notes}}
ADDITIONAL NOTES:
{{notes}}
{{/if}}

Generate the complete code for all 6 sections (Controller, Service, ServiceImpl, DTO, Mapper, MapperXML)."#;
