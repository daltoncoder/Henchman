use anyhow::Result;
use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, ScalarQuantizationBuilder,
        SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
    },
    Payload, Qdrant,
};
use uuid::Uuid;

use self::types::{Embedding, Memory, MemoryData};

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
                        "id": m.data.id,
                        "content": m.data.content
                    }
                )
                .try_into()
                .unwrap();
                let id_hash = fasthash::spooky::hash128(m.data.id);
                let id = Uuid::from_bytes(id_hash.to_le_bytes());
                PointStruct::new(id.to_string(), m.embedding.data, payload)
            })
            .collect();
        self.client
            .upsert_points(UpsertPointsBuilder::new(collection_name, points))
            .await?;
        Ok(())
    }

    pub async fn get_k_most_similar_memories(
        &self,
        collection_name: &str,
        embedding: Embedding,
        k: u64,
    ) -> Result<Vec<MemoryData>> {
        let search_result = self
            .client
            .search_points(
                SearchPointsBuilder::new(collection_name, embedding.data, k)
                    //.filter(Filter::all([Condition::matches("bar", 12)]))
                    .with_payload(true)
                    .params(SearchParamsBuilder::default().exact(true)),
            )
            .await
            .unwrap();

        let res = search_result
            .result
            .iter()
            .filter_map(|r| {
                if let Some(content) = r.payload.get("content") {
                    let content = content.as_str().unwrap().to_string();
                    if let Some(id) = r.payload.get("id") {
                        let id = id.as_str().unwrap().to_string();
                        Some(MemoryData { id, content })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<MemoryData>>();
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{
        types::{Embedding, Memory, MemoryData},
        DB,
    };

    #[ignore]
    #[tokio::test]
    async fn test_db() {
        let db = DB::new("http://localhost:6334").unwrap();

        let table = "test6";
        db.create_collection(table, 3).await.unwrap();

        let mem1 = Memory {
            data: MemoryData {
                id: String::from("1"),
                content: String::from("hello"),
            },
            embedding: Embedding::new(vec![1.2, 2.2, 3.2]),
        };
        let mem2 = Memory {
            data: MemoryData {
                id: String::from("2"),
                content: String::from("goodbye"),
            },
            embedding: Embedding::new(vec![1.1, 2.1, 3.1]),
        };
        let mem3 = Memory {
            data: MemoryData {
                id: String::from("3"),
                content: String::from("foo"),
            },
            embedding: Embedding::new(vec![1.3, 2.3, 3.3]),
        };
        let mem4 = Memory {
            data: MemoryData {
                id: String::from("3"),
                content: String::from("bar"),
            },
            embedding: Embedding::new(vec![1.4, 2.4, 3.4]),
        };

        let memories = vec![mem1, mem2, mem3, mem4];

        db.upsert_memories(table, memories).await.unwrap();

        let res = db
            .get_k_most_similar_memories(table, Embedding::new(vec![1., 2., 3.]), 2)
            .await
            .unwrap();

        for r in &res {
            println!("{r:?}");
        }
    }
}
