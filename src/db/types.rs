use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Memory {
    pub embedding: Embedding,
    pub data: MemoryData,
}

#[derive(Debug)]
pub struct Embedding {
    pub id: u128,
    pub data: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryData {
    pub id: u128,
    pub score: u16,
    pub content: String,
}

impl Embedding {
    pub fn new(id: u128, data: Vec<f32>) -> Self {
        Self { id, data }
    }

    pub fn cosine_similarity(&self, rhs: &Embedding) -> f32 {
        self.dot(rhs) / (self.l2_norm() * rhs.l2_norm())
    }

    pub fn dot(&self, rhs: &Embedding) -> f32 {
        // We assume that the dimensions always match.
        // The embeddings returned from the OpenAI API always have the same dimensions.
        self.data
            .iter()
            .zip(rhs.data.iter())
            .map(|(x, y)| x * y)
            .sum::<f32>()
            .sqrt()
    }

    pub fn l2_norm(&self) -> f32 {
        self.data.iter().map(|x| x.powf(2.0)).sum::<f32>().sqrt()
    }
}

impl Display for MemoryData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A tweet from us: {}", self.content)
    }
}
