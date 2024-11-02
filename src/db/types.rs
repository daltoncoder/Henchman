#[derive(Debug)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub embedding: Embedding,
}

#[derive(Debug)]
pub struct Embedding {
    pub data: Vec<f32>,
}

impl Embedding {
    pub fn new(data: Vec<f32>) -> Self {
        Self { data }
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
