//! Admin Services
//!
//! Service layer for admin CRUD operations.
//! Implements the pagination pattern from HWS/docs/patterns/PAGINATION_PATTERN.md

pub mod prompt_template;
pub mod company_rule;
pub mod llm_config;
pub mod generation_log;
pub mod user;

pub use prompt_template::PromptTemplateService;
pub use company_rule::CompanyRuleService;
pub use llm_config::LlmConfigService;
pub use generation_log::GenerationLogService;
pub use user::UserService;
