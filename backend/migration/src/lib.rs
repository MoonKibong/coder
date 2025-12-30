#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20251228_002114_prompt_templates;
mod m20251228_002511_company_rules;
mod m20251228_002936_generation_logs;
mod m20251228_003221_llm_configs;
mod m20251228_003509_add_indexes;
mod m20251228_041600_seed_spring_prompt_templates;
mod m20251228_050000_add_job_queue_fields;
mod m20251228_104804_seed_xframe5_prompt_templates;
mod m20251228_125645_knowledge_bases;
mod m20251229_113109_seed_xframe5_prompt_templates_v3;
mod m20251230_005747_seed_review_prompt_templates;
mod m20251230_012654_seed_qa_prompt_templates;
mod m20251230_120000_add_local_llm_columns;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20251228_002114_prompt_templates::Migration),
            Box::new(m20251228_002511_company_rules::Migration),
            Box::new(m20251228_002936_generation_logs::Migration),
            Box::new(m20251228_003221_llm_configs::Migration),
            Box::new(m20251228_003509_add_indexes::Migration),
            Box::new(m20251228_041600_seed_spring_prompt_templates::Migration),
            Box::new(m20251228_050000_add_job_queue_fields::Migration),
            Box::new(m20251228_104804_seed_xframe5_prompt_templates::Migration),
            Box::new(m20251228_125645_knowledge_bases::Migration),
            Box::new(m20251229_113109_seed_xframe5_prompt_templates_v3::Migration),
            Box::new(m20251230_005747_seed_review_prompt_templates::Migration),
            Box::new(m20251230_012654_seed_qa_prompt_templates::Migration),
            Box::new(m20251230_120000_add_local_llm_columns::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}