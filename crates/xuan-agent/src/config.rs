//! XuanAgent 配置管理

use serde::Deserialize;
use std::fs;

use crate::{Error, Result};

/// XuanAgent 配置
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// AI Provider 配置
    pub ai_provider: AiProviderConfig,

    /// 数据库配置
    pub db: DbConfig,

    /// System Prompt 配置
    #[serde(default)]
    pub system_prompt: Option<SystemPromptConfig>,
}

/// System Prompt 配置
#[derive(Debug, Clone, Deserialize)]
pub struct SystemPromptConfig {
    /// 基础系统提示词
    #[serde(default)]
    pub base: String,

    /// 工具调用提示模板
    #[serde(default)]
    pub tools_template: String,
}

impl Default for SystemPromptConfig {
    fn default() -> Self {
        Self {
            base: default_system_prompt(),
            tools_template: default_tools_prompt().to_string(),
        }
    }
}

/// AI Provider 配置
#[derive(Debug, Clone, Deserialize)]
pub struct AiProviderConfig {
    /// Provider 类型 (openai, anthropic, siliconflow, ollama)
    pub provider: String,

    /// API 基础 URL
    pub base_url: String,

    /// API 密钥
    pub api_key: String,

    /// 模型名称
    pub model: String,

    /// 配置描述
    #[serde(default)]
    pub description: Option<String>,
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    /// 数据库连接地址
    pub connect: String,

    /// 用户名
    pub user: String,

    /// 密码
    pub pass: String,

    /// 命名空间
    pub namespace: String,

    /// 数据库名称
    pub database: String,
}

impl Config {
    /// 从 JSON 格式的 .env 文件加载配置
    pub fn from_env() -> Result<Self> {
        // 读取 JSON 格式的 .env 文件
        let content = fs::read_to_string(".env")
            .map_err(|e| Error::Config(format!("Failed to read .env: {}", e)))?;

        let config: Config = serde_json::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse .env: {}", e)))?;

        Ok(config)
    }

    /// 获取 LLM 配置元组 (provider, base_url, model)
    pub fn get_llm_config(&self) -> (&str, &str, &str) {
        (
            &self.ai_provider.provider,
            &self.ai_provider.base_url,
            &self.ai_provider.model,
        )
    }

    /// 获取数据库连接 URL
    pub fn get_db_url(&self) -> String {
        format!("{}/db/{}", self.db.connect, self.db.database)
    }

    /// 获取 System Prompt 配置
    pub fn get_system_prompt(&self) -> SystemPromptConfig {
        self.system_prompt.clone().unwrap_or_default()
    }
}

/// 默认科研助手 System Prompt
fn default_system_prompt() -> String {
    r#"# XuanAgent - 科研 AI 助手

你是一个专业的科研助手，专注于协助研究人员进行以下工作：

## 核心能力
- **文献管理**: 帮助检索、整理和管理学术文献
- **学术写作**: 辅助论文写作、润色和格式化
- **知识管理**: 协助构建和维护科研知识库
- **数据分析**: 提供研究数据分析和可视化建议
- **方法咨询**: 解答研究方法和实验设计相关问题

## 工作原则
1. **准确优先**: 对不确定的信息明确说明，不编造内容
2. **学术规范**: 遵循学术写作规范和引用标准
3. **批判思维**: 帮助用户审视研究设计和论证逻辑
4. **效率优先**: 简洁明了地回答，避免冗长

## 交互风格
- 使用中文回复（除非用户明确要求英文）
- 对复杂问题分步骤解答
- 提供可操作的建议和具体示例
- 必要时主动询问细节以提供更精准的帮助"#
        .to_string()
}

/// 默认工具提示模板
fn default_tools_prompt() -> &'static str {
    r#"

## 可用工具

你有以下工具可以使用：

{tools}

当需要使用工具时，请在回复中明确指出要调用的工具名称和参数。
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let json = r#"{
            "ai_provider": {
                "provider": "test",
                "base_url": "http://test.com",
                "api_key": "test-key",
                "model": "test-model"
            },
            "db": {
                "connect": "http://localhost:8000",
                "user": "root",
                "pass": "secret",
                "namespace": "test",
                "database": "test"
            }
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.ai_provider.provider, "test");
        assert_eq!(config.db.database, "test");
    }
}
