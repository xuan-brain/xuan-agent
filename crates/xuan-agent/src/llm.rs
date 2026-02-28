//! LLM 服务模块

use crate::{config::AiProviderConfig, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// LLM 服务
pub struct LlmService {
    client: Client,
    config: AiProviderConfig,
    system_prompt: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}

impl LlmService {
    /// 创建新的 LLM 服务
    pub fn new(config: AiProviderConfig) -> Self {
        Self {
            client: Client::new(),
            config,
            system_prompt: "你是 XuanAgent，一个专业的科研 AI 助手。".to_string(),
        }
    }

    /// 设置 System Prompt
    pub fn set_system_prompt(&mut self, prompt: String) {
        self.system_prompt = prompt;
    }

    /// 获取当前 System Prompt
    pub fn get_system_prompt(&self) -> &str {
        &self.system_prompt
    }

    /// 使用自定义 System Prompt 聊天
    pub async fn chat_with_prompt(&self, message: &str, system_prompt: &str) -> Result<String> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: message.to_string(),
                },
            ],
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await?

            .json::<ChatResponse>()
            .await
            .map_err(|e| crate::Error::Llm(format!("解析响应失败: {}", e)))?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Ok("抱歉，没有收到响应。".to_string())
        }
    }

    /// 聊天接口
    pub async fn chat(&self, message: &str) -> Result<String> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: self.system_prompt.clone(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: message.to_string(),
                },
            ],
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await?

            .json::<ChatResponse>()
            .await
            .map_err(|e| crate::Error::Llm(format!("解析响应失败: {}", e)))?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Ok("抱歉，没有收到响应。".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_llm_service() {
        let config = AiProviderConfig {
            provider: "test".to_string(),
            base_url: "http://test.com".to_string(),
            api_key: "test-key".to_string(),
            model: "test-model".to_string(),
            description: None,
        };

        let service = LlmService::new(config);
        assert_eq!(service.get_system_prompt(), "你是 XuanAgent，一个专业的科研 AI 助手。");
    }

    #[test]
    fn test_set_system_prompt() {
        let config = AiProviderConfig {
            provider: "test".to_string(),
            base_url: "http://test.com".to_string(),
            api_key: "test-key".to_string(),
            model: "test-model".to_string(),
            description: None,
        };

        let mut service = LlmService::new(config);
        service.set_system_prompt("自定义提示词".to_string());
        assert_eq!(service.get_system_prompt(), "自定义提示词");
    }
}
