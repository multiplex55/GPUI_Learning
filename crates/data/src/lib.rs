#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, missing_docs, unreachable_pub)]

use chrono::{DateTime, Duration, Utc};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents a mock transaction record used by dashboards and list demos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// Unique identifier used for optimistic updates.
    pub id: u64,
    /// Display name for the account or counterparty.
    pub account: String,
    /// Category used for grouping and filtering in UI widgets.
    pub category: TransactionCategory,
    /// Amount recorded in the base currency.
    pub amount: f64,
    /// ISO-8601 timestamp of when the transaction occurred.
    pub occurred_at: DateTime<Utc>,
    /// Workflow state of the record.
    pub status: TransactionStatus,
}

/// Broad transaction buckets for demo data generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionCategory {
    /// Transactions that reflect marketplace fees or merchant revenue.
    Commerce,
    /// Cloud hosting and platform infrastructure expenses.
    Infrastructure,
    /// Campaign and advertising spend tracked by marketing teams.
    Marketing,
    /// Compensation entries for payroll processing.
    Payroll,
    /// Miscellaneous adjustments that don't fall into a core bucket.
    Misc,
}

/// Simplified approval workflow used by demos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    /// The transaction has been created but not yet settled.
    Pending,
    /// The payment has cleared and been reconciled.
    Settled,
    /// Requires analyst review because automated checks raised a flag.
    Flagged,
}

impl Transaction {
    /// Convenience method for computing whether the transaction represents an
    /// expense.
    #[must_use]
    pub fn is_expense(&self) -> bool {
        self.amount < 0.0
    }
}

/// Generates a deterministic catalogue of transactions.
#[must_use]
pub fn generate_transactions(count: usize) -> Vec<Transaction> {
    TransactionFactory::default().take(count).collect()
}

/// Iterator that yields reproducible transaction fixtures.
#[derive(Debug, Clone)]
pub struct TransactionFactory {
    rng: StdRng,
    next_id: u64,
}

impl Default for TransactionFactory {
    fn default() -> Self {
        Self {
            rng: StdRng::seed_from_u64(42),
            next_id: 1,
        }
    }
}

impl Iterator for TransactionFactory {
    type Item = Transaction;

    fn next(&mut self) -> Option<Self::Item> {
        let account = match self.rng.gen_range(0..=4) {
            0 => "Acme Cloud".to_owned(),
            1 => "Marketing Partners".to_owned(),
            2 => "Payroll".to_owned(),
            3 => "Data Warehouse".to_owned(),
            _ => "Misc Vendor".to_owned(),
        };

        let category = match self.rng.gen_range(0..=4) {
            0 => TransactionCategory::Infrastructure,
            1 => TransactionCategory::Marketing,
            2 => TransactionCategory::Payroll,
            3 => TransactionCategory::Commerce,
            _ => TransactionCategory::Misc,
        };

        let status = match self.rng.gen_range(0..=10) {
            0 => TransactionStatus::Flagged,
            1..=3 => TransactionStatus::Pending,
            _ => TransactionStatus::Settled,
        };

        let amount: f64 = match category {
            TransactionCategory::Commerce => self.rng.gen_range(40.0..200.0),
            TransactionCategory::Marketing => -self.rng.gen_range(120.0..750.0),
            TransactionCategory::Payroll => -self.rng.gen_range(2_000.0..8_500.0),
            TransactionCategory::Infrastructure => -self.rng.gen_range(300.0..2_000.0),
            TransactionCategory::Misc => self.rng.gen_range(-150.0..250.0),
        };

        let occurred_at =
            Utc::now() - Duration::minutes(self.rng.gen_range(0..(60 * 24 * 30)) as i64);

        let txn = Transaction {
            id: self.next_id,
            account,
            category,
            amount: (amount * 100.0).round() / 100.0,
            occurred_at,
            status,
        };
        self.next_id += 1;
        Some(txn)
    }
}

/// Error type surfaced by asynchronous loaders.
#[derive(Debug, Error)]
pub enum LoaderError {
    /// Wrapper around JSON serialization failures.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Serializes generated transactions to JSON for use in demos.
pub fn export_transactions(count: usize) -> Result<String, LoaderError> {
    let txns = generate_transactions(count);
    Ok(serde_json::to_string_pretty(&txns)?)
}

/// Utilities for measuring virtualized list performance.
#[derive(Debug, Clone)]
pub struct VirtualListBenchmark {
    /// Total rows in the backing data set.
    pub total_rows: usize,
    /// Height of each row in logical pixels.
    pub row_height: f32,
    /// Height of the viewport in logical pixels.
    pub viewport_height: f32,
}

impl VirtualListBenchmark {
    /// Estimates the amount of data that needs to be drawn per frame.
    #[must_use]
    pub fn rows_per_viewport(&self) -> usize {
        (self.viewport_height / self.row_height).ceil() as usize
    }

    /// Calculates the number of buffered rows recommended for smooth scrolling.
    #[must_use]
    pub fn suggested_buffer(&self) -> usize {
        (self.rows_per_viewport() as f32 * 0.5).ceil() as usize
    }

    /// Returns a projected render cost assuming a simple O(n) renderer.
    #[must_use]
    pub fn estimated_render_cost(&self) -> usize {
        let buffer = self.suggested_buffer();
        (self.rows_per_viewport() + buffer).min(self.total_rows)
    }
}

/// Asynchronously loads transactions after an artificial delay.
#[cfg(feature = "async-loaders")]
pub async fn load_transactions_async(count: usize) -> Result<Vec<Transaction>, LoaderError> {
    use tokio::time::{sleep, Duration};

    sleep(Duration::from_millis(120)).await;
    let payload = generate_transactions(count);
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_produces_consistent_amounts() {
        let data = generate_transactions(5);
        assert_eq!(data.len(), 5);
        assert!(data.iter().any(Transaction::is_expense));
    }

    #[test]
    fn export_serializes_transactions() {
        let json = export_transactions(2).unwrap();
        assert!(json.contains("account"));
    }

    #[test]
    fn virtual_list_helpers() {
        let bench = VirtualListBenchmark {
            total_rows: 10_000,
            row_height: 32.0,
            viewport_height: 480.0,
        };
        assert!(bench.estimated_render_cost() >= bench.rows_per_viewport());
    }
}
