# xFrame5 Validation Pattern

## 목적
LLM 출력이 유효한 xFrame5 코드인지 검증하여 품질 보장

## 검증 단계

```
LLM Output → Parse → Validate Structure → Validate Bindings → Result
```

## XML 검증

### 1. Parse 검증
```rust
pub fn validate_xml(xml: &str) -> Result<XmlDocument, ValidationError> {
    // 기본 XML 파싱
    let doc = quick_xml::parse(xml)?;
    Ok(doc)
}
```

### 2. 구조 검증
```rust
pub fn validate_structure(doc: &XmlDocument) -> Result<(), Vec<ValidationError>> {
    let mut errors = vec![];

    // Dataset 필수 요소
    if !has_element(doc, "Dataset") {
        errors.push(ValidationError::MissingElement("Dataset"));
    }

    // Grid 필수 속성
    for grid in find_elements(doc, "Grid") {
        if !has_attr(&grid, "id") {
            errors.push(ValidationError::MissingAttribute("Grid", "id"));
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

### 3. 바인딩 검증
```rust
pub fn validate_bindings(doc: &XmlDocument) -> Result<(), Vec<ValidationError>> {
    let datasets: HashSet<_> = find_elements(doc, "Dataset")
        .filter_map(|d| get_attr(&d, "id"))
        .collect();

    let mut errors = vec![];

    // Grid → Dataset 바인딩 확인
    for grid in find_elements(doc, "Grid") {
        if let Some(ds_id) = get_attr(&grid, "dataset") {
            if !datasets.contains(&ds_id) {
                errors.push(ValidationError::InvalidBinding(
                    format!("Grid references undefined Dataset: {}", ds_id)
                ));
            }
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

## JavaScript 검증

### 1. 함수 존재 확인
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

### 2. 문법 검증 (Optional)
- 기본적인 구문 오류 확인
- 금융권 환경에서는 엄격 모드 사용

## 실패 대응

### 자동 수정 가능한 경우
```rust
pub fn auto_fix(artifact: &mut Artifact) {
    // 빠진 TODO 주석 추가
    // 함수명 케이스 수정
    // 공백/인덴트 정규화
}
```

### 재시도 필요한 경우
```rust
pub enum RetryStrategy {
    SimplifyPrompt,    // 프롬프트 간소화
    SplitRequest,      // 요청 분할 (XML만 / JS만)
    Fail,              // 에러 반환
}
```

## 검증 결과

```rust
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub auto_fixed: Vec<String>,
}
```

## strictMode 옵션

```rust
// options.strictMode = true 일 때
pub fn strict_validate(artifact: &Artifact) -> Result<(), Vec<ValidationError>> {
    // 추가 검증: 주석 필수, 네이밍 규칙 등
}
```
