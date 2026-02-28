//! 工具调用模块

use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: JsonValue,
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: String,
    pub metadata: Option<JsonValue>,
}

/// 工具注册表
pub struct ToolRegistry {
    tools: Vec<Tool>,
}

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
        }
    }

    /// 注册工具
    pub fn register(&mut self, tool: Tool) {
        self.tools.push(tool);
    }

    /// 获取所有工具
    pub fn list_tools(&self) -> &[Tool] {
        &self.tools
    }

    /// 根据名称获取工具
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.iter().find(|t| t.name == name)
    }

    /// 获取工具的 JSON Schema (用于 LLM)
    pub fn get_tools_schema(&self) -> JsonValue {
        let tools: Vec<JsonValue> = self.tools.iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            })
            .collect();

        serde_json::json!({ "tools": tools })
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();

        registry.register(Tool {
            name: "test_tool".to_string(),
            description: "测试工具".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"}
                }
            }),
        });

        assert_eq!(registry.list_tools().len(), 1);
        assert!(registry.get_tool("test_tool").is_some());
        assert!(registry.get_tool("nonexistent").is_none());
    }
}
