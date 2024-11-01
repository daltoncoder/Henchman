#[derive(Debug)]
pub struct Memory {
    id: String,
    content: String,
    embedding: Embedding,
}

#[derive(Debug)]
pub struct Embedding {
    data: Vec<f64>,
}

impl Embedding {
    pub fn new(data: Vec<f64>) -> Self {
        Self { data }
    }

    pub fn cosine_similarity(&self, rhs: &Embedding) -> f64 {
        self.dot(rhs) / (self.l2_norm() * rhs.l2_norm())
    }

    pub fn dot(&self, rhs: &Embedding) -> f64 {
        // We assume that the dimensions always match.
        // The embeddings returned from the OpenAI API always have the same dimensions.
        self.data
            .iter()
            .zip(rhs.data.iter())
            .map(|(x, y)| x * y)
            .sum::<f64>()
            .sqrt()
    }

    pub fn l2_norm(&self) -> f64 {
        self.data.iter().map(|x| x.powf(2.0)).sum::<f64>().sqrt()
    }
}
