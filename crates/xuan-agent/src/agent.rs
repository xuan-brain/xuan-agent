//! XuanAgent 核心逻辑

use crate::{config::Config, llm::LlmService, mcp::McpHost, tools::ToolRegistry, Result};

/// XuanAgent 核心结构
pub struct XuanAgent {
    config: Config,
    llm: LlmService,
    mcp_host: McpHost,
    tools: ToolRegistry,
}

impl XuanAgent {
    /// 创建新的 Agent 实例
    pub async fn new(config: Config) -> Result<Self> {
        // 创建 LLM 服务
        let mut llm = LlmService::new(config.ai_provider.clone());

        // 设置 System Prompt
        let system_prompt_config = config.get_system_prompt();
        llm.set_system_prompt(system_prompt_config.base);

        // 创建 MCP Host
        let mcp_host = McpHost::new();

        // 创建工具注册表
        let tools = ToolRegistry::new();

        Ok(Self { config, llm, mcp_host, tools })
    }

    /// 对话接口
    pub async fn chat(&mut self, message: &str) -> Result<String> {
        self.llm.chat(message).await
    }

    /// 对话接口（带工具调用支持）
    pub async fn chat_with_tools(&mut self, message: &str) -> Result<String> {
        // TODO: P0-Sprint 2 - 实现工具调用
        // 目前直接调用 LLM，后续需要：
        // 1. 将可用工具信息传递给 LLM
        // 2. 解析 LLM 返回的工具调用请求
        // 3. 执行工具调用
        // 4. 将工具结果返回给 LLM

        // 构建包含工具信息的 System Prompt
        let prompt = self.build_prompt_with_tools();
        self.llm.chat_with_prompt(message, &prompt).await
    }

    /// 构建包含工具信息的 System Prompt
    fn build_prompt_with_tools(&self) -> String {
        let system_prompt_config = self.config.get_system_prompt();
        let tools = self.tools.list_tools();

        if tools.is_empty() {
            return system_prompt_config.base;
        }

        // 构建工具列表文本
        let tools_text: Vec<String> = tools.iter()
            .map(|t| format!("- **{}**: {}", t.name, t.description))
            .collect();

        // 替换模板中的 {tools} 占位符
        system_prompt_config
            .tools_template
            .replace("{tools}", &tools_text.join("\n"))
            + &system_prompt_config.base
    }

    /// 注册工具
    pub fn register_tool(&mut self, tool: crate::tools::Tool) {
        self.tools.register(tool);
    }

    /// 获取工具注册表引用
    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    /// 获取工具注册表可变引用
    pub fn tools_mut(&mut self) -> &mut ToolRegistry {
        &mut self.tools
    }

    /// 检查 Agent 是否就绪
    pub fn is_ready(&self) -> bool {
        true
    }

    /// 获取配置引用
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// 添加 MCP 服务器
    pub async fn add_mcp_server(&mut self, name: &str, command: &str) -> Result<()> {
        self.mcp_host.start_server(name, command).await
    }

    /// 获取 MCP Host 引用
    pub fn mcp_host(&self) -> &McpHost {
        &self.mcp_host
    }

    /// 获取 MCP Host 可变引用
    pub fn mcp_host_mut(&mut self) -> &mut McpHost {
        &mut self.mcp_host
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        serde_json::from_str(r#"{
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
        }"#).unwrap()
    }

    #[tokio::test]
    async fn test_agent_creation() {
        let config = create_test_config();
        let agent = XuanAgent::new(config).await.unwrap();
        assert!(agent.is_ready());
    }
}
