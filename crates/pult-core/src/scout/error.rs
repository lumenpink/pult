use thiserror::Error;

/// Erros tipados decorrentes da comunicação e inferência com o Ollama / Scout.
#[derive(Debug, Error)]
pub enum ScoutError {
    #[error("Ollama inacessível em '{endpoint}': {message}")]
    OllamaUnreachable { endpoint: String, message: String },

    #[error("Modelo local '{model}' não encontrado no Ollama. Modelos disponíveis: {available:?}")]
    ModelNotFound {
        model: String,
        available: Vec<String>,
    },

    #[error("Erro HTTP ao comunicar com Ollama: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Falha ao analisar resposta do Ollama: {0}")]
    ParseError(String),
}
