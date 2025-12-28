//! Analytics Service
//!
//! Provides statistics and analytics for generation requests.

use chrono::{DateTime, Duration, FixedOffset, Utc};
use loco_rs::prelude::*;
use sea_orm::{
    query::*, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QuerySelect,
};
use serde::Serialize;

use crate::models::_entities::generation_logs::{Column, Entity};

/// Generation statistics
#[derive(Debug, Serialize)]
pub struct GenerationStats {
    /// Total number of generation requests
    pub total_requests: u64,
    /// Successful generations
    pub success_count: u64,
    /// Failed generations
    pub failure_count: u64,
    /// Success rate percentage (0-100)
    pub success_rate: f32,
    /// Average generation time in milliseconds
    pub avg_generation_time_ms: f32,
    /// Requests in the last 24 hours
    pub requests_last_24h: u64,
    /// Requests in the last 7 days
    pub requests_last_7d: u64,
    /// Requests in the last 30 days
    pub requests_last_30d: u64,
}

/// Statistics by category (product or input type)
#[derive(Debug, Serialize)]
pub struct CategoryStats {
    pub category: String,
    pub count: u64,
    pub percentage: f32,
}

/// Time-series data point for charts
#[derive(Debug, Serialize)]
pub struct TimeSeriesPoint {
    pub label: String,
    pub value: u64,
}

/// Complete analytics data for dashboard
#[derive(Debug, Serialize)]
pub struct DashboardAnalytics {
    pub generation_stats: GenerationStats,
    pub by_product: Vec<CategoryStats>,
    pub by_input_type: Vec<CategoryStats>,
    pub by_status: Vec<CategoryStats>,
    pub requests_by_day: Vec<TimeSeriesPoint>,
    pub recent_activity: RecentActivity,
}

/// Recent activity summary
#[derive(Debug, Serialize)]
pub struct RecentActivity {
    pub last_generation_at: Option<DateTime<FixedOffset>>,
    pub active_users_24h: u64,
}

pub struct AnalyticsService;

impl AnalyticsService {
    /// Get complete dashboard analytics
    pub async fn get_dashboard_analytics(db: &DatabaseConnection) -> Result<DashboardAnalytics> {
        let generation_stats = Self::get_generation_stats(db).await?;
        let by_product = Self::get_stats_by_product(db).await?;
        let by_input_type = Self::get_stats_by_input_type(db).await?;
        let by_status = Self::get_stats_by_status(db).await?;
        let requests_by_day = Self::get_requests_by_day(db, 7).await?;
        let recent_activity = Self::get_recent_activity(db).await?;

        Ok(DashboardAnalytics {
            generation_stats,
            by_product,
            by_input_type,
            by_status,
            requests_by_day,
            recent_activity,
        })
    }

    /// Get overall generation statistics
    pub async fn get_generation_stats(db: &DatabaseConnection) -> Result<GenerationStats> {
        let total_requests = Entity::find().count(db).await.unwrap_or(0);

        let success_count = Entity::find()
            .filter(Column::Status.eq("success"))
            .count(db)
            .await
            .unwrap_or(0);

        let failure_count = Entity::find()
            .filter(Column::Status.eq("error"))
            .count(db)
            .await
            .unwrap_or(0);

        let success_rate = if total_requests > 0 {
            (success_count as f32 / total_requests as f32) * 100.0
        } else {
            0.0
        };

        // Calculate average generation time
        let avg_generation_time_ms = Self::calculate_avg_generation_time(db).await?;

        // Time-based counts
        let now = Utc::now();
        let last_24h = now - Duration::hours(24);
        let last_7d = now - Duration::days(7);
        let last_30d = now - Duration::days(30);

        let requests_last_24h = Entity::find()
            .filter(Column::CreatedAt.gte(last_24h))
            .count(db)
            .await
            .unwrap_or(0);

        let requests_last_7d = Entity::find()
            .filter(Column::CreatedAt.gte(last_7d))
            .count(db)
            .await
            .unwrap_or(0);

        let requests_last_30d = Entity::find()
            .filter(Column::CreatedAt.gte(last_30d))
            .count(db)
            .await
            .unwrap_or(0);

        Ok(GenerationStats {
            total_requests,
            success_count,
            failure_count,
            success_rate,
            avg_generation_time_ms,
            requests_last_24h,
            requests_last_7d,
            requests_last_30d,
        })
    }

    async fn calculate_avg_generation_time(db: &DatabaseConnection) -> Result<f32> {
        // Get all generation times and calculate average in Rust
        // (More portable than DB-specific AVG function)
        let logs = Entity::find()
            .filter(Column::GenerationTimeMs.is_not_null())
            .select_only()
            .column(Column::GenerationTimeMs)
            .into_tuple::<Option<i32>>()
            .all(db)
            .await?;

        if logs.is_empty() {
            return Ok(0.0);
        }

        let sum: i64 = logs.iter().filter_map(|t| t.map(|v| v as i64)).sum();
        let count = logs.iter().filter(|t| t.is_some()).count();

        if count == 0 {
            Ok(0.0)
        } else {
            Ok(sum as f32 / count as f32)
        }
    }

    /// Get statistics grouped by product
    pub async fn get_stats_by_product(db: &DatabaseConnection) -> Result<Vec<CategoryStats>> {
        Self::get_category_stats(db, Column::Product).await
    }

    /// Get statistics grouped by input type
    pub async fn get_stats_by_input_type(db: &DatabaseConnection) -> Result<Vec<CategoryStats>> {
        Self::get_category_stats(db, Column::InputType).await
    }

    /// Get statistics grouped by status
    pub async fn get_stats_by_status(db: &DatabaseConnection) -> Result<Vec<CategoryStats>> {
        Self::get_category_stats(db, Column::Status).await
    }

    async fn get_category_stats(
        db: &DatabaseConnection,
        column: Column,
    ) -> Result<Vec<CategoryStats>> {
        // Get total count
        let total = Entity::find().count(db).await.unwrap_or(0) as f32;

        // Get all records and group in Rust for simplicity
        // Use into_tuple to only select the one column we need
        let logs: Vec<String> = Entity::find()
            .select_only()
            .column(column)
            .into_tuple::<String>()
            .all(db)
            .await?;

        let mut counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

        for category in logs {
            *counts.entry(category).or_insert(0) += 1;
        }

        let mut stats: Vec<CategoryStats> = counts
            .into_iter()
            .map(|(category, count)| {
                let percentage = if total > 0.0 {
                    (count as f32 / total) * 100.0
                } else {
                    0.0
                };
                CategoryStats {
                    category,
                    count,
                    percentage,
                }
            })
            .collect();

        // Sort by count descending
        stats.sort_by(|a, b| b.count.cmp(&a.count));

        Ok(stats)
    }

    /// Get requests per day for the last N days
    pub async fn get_requests_by_day(
        db: &DatabaseConnection,
        days: i64,
    ) -> Result<Vec<TimeSeriesPoint>> {
        let now = Utc::now();
        let mut results = Vec::new();

        for i in (0..days).rev() {
            let day_start = (now - Duration::days(i))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let day_end = (now - Duration::days(i))
                .date_naive()
                .and_hms_opt(23, 59, 59)
                .unwrap();

            let count = Entity::find()
                .filter(Column::CreatedAt.gte(day_start))
                .filter(Column::CreatedAt.lte(day_end))
                .count(db)
                .await
                .unwrap_or(0);

            let label = (now - Duration::days(i)).format("%m/%d").to_string();

            results.push(TimeSeriesPoint {
                label,
                value: count,
            });
        }

        Ok(results)
    }

    /// Get recent activity summary
    pub async fn get_recent_activity(db: &DatabaseConnection) -> Result<RecentActivity> {
        // Get the most recent generation
        let last_log = Entity::find()
            .order_by_desc(Column::CreatedAt)
            .one(db)
            .await?;

        let last_generation_at = last_log.map(|l| l.created_at);

        // Count distinct users in last 24 hours
        let now = Utc::now();
        let last_24h = now - Duration::hours(24);

        let logs = Entity::find()
            .filter(Column::CreatedAt.gte(last_24h))
            .select_only()
            .column(Column::UserId)
            .into_tuple::<i32>()
            .all(db)
            .await?;

        let active_users_24h = logs
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .len() as u64;

        Ok(RecentActivity {
            last_generation_at,
            active_users_24h,
        })
    }
}
