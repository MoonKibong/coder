# Product Validation Pattern

## Purpose
Validate LLM output to ensure generated code meets product-specific quality standards.

## Validation Flow

```
LLM Output → Parse → Validate Structure → Product-Specific Rules → Result
```

## Validator Trait

```rust
pub trait ProductValidator: Send + Sync {
    fn product(&self) -> &str;
    fn validate(&self, artifacts: &Artifacts) -> ValidationResult;
    fn auto_fix(&self, artifacts: &mut Artifacts) -> Vec<String>;
}
```

## Spring Boot Validator

### Java Syntax Validation
```rust
impl ProductValidator for SpringValidator {
    fn validate(&self, artifacts: &Artifacts) -> ValidationResult {
        let mut errors = vec![];

        // Check Entity class
        if let Some(entity) = &artifacts.entity {
            if !entity.contains("@Entity") {
                errors.push("Entity missing @Entity annotation");
            }
            if !entity.contains("@Id") {
                errors.push("Entity missing @Id annotation");
            }
        }

        // Check Repository interface
        if let Some(repo) = &artifacts.repository {
            if !repo.contains("extends JpaRepository") {
                errors.push("Repository must extend JpaRepository");
            }
        }

        ValidationResult { is_valid: errors.is_empty(), errors, .. }
    }
}
```

## xFrame5 Validator

### XML Validation
```rust
impl ProductValidator for XFrame5Validator {
    fn validate(&self, artifacts: &Artifacts) -> ValidationResult {
        let mut errors = vec![];

        if let Some(xml) = &artifacts.xml {
            // Parse XML
            let doc = quick_xml::parse(xml)?;

            // Dataset required
            if !has_element(&doc, "Dataset") {
                errors.push("Missing Dataset element");
            }

            // Grid → Dataset binding
            for grid in find_elements(&doc, "Grid") {
                if let Some(ds_id) = get_attr(&grid, "dataset") {
                    if !datasets.contains(&ds_id) {
                        errors.push(format!("Grid references undefined Dataset: {}", ds_id));
                    }
                }
            }
        }

        ValidationResult { is_valid: errors.is_empty(), errors, .. }
    }
}
```

### JavaScript Validation
```rust
pub fn validate_js_functions(js: &str, intent: &UiIntent) -> Result<(), Vec<ValidationError>> {
    let mut errors = vec![];

    for action in &intent.actions {
        let fn_name = format!("fn_{}", action.name.to_lowercase());
        if !js.contains(&fn_name) {
            errors.push(ValidationError::MissingFunction(fn_name));
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

## Failure Handling

### Auto-fixable Issues
```rust
pub fn auto_fix(artifact: &mut Artifact) {
    // Add missing TODO comments
    // Fix function name casing
    // Normalize whitespace/indentation
}
```

### Retry Strategy
```rust
pub enum RetryStrategy {
    SimplifyPrompt,    // Simplify the prompt
    SplitRequest,      // Split request (XML only / JS only)
    Fail,              // Return error
}
```

## Validation Result

```rust
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub auto_fixed: Vec<String>,
}
```

## Strict Mode

```rust
// When options.strictMode = true
pub fn strict_validate(artifact: &Artifact) -> Result<(), Vec<ValidationError>> {
    // Additional validation: comments required, naming conventions, etc.
}
```

## Adding New Product Validators

1. Implement `ProductValidator` trait
2. Register in validator factory
3. Add product-specific rules
4. Update prompt templates with validation hints
