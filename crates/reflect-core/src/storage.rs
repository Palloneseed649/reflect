use async_trait::async_trait;
use uuid::Uuid;

use crate::error::Result;
use crate::types::{ErrorPattern, Reflection, ReflectionStats, ScoredReflection};

#[async_trait]
pub trait Storage: Send + Sync {
    async fn store_reflection(&self, reflection: &Reflection) -> Result<()>;
    async fn get_reflection(&self, id: &Uuid) -> Result<Option<Reflection>>;
    async fn delete_reflection(&self, id: &Uuid) -> Result<bool>;
    async fn search_reflections(
        &self,
        query: &str,
        tags: &[String],
        limit: usize,
    ) -> Result<Vec<ScoredReflection>>;
    async fn upsert_pattern(&self, pattern: &ErrorPattern) -> Result<()>;
    async fn get_pattern(&self, id: &str) -> Result<Option<ErrorPattern>>;
    async fn list_patterns(&self, min_occurrences: u32, limit: usize) -> Result<Vec<ErrorPattern>>;
    async fn get_stats(&self) -> Result<ReflectionStats>;
}
