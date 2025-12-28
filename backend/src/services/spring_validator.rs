use crate::domain::{CrudOperation, SpringArtifacts, SpringIntent, to_camel_case};
use anyhow::{anyhow, Result};
use regex::Regex;

/// Service for validating Spring Framework output
pub struct SpringValidator;

impl SpringValidator {
    /// Parse and validate LLM output for Spring code
    pub fn parse_and_validate(raw: &str, intent: &SpringIntent) -> Result<SpringArtifacts> {
        // 1. Split sections
        let sections = Self::split_output(raw)?;

        // 2. Validate each section
        let mut warnings = Vec::new();

        warnings.extend(Self::validate_controller(&sections.controller, intent)?);
        warnings.extend(Self::validate_service(&sections.service_interface, intent)?);
        warnings.extend(Self::validate_service_impl(&sections.service_impl, &sections.service_interface)?);
        warnings.extend(Self::validate_dto(&sections.dto, intent)?);
        warnings.extend(Self::validate_mapper(&sections.mapper_interface, intent)?);
        warnings.extend(Self::validate_mapper_xml(&sections.mapper_xml, intent)?);

        Ok(SpringArtifacts {
            controller: sections.controller,
            service_interface: sections.service_interface,
            service_impl: sections.service_impl,
            dto: sections.dto,
            search_dto: sections.search_dto,
            mapper_interface: sections.mapper_interface,
            mapper_xml: sections.mapper_xml,
            warnings,
        })
    }

    /// Split LLM output into separate code sections
    fn split_output(raw: &str) -> Result<ParsedSections> {
        let controller = Self::extract_section(raw, &["--- CONTROLLER ---", "---CONTROLLER---", "// Controller"])?;
        let service_interface = Self::extract_section(raw, &["--- SERVICE ---", "---SERVICE---", "// Service Interface"])?;
        let service_impl = Self::extract_section(raw, &["--- SERVICE_IMPL ---", "---SERVICE_IMPL---", "// Service Implementation"])?;
        let dto = Self::extract_section(raw, &["--- DTO ---", "---DTO---", "// DTO"])?;
        let mapper_interface = Self::extract_section(raw, &["--- MAPPER ---", "---MAPPER---", "// Mapper Interface"])?;
        let mapper_xml = Self::extract_section(raw, &["--- MAPPER_XML ---", "---MAPPER_XML---", "<!-- Mapper XML -->"])?;

        // Search DTO is optional
        let search_dto = Self::extract_section(raw, &["--- SEARCH_DTO ---", "---SEARCH_DTO---", "// Search DTO"]).ok();

        Ok(ParsedSections {
            controller,
            service_interface,
            service_impl,
            dto,
            search_dto,
            mapper_interface,
            mapper_xml,
        })
    }

    /// Extract a section from the raw output
    fn extract_section(raw: &str, markers: &[&str]) -> Result<String> {
        let start_pos = markers.iter()
            .filter_map(|m| raw.find(m).map(|pos| (pos, m.len())))
            .min_by_key(|(pos, _)| *pos);

        if let Some((start, marker_len)) = start_pos {
            let content_start = start + marker_len;

            // Find the next section marker or end of text
            let end_markers = [
                "--- CONTROLLER ---", "---CONTROLLER---",
                "--- SERVICE ---", "---SERVICE---",
                "--- SERVICE_IMPL ---", "---SERVICE_IMPL---",
                "--- DTO ---", "---DTO---",
                "--- SEARCH_DTO ---", "---SEARCH_DTO---",
                "--- MAPPER ---", "---MAPPER---",
                "--- MAPPER_XML ---", "---MAPPER_XML---",
            ];

            let end_pos = end_markers.iter()
                .filter_map(|m| {
                    raw[content_start..].find(m).map(|p| content_start + p)
                })
                .min()
                .unwrap_or(raw.len());

            let content = Self::clean_section(&raw[content_start..end_pos]);

            if content.is_empty() {
                return Err(anyhow!("Section is empty after marker: {:?}", markers[0]));
            }

            return Ok(content);
        }

        Err(anyhow!("Section not found: {:?}", markers[0]))
    }

    /// Clean section content
    fn clean_section(text: &str) -> String {
        let mut result = text.trim().to_string();

        // Remove markdown code blocks
        if result.starts_with("```java") {
            result = result.strip_prefix("```java").unwrap_or(&result).to_string();
        }
        if result.starts_with("```xml") {
            result = result.strip_prefix("```xml").unwrap_or(&result).to_string();
        }
        if result.starts_with("```") {
            result = result.strip_prefix("```").unwrap_or(&result).to_string();
        }
        if result.ends_with("```") {
            result = result.strip_suffix("```").unwrap_or(&result).to_string();
        }

        result.trim().to_string()
    }

    /// Validate Controller class
    fn validate_controller(code: &str, intent: &SpringIntent) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for @RestController annotation
        if !code.contains("@RestController") {
            warnings.push("Warning: Missing @RestController annotation".to_string());
        }

        // Check for @RequestMapping with correct path
        let expected_path = intent.path_name();
        if !code.contains("@RequestMapping") {
            warnings.push("Warning: Missing @RequestMapping annotation".to_string());
        } else if !code.to_lowercase().contains(&expected_path.to_lowercase()) {
            warnings.push(format!("Note: Expected path '{}' in @RequestMapping", expected_path));
        }

        // Check for expected CRUD endpoints
        for op in &intent.crud_operations {
            let annotation = op.spring_annotation();
            if !code.contains(annotation) {
                warnings.push(format!("Warning: Missing {} for {:?} operation", annotation, op));
            }
        }

        // Check for service injection
        if !code.contains("@Autowired") && !code.contains("@RequiredArgsConstructor") {
            warnings.push("Warning: No dependency injection found".to_string());
        }

        // Check class name
        let expected_class = intent.controller_name();
        if !code.contains(&format!("class {}", expected_class)) {
            warnings.push(format!("Note: Expected class name '{}'", expected_class));
        }

        Ok(warnings)
    }

    /// Validate Service interface
    fn validate_service(code: &str, intent: &SpringIntent) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for interface declaration
        let expected_interface = intent.service_name();
        if !code.contains(&format!("interface {}", expected_interface)) {
            warnings.push(format!("Warning: Expected interface '{}'", expected_interface));
        }

        // Check for expected methods
        for op in &intent.crud_operations {
            let method_pattern = Self::expected_method_name(op, &intent.entity_name);
            if !code.contains(&method_pattern) {
                warnings.push(format!("Warning: Missing method '{}' for {:?}", method_pattern, op));
            }
        }

        Ok(warnings)
    }

    /// Validate Service implementation
    fn validate_service_impl(code: &str, interface_code: &str) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for @Service annotation
        if !code.contains("@Service") {
            warnings.push("Warning: Missing @Service annotation".to_string());
        }

        // Check for implements clause
        if !code.contains("implements") {
            warnings.push("Warning: ServiceImpl should implement Service interface".to_string());
        }

        // Check that all interface methods are implemented
        let method_regex = Regex::new(r"(\w+)\s*\([^)]*\)\s*;").unwrap();
        for cap in method_regex.captures_iter(interface_code) {
            let method_name = &cap[1];
            if !code.contains(method_name) {
                warnings.push(format!("Warning: Method '{}' not implemented", method_name));
            }
        }

        // Check for mapper injection
        if !code.contains("Mapper") {
            warnings.push("Note: No Mapper reference found in ServiceImpl".to_string());
        }

        Ok(warnings)
    }

    /// Validate DTO class
    fn validate_dto(code: &str, intent: &SpringIntent) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check class name
        let expected_class = intent.dto_name();
        if !code.contains(&format!("class {}", expected_class)) {
            warnings.push(format!("Warning: Expected class '{}'", expected_class));
        }

        // Check for Lombok annotations (if enabled)
        if intent.options.use_lombok {
            if !code.contains("@Data") && !code.contains("@Getter") {
                warnings.push("Note: Consider adding @Data or @Getter/@Setter".to_string());
            }
        }

        // Check for validation annotations (if enabled)
        if intent.options.use_validation {
            let has_validation = code.contains("@NotNull") ||
                                 code.contains("@NotBlank") ||
                                 code.contains("@Size") ||
                                 code.contains("@Valid");
            if !has_validation {
                warnings.push("Note: Consider adding validation annotations".to_string());
            }
        }

        // Check that all columns are represented
        for col in &intent.columns {
            let field_name = to_camel_case(&col.name);
            if !code.contains(&field_name) {
                warnings.push(format!("Warning: Field '{}' not found in DTO", field_name));
            }
        }

        Ok(warnings)
    }

    /// Validate Mapper interface
    fn validate_mapper(code: &str, intent: &SpringIntent) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for @Mapper annotation
        if !code.contains("@Mapper") {
            warnings.push("Warning: Missing @Mapper annotation".to_string());
        }

        // Check interface name
        let expected_interface = intent.mapper_name();
        if !code.contains(&format!("interface {}", expected_interface)) {
            warnings.push(format!("Warning: Expected interface '{}'", expected_interface));
        }

        // Check for CRUD method signatures
        for op in &intent.crud_operations {
            let method_pattern = Self::expected_mapper_method(op);
            if !code.contains(&method_pattern) {
                warnings.push(format!("Note: Consider adding '{}' method", method_pattern));
            }
        }

        Ok(warnings)
    }

    /// Validate Mapper XML
    fn validate_mapper_xml(code: &str, intent: &SpringIntent) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for XML declaration
        if !code.contains("<?xml") {
            warnings.push("Note: Missing XML declaration".to_string());
        }

        // Check for MyBatis mapper namespace
        if !code.contains("<mapper") {
            warnings.push("Warning: Missing <mapper> element".to_string());
        }

        // Check for namespace matching Mapper interface
        let expected_namespace = format!("{}.mapper.{}", intent.package_base, intent.mapper_name());
        if !code.contains(&intent.mapper_name()) {
            warnings.push(format!("Warning: Namespace should reference {}", expected_namespace));
        }

        // Check for resultMap
        if !code.contains("<resultMap") && !code.contains("resultType") {
            warnings.push("Note: Consider defining a resultMap".to_string());
        }

        // Check for CRUD statements
        let crud_elements = [
            ("select", CrudOperation::Read),
            ("insert", CrudOperation::Create),
            ("update", CrudOperation::Update),
            ("delete", CrudOperation::Delete),
        ];

        for (element, op) in crud_elements {
            if intent.crud_operations.contains(&op) && !code.contains(&format!("<{}", element)) {
                warnings.push(format!("Warning: Missing <{}> for {:?} operation", element, op));
            }
        }

        // Check for table name
        if !code.contains(&intent.table_name) {
            warnings.push(format!("Warning: Table name '{}' not found in queries", intent.table_name));
        }

        // Check for parameterized queries (prevent SQL injection)
        if code.contains("${") {
            warnings.push("Warning: Found ${} placeholder - consider using #{} to prevent SQL injection".to_string());
        }

        Ok(warnings)
    }

    /// Get expected method name for a CRUD operation
    fn expected_method_name(op: &CrudOperation, entity_name: &str) -> String {
        match op {
            CrudOperation::Create => format!("create{}", entity_name),
            CrudOperation::Read => format!("get{}ById", entity_name),
            CrudOperation::ReadList => format!("get{}List", entity_name),
            CrudOperation::Update => format!("update{}", entity_name),
            CrudOperation::Delete => format!("delete{}", entity_name),
        }
    }

    /// Get expected mapper method name
    fn expected_mapper_method(op: &CrudOperation) -> &'static str {
        match op {
            CrudOperation::Create => "insert",
            CrudOperation::Read => "selectById",
            CrudOperation::ReadList => "selectList",
            CrudOperation::Update => "update",
            CrudOperation::Delete => "delete",
        }
    }

    /// Post-process the output to fix common issues
    pub fn post_process(artifacts: &mut SpringArtifacts, intent: &SpringIntent) {
        // Add missing imports if detected
        Self::add_missing_imports(&mut artifacts.controller);
        Self::add_missing_imports(&mut artifacts.service_impl);
        Self::add_missing_imports(&mut artifacts.dto);

        // Add warning if no primary key defined
        if intent.primary_key_columns().is_empty() {
            artifacts.warnings.push("Warning: No primary key column defined".to_string());
        }
    }

    /// Add common missing imports
    fn add_missing_imports(code: &mut String) {
        // Check for annotations without imports
        let import_mappings = [
            ("@RestController", "org.springframework.web.bind.annotation.RestController"),
            ("@Service", "org.springframework.stereotype.Service"),
            ("@Autowired", "org.springframework.beans.factory.annotation.Autowired"),
            ("@NotNull", "javax.validation.constraints.NotNull"),
            ("@Valid", "javax.validation.Valid"),
            ("LocalDate", "java.time.LocalDate"),
            ("LocalDateTime", "java.time.LocalDateTime"),
            ("BigDecimal", "java.math.BigDecimal"),
        ];

        for (annotation, import) in import_mappings {
            if code.contains(annotation) && !code.contains(import) {
                // Add a note about missing import
                if !code.contains(&format!("import {};", import)) {
                    // Could add import here, but for now just let the warning system handle it
                }
            }
        }
    }
}

/// Intermediate structure for parsed sections
struct ParsedSections {
    controller: String,
    service_interface: String,
    service_impl: String,
    dto: String,
    search_dto: Option<String>,
    mapper_interface: String,
    mapper_xml: String,
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
            )
            .with_column(
                ColumnIntent::new("email", "이메일")
                    .with_ui_type(UiType::Input)
                    .with_data_type(DataType::String)
            )
    }

    #[test]
    fn test_validate_controller() {
        let intent = create_test_intent();
        let controller = r#"
@RestController
@RequestMapping("/api/member")
public class MemberController {
    @Autowired
    private MemberService memberService;

    @GetMapping("/{id}")
    public MemberDTO getMember(@PathVariable Long id) {
        return memberService.getMemberById(id);
    }

    @PostMapping
    public void createMember(@RequestBody MemberDTO dto) {
        memberService.createMember(dto);
    }

    @PutMapping("/{id}")
    public void updateMember(@PathVariable Long id, @RequestBody MemberDTO dto) {
        memberService.updateMember(dto);
    }

    @DeleteMapping("/{id}")
    public void deleteMember(@PathVariable Long id) {
        memberService.deleteMember(id);
    }

    @GetMapping
    public List<MemberDTO> getMemberList() {
        return memberService.getMemberList();
    }
}
"#;

        let warnings = SpringValidator::validate_controller(controller, &intent).unwrap();
        // Should have no critical warnings for a complete controller
        assert!(warnings.iter().all(|w| w.starts_with("Note:")));
    }

    #[test]
    fn test_validate_controller_missing_annotations() {
        let intent = create_test_intent();
        let controller = "public class MemberController {}";

        let warnings = SpringValidator::validate_controller(controller, &intent).unwrap();
        assert!(warnings.iter().any(|w| w.contains("@RestController")));
        assert!(warnings.iter().any(|w| w.contains("@RequestMapping")));
    }

    #[test]
    fn test_validate_dto() {
        let intent = create_test_intent();
        let dto = r#"
@Data
public class MemberDTO {
    private Long memberId;
    @NotBlank
    private String memberName;
    private String email;
}
"#;

        let warnings = SpringValidator::validate_dto(dto, &intent).unwrap();
        // Should find all fields
        assert!(!warnings.iter().any(|w| w.contains("Field") && w.contains("not found")));
    }

    #[test]
    fn test_validate_mapper_xml() {
        let intent = create_test_intent();
        let mapper_xml = r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE mapper PUBLIC "-//mybatis.org//DTD Mapper 3.0//EN" "http://mybatis.org/dtd/mybatis-3-mapper.dtd">
<mapper namespace="com.company.project.mapper.MemberMapper">
    <resultMap id="MemberResultMap" type="com.company.project.dto.MemberDTO">
        <id property="memberId" column="MEMBER_ID"/>
        <result property="memberName" column="MEMBER_NAME"/>
        <result property="email" column="EMAIL"/>
    </resultMap>

    <select id="selectById" parameterType="long" resultMap="MemberResultMap">
        SELECT * FROM TB_MEMBER WHERE MEMBER_ID = #{memberId}
    </select>

    <insert id="insert" parameterType="MemberDTO">
        INSERT INTO TB_MEMBER (MEMBER_NAME, EMAIL)
        VALUES (#{memberName}, #{email})
    </insert>

    <update id="update" parameterType="MemberDTO">
        UPDATE TB_MEMBER SET MEMBER_NAME = #{memberName}, EMAIL = #{email}
        WHERE MEMBER_ID = #{memberId}
    </update>

    <delete id="delete" parameterType="long">
        DELETE FROM TB_MEMBER WHERE MEMBER_ID = #{memberId}
    </delete>
</mapper>
"#;

        let warnings = SpringValidator::validate_mapper_xml(mapper_xml, &intent).unwrap();
        // Should not have critical warnings for a complete mapper
        assert!(!warnings.iter().any(|w| w.contains("SQL injection")));
    }

    #[test]
    fn test_validate_mapper_xml_sql_injection() {
        let intent = create_test_intent();
        let mapper_xml = r#"
<mapper namespace="com.company.project.mapper.MemberMapper">
    <select id="selectByName">
        SELECT * FROM TB_MEMBER WHERE MEMBER_NAME = '${memberName}'
    </select>
</mapper>
"#;

        let warnings = SpringValidator::validate_mapper_xml(mapper_xml, &intent).unwrap();
        assert!(warnings.iter().any(|w| w.contains("SQL injection")));
    }

    #[test]
    fn test_expected_method_names() {
        assert_eq!(
            SpringValidator::expected_method_name(&CrudOperation::Create, "Member"),
            "createMember"
        );
        assert_eq!(
            SpringValidator::expected_method_name(&CrudOperation::Read, "Member"),
            "getMemberById"
        );
        assert_eq!(
            SpringValidator::expected_method_name(&CrudOperation::ReadList, "Member"),
            "getMemberList"
        );
    }
}
