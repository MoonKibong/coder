# Prompt Compiler Pattern

## 목적
Plugin 요청을 구조화된 DSL로 변환한 후, LLM용 프롬프트로 컴파일

## 변환 흐름

```
요청(JSON) → InputKind → UiIntent(DSL) → Template(DB) → LLM Prompt
```

## Internal DSL

### UiIntent
```rust
#[derive(Debug, Clone)]
pub struct UiIntent {
    pub screen_name: String,
    pub screen_type: ScreenType,
    pub datasets: Vec<DatasetIntent>,
    pub grids: Vec<GridIntent>,
    pub actions: Vec<ActionIntent>,
}

#[derive(Debug, Clone)]
pub enum ScreenType {
    List,
    Detail,
    Popup,
    ListWithPopup,
}
```

## Template Storage (Database)

### Schema
```sql
CREATE TABLE prompt_templates (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    product VARCHAR(50) NOT NULL,        -- 'xframe5-ui'
    screen_type VARCHAR(50),              -- 'list', 'popup', 'crud', NULL for default
    system_prompt TEXT NOT NULL,
    user_prompt_template TEXT NOT NULL,
    version INT NOT NULL DEFAULT 1,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE company_rules (
    id UUID PRIMARY KEY,
    company_id UUID NOT NULL,
    naming_convention JSONB,              -- {"prefix": "fn_", "case": "camelCase"}
    additional_rules TEXT,                -- Appended to RULES section
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_templates_product_type ON prompt_templates(product, screen_type, is_active);
```

### Template Structure
```rust
pub struct PromptTemplate {
    pub id: Uuid,
    pub name: String,
    pub product: String,
    pub screen_type: Option<String>,
    pub system_prompt: String,
    pub user_prompt_template: String,  // Contains {{dsl_description}} placeholder
    pub version: i32,
    pub is_active: bool,
}
```

### Default Template Data
```sql
INSERT INTO prompt_templates (name, product, screen_type, system_prompt, user_prompt_template) VALUES
('xframe5-ui-default', 'xframe5-ui', NULL,
'You are an expert xFrame5 frontend developer.

RULES:
- Generate ONLY xframe5 XML and JavaScript
- NO HTML, NO React, NO JSP
- Dataset and Grid MUST be properly bound
- Use transaction stub functions (fn_search, fn_save, etc.)
- If information is missing, add TODO comments
- Follow Korean coding conventions',

'{{dsl_description}}

OUTPUT FORMAT:
--- XML ---
[xFrame5 XML content here]
--- JS ---
[JavaScript content here]'
);
```

## Template Selection

```rust
pub async fn get_template(
    db: &DatabaseConnection,
    product: &str,
    screen_type: Option<&str>,
) -> Result<PromptTemplate> {
    // 1. Try specific screen_type match
    // 2. Fall back to product default (screen_type = NULL)
    // 3. Error if no template found
}
```

## 컴파일 과정

### 1. Load Template from DB
```rust
let template = get_template(&db, "xframe5-ui", Some("list")).await?;
```

### 2. Load Company Rules (if applicable)
```rust
let rules = get_company_rules(&db, company_id).await?;
```

### 3. DSL → Description
```rust
let description = describe_intent(&intent);
```

### 4. Compile Final Prompt
```rust
pub fn compile_prompt(
    template: &PromptTemplate,
    intent: &UiIntent,
    rules: Option<&CompanyRules>,
) -> CompiledPrompt {
    let mut system = template.system_prompt.clone();

    // Inject company-specific rules
    if let Some(r) = rules {
        system.push_str("\n\nCOMPANY RULES:\n");
        system.push_str(&r.additional_rules);
    }

    let user = template.user_prompt_template
        .replace("{{dsl_description}}", &describe_intent(intent));

    CompiledPrompt { system, user }
}
```

## 장점

1. **재배포 없이 템플릿 수정** - DB만 업데이트
2. **고객사별 커스터마이징** - company_rules 테이블
3. **버전 관리** - version 컬럼으로 롤백 가능
4. **A/B 테스트** - is_active 플래그로 전환

## 실패 대응

- Parse 실패 → 간소화 프롬프트 재시도 (max 2회)
- 정보 부족 → `TODO:` 주석 강제 삽입
