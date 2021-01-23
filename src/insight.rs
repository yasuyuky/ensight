use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Insights {
    pub items: Vec<InsightItem>,
}

#[derive(Debug, Deserialize)]
pub struct InsightItem {
    pub name: String,
    pub metrics: Metrics,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct Metrics {
    pub success_rate: f64,
    pub total_runs: usize,
    pub failed_runs: usize,
    pub successful_runs: usize,
    pub throughput: f64,
    pub duration_metrics: DurationMetrics,
    pub total_credits_used: usize,
}

#[derive(Debug, Deserialize)]
pub struct DurationMetrics {
    pub min: usize,
    pub max: usize,
    pub median: usize,
    pub mean: usize,
    pub p95: usize,
    pub standard_deviation: f64,
}

#[derive(Debug, Deserialize)]
pub struct Items {
    pub items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub id: Option<String>,
    // pub created_at: DateTime<Utc>,
    // pub stopped_at: DateTime<Utc>,
    pub duration: usize,
    pub status: Option<String>,
    pub credits_used: usize,
}
