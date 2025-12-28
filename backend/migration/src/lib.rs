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
            // inject-above (do not remove this comment)
        ]
    }
}