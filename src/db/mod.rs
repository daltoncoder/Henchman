use anyhow::Result;
use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, ScalarQuantizationBuilder,
        UpsertPointsBuilder, VectorParamsBuilder,
    },
    Payload, Qdrant,
};

use self::types::Memory;

pub mod types;

pub struct DB {
    client: Qdrant,
}

impl DB {
    pub fn new(db_url: &str) -> Result<Self> {
        let client = Qdrant::from_url(db_url).build()?;
        Ok(Self { client })
    }

    pub async fn create_collection(&self, collection_name: &str, vector_dim: u64) -> Result<()> {
        if self.client.collection_exists(collection_name).await? {
            Ok(())
        } else {
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(collection_name)
                        .vectors_config(VectorParamsBuilder::new(vector_dim, Distance::Cosine))
                        .quantization_config(ScalarQuantizationBuilder::default()),
                )
                .await?;
            Ok(())
        }
    }

    pub async fn upsert_memories(
        &self,
        collection_name: &str,
        memories: Vec<Memory>,
    ) -> Result<()> {
        let points: Vec<PointStruct> = memories
            .into_iter()
            .map(|m| {
                let payload: Payload = serde_json::json!(
                    {
                        "id": m.id,
                        "content": m.content
                    }
                )
                .try_into()
                .unwrap();
                PointStruct::new(0, m.embedding.data, payload)
            })
            .collect();
        self.client
            .upsert_points(UpsertPointsBuilder::new(collection_name, points))
            .await?;
        Ok(())
    }
}
