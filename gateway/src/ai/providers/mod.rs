use crate::{ai::docs::types::GeneratedApiDocs, ai::error::AiError};

pub mod gemini;

pub enum DocsProvider {
    Gemini(gemini::GeminiProvider),
}

impl DocsProvider {
    #[inline(always)]
    pub fn default_from_env() -> Result<Self, AiError> {
        Ok(Self::Gemini(gemini::GeminiProvider::from_env()?))
    }

    #[inline(always)]
    pub async fn generate(
        &self,
        prompt: &str,
    ) -> Result<GeneratedApiDocs, AiError> {
        match self {
            DocsProvider::Gemini(provider) => provider.generate(prompt).await,
        }
    }
}
