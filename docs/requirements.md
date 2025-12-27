# xFrame5 Code Assistant - Requirements

## 1. 프로젝트 개요

### 1.1 목표
xFrame5 프론트엔드 개발 자동화 PoC - 기존 개발 프로세스를 바꾸지 않는 코드 생성 도구

### 1.2 핵심 원칙
- "AI", "지능형", "Copilot" 용어 의도적 배제
- "자동화 + 기존 프로세스 유지" 강조
- 기존 개발 방식, 코딩 규칙, 리뷰 프로세스 변경 없음

---

## 2. 문제 정의

### 2.1 현재 상황
- xFrame5 화면 개발 구성:
  - XML 화면 정의
  - Dataset 매핑
  - 이벤트 JS 작성
- 반복 작업 비중 높음
- 숙련자 의존도 높음
- 복사/붙여넣기 실수 빈번

### 2.2 현장의 불편
- "화면 하나 만드는데 왜 이렇게 시간이 걸리지?"
- "비슷한 화면인데 매번 처음부터..."

---

## 3. PoC 범위

### 3.1 포함
- 회원 목록 화면
- 조회 기능
- 상세 팝업

### 3.2 제외
- 백엔드 생성
- 전면 자동화
- 런타임 개입

> **PoC는 작아야 승인된다**

---

## 4. PoC 시나리오

### 4.1 입력
- DB 쿼리 결과 (컬럼 + 샘플 데이터)
- 간단한 화면 설명 (체크박스 기반)

### 4.2 처리
- 내부 규칙 기반 UI 모델 생성
- 회사 표준 템플릿 적용

### 4.3 출력
- xFrame5 XML 화면 파일
- JavaScript 이벤트 파일

### 4.4 개발자 작업
- IDE 내에서 결과 확인
- 필요 시 구조 수정
- 파일 생성 → 커밋

---

## 5. 성공 기준

| 항목 | 기준 |
|------|------|
| 화면 골격 생성 | 5분 이내 |
| 수동 수정 | 기존 대비 50% 이하 |
| 생성 코드 품질 | 기존 코드 리뷰 통과 |
| 외부 전송 | 없음 |

---

## 6. 기술 스택

### 6.1 Agent Server (Backend)
- **Framework**: Loco.rs (Rust)
- **Database**: PostgreSQL
- **LLM Runtime**: Ollama / llama.cpp (설정 기반 선택)

### 6.2 Eclipse Plugin (Frontend)
- **Platform**: Eclipse PDE
- **Language**: Java

### 6.3 Infrastructure
- **Deployment**: Docker 또는 Native 설치
- **Network**: 사내망 전용 (인터넷 차단 환경 가능)

---

## 7. 아키텍처 요구사항

### 7.1 LLM 추상화
- LLM은 보조적 역할
- 입력: DB 스키마, 쿼리 결과
- 결과: 규칙 엔진이 최종 결정

> "대규모 언어 모델은 화면 의도 해석 보조 용도로만 사용되며, 생성 결과는 사내 규칙 엔진을 통해 확정됩니다."

### 7.2 배포 옵션

**옵션 A (추천)**:
- 고객 VM에 Docker
- 모든 컴포넌트 로컬

**옵션 B**:
- 고객 서버에 Native 설치
- LLM 선택적 비활성화 가능

### 7.3 로그 및 감사

| 항목 | 저장 여부 |
|------|----------|
| 입력 데이터 | ❌ |
| Meta Model | ⭕ |
| 생성 결과 | ⭕ |
| 생성자/시간 | ⭕ |

> "데이터는 남기지 않고, 결과와 구조만 남깁니다."

---

## 8. Eclipse Plugin 요구사항

### 8.1 설계 원칙
플러그인은 의도적으로 "무지(無知)"해야 함

### 8.2 플러그인이 아는 것
- 입력 타입 (자연어 / 스키마 / 샘플 데이터)
- 프로젝트 컨텍스트 (선택된 파일, 패키지, 경로)
- Agent Server 엔드포인트 URL

### 8.3 플러그인이 절대 몰라야 하는 것
- 모델 이름
- 프롬프트 구조
- Ollama 존재 여부
- Token / Temperature / System Prompt

### 8.4 요청 형식
```json
{
  "product": "xframe5-ui",
  "inputType": "db-schema",
  "input": {
    "table": "CUSTOMER",
    "columns": [
      {"name": "CUST_ID", "type": "string"},
      {"name": "CUST_NAME", "type": "string"}
    ]
  },
  "context": {
    "projectType": "xframe5",
    "language": "javascript"
  }
}
```

---

## 9. Agent Server 요구사항

### 9.1 처리 단계

1. **요청 정규화 (Normalize)**
   - 다양한 입력을 Canonical Form으로 변환

2. **프롬프트 변환 (Prompt Compiler)**
   - 요청(JSON) → 내부 DSL → Prompt Template → LLM Prompt
   - xFrame5 전용 system prompt는 서버에만 존재

3. **LLM Runtime 호출**
   - LlmBackend trait 기반 추상화
   - 구현체 교체 가능 (Ollama, llama.cpp, 사내 GPU)

4. **결과 파싱 및 검증**
   - XML / JS 분리
   - xFrame5 문법 검증
   - 실패 시 재시도 또는 에러 리포트

5. **결과 반환**
   ```json
   {
     "status": "success",
     "artifacts": {
       "xml": "<Dataset id=...>",
       "javascript": "this.fn_search = function() {...}"
     },
     "warnings": ["API endpoint not defined yet"]
   }
   ```

---

## 10. 비기능 요구사항

### 10.1 보안
- 외부 통신 없음
- 인터넷 차단 환경 지원
- 입력 데이터 저장 금지

### 10.2 성능
- 화면 생성 5분 이내

### 10.3 확장성
- 고객사별 규칙 주입 가능
- 제품 분리 가능 (코드 생성 / 코드 리뷰)

### 10.4 운영
- LLM 없이도 동작 가능 (Rule-based fallback)
- 모델 교체 시 플러그인 변경 불필요

---

## 11. 검증 기준

### 11.1 아키텍처 검증
> "모델을 바꿨는데 Eclipse 플러그인은 단 한 줄도 안 바뀐다"

### 11.2 품질 검증
- 생성된 코드가 기존 코드 리뷰 통과
- xFrame5 런타임에서 정상 동작
