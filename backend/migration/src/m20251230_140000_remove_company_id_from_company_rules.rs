use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Add name column (before dropping company_id)
        m.alter_table(
            Table::alter()
                .table(Alias::new("company_rules"))
                .add_column(
                    ColumnDef::new(Alias::new("name"))
                        .string()
                        .not_null()
                        .default("Default Rules"),
                )
                .to_owned(),
        )
        .await?;

        // Copy company_id values to name for existing records
        m.get_connection()
            .execute_unprepared("UPDATE company_rules SET name = company_id")
            .await?;

        // Drop company_id column
        m.alter_table(
            Table::alter()
                .table(Alias::new("company_rules"))
                .drop_column(Alias::new("company_id"))
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Add company_id column back
        m.alter_table(
            Table::alter()
                .table(Alias::new("company_rules"))
                .add_column(
                    ColumnDef::new(Alias::new("company_id"))
                        .string()
                        .not_null()
                        .default("default"),
                )
                .to_owned(),
        )
        .await?;

        // Copy name values to company_id
        m.get_connection()
            .execute_unprepared("UPDATE company_rules SET company_id = name")
            .await?;

        // Drop name column
        m.alter_table(
            Table::alter()
                .table(Alias::new("company_rules"))
                .drop_column(Alias::new("name"))
                .to_owned(),
        )
        .await?;

        Ok(())
    }
}
