use axum::response::Json;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

#[derive(Debug, Serialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<Model>,
}

pub async fn handle_models() -> Json<ModelList> {
    Json(ModelList {
        object: "list".into(),
        data: vec![
            Model {
                id: "gpt-4".into(),
                object: "model".into(),
                created: 1687882411,
                owned_by: "bkg-cave".into(),
            },
            Model {
                id: "gpt-3.5-turbo".into(),
                object: "model".into(),
                created: 1687882411,
                owned_by: "bkg-cave".into(),
            },
            Model {
                id: "bkg-llama-3.1-8b".into(),
                object: "model".into(),
                created: 1720000000,
                owned_by: "bkg-cave".into(),
            },
            Model {
                id: "bkg-llama-3.1-70b".into(),
                object: "model".into(),
                created: 1720000000,
                owned_by: "bkg-cave".into(),
            },
        ],
    })
}
