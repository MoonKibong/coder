#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

// Schema migrations
mod m20220101_000001_users;
mod m20251228_002114_prompt_templates;
mod m20251228_002511_company_rules;
mod m20251228_002936_generation_logs;
mod m20251228_003221_llm_configs;
mod m20251228_003509_add_indexes;
mod m20251228_050000_add_job_queue_fields;
mod m20251228_125645_knowledge_bases;
mod m20251230_120000_add_local_llm_columns;
mod m20251230_140000_remove_company_id_from_company_rules;
mod m20251230_150000_add_llm_info_to_generation_logs;

// NOTE: Seed migrations removed - use fixtures instead:
//   cargo loco db seed --reset
// Fixture files: src/fixtures/*.yaml

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
            Box::new(m20251228_050000_add_job_queue_fields::Migration),
            Box::new(m20251228_125645_knowledge_bases::Migration),
            Box::new(m20251230_120000_add_local_llm_columns::Migration),
            Box::new(m20251230_140000_remove_company_id_from_company_rules::Migration),
            Box::new(m20251230_150000_add_llm_info_to_generation_logs::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}