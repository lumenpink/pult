pub mod client;
pub mod engine;
pub mod error;
pub mod models;

pub use client::{
    OllamaClient, DEFAULT_OLLAMA_MODEL, DEFAULT_OLLAMA_URL, DOCKER_GATEWAY_OLLAMA_URL,
};
pub use engine::ScoutEngine;
pub use error::ScoutError;
pub use models::{GpuModelInfo, ScoutPromptContext, ScoutResult};
