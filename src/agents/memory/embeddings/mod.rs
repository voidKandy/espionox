#[cfg(feature = "embedding_sql")]
pub mod vector_model;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingVector(Vec<f32>);

impl From<Vec<f32>> for EmbeddingVector {
    fn from(v: Vec<f32>) -> Self {
        EmbeddingVector(v)
    }
}

impl Into<Vec<f32>> for EmbeddingVector {
    fn into(self) -> Vec<f32> {
        self.0
    }
}

impl PartialEq for EmbeddingVector {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl EmbeddingVector {
    pub fn score_l2(&self, other: &Self) -> f32 {
        let sum_of_squares: f32 = self
            .0
            .iter()
            .zip(other.0.iter())
            .map(|(&x, &y)| (x - y).powi(2))
            .sum();

        sum_of_squares.sqrt()
    }
}
