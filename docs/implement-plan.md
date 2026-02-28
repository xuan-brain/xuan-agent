根据《科研AI Agent（SciAgent）需求说明书 v1.0》，制定以下详细开发计划。本计划遵循“**小步快跑、增量交付、优先复用**”的原则，确保在一年内交付高质量的产品。
---
## 一、 项目总体规划
### 1.1 开发方法论
- **模型**：采用 **Scrum 敏捷开发**模式，以2周为一个Sprint（迭代周期）。
- **优先级原则**：
  1. **MCP First**：优先集成成熟开源MCP服务器（如Zotero MCP），快速构建能力。
  2. **Core Stability**：优先确保Rust核心框架的稳定与性能，再扩展上层应用。
  3. **User-Centric**：每个阶段必须产出可验证的、面向用户的功能。
### 1.2 里程碑概览
| 阶段 | 时间跨度 | 核心目标 | 关键交付物 |
| :--- | :--- | :--- | :--- |
| **P0: 基础设施与原型** | 第1-2月 | 跑通“对话式文献检索”闭环 | Rust核心框架、MCP Host、基础聊天界面 |
| **P1: 智能文献处理** | 第3-5月 | 实现“文献深度理解与知识库” | 文献阅读MCP、向量知识库、PDF阅读视图 |
| **P2: 学术写作辅助** | 第6-8月 | 实现“从文献到写作”的流程 | 写作MCP、论文/基金撰写工具、编辑器插件 |
| **P3: 产品化与生态** | 第9-12月 | 打造成熟产品与开发者生态 | 性能优化、完整Web应用、插件市场、SDK |
---
## 二、 详细迭代执行计划
### 第一阶段：P0 基础设施与原型（第1-2个月）
**目标**：搭建基于Rust+Rig的Agent骨架，集成开源Zotero MCP，实现最简可行产品（MVP）。
#### Sprint 1 (Week 1-2): 核心框架搭建
- **后端**:
  - 初始化 Rust workspace，搭建 `core`, `mcp/host`, `api` 模块。
  - **集成 Rig 框架**：配置 LLM Provider（OpenAI/Local），实现基础的 `Agent` 结构体，具备对话能力。
- **MCP层**:
  - 实现 MCP Host 的基础协议栈（JSON-RPC 2.0）。
  - **集成 Zotero MCP**：不重造轮子，直接作为外部进程或服务启动，连接用户的 Zotero 数据库。
- **前端**:
  - 初始化 React 项目，实现最简单的聊天界面（输入框 + 对话记录）。
#### Sprint 2 (Week 3-4): 文献检索对话（MVP发布）
- **功能**:
  - Agent 接收用户自然语言指令（如“帮我找关于Rust并发的论文”）。
  - Agent 解析意图，调用 MCP Host，向 Zotero MCP 发起检索指令。
  - 结果以卡片形式返回前端展示。
- **交付物**:
  - **v0.1.0 Alpha**: 一个命令行工具或简易Web页面，能“看懂”用户Zotero库并回答简单问题。
---
### 第二阶段：P1 智能文献处理（第3-5个月）
**目标**：从“查”到“懂”，构建私有化知识库，实现深度阅读。
#### Sprint 3 (Week 5-6): 向量知识库构建
- **后端**:
  - 引入向量数据库（推荐 Qdrant 或 Postgres pgvector，通过 Rig 集成）。
  - 开发 **Ingestion Pipeline**: 监听 Zotero 新增文献 -> 提取 PDF 全文 -> 切片 -> Embedding -> 入库。
- **MCP层**:
  - 开发 **`mcp-server-semantic-scholar`**: 封装外部学术 API，扩展检索范围。
#### Sprint 4 (Week 7-8): 文献阅读MCP开发
- **核心开发**: 开发 **`mcp-server-literature-reading`** (Rust)。
  - 实现 `tools`:
    - `summarize_paper`: 调用 LLM 生成结构化摘要。
    - `compare_papers`: 基于向量库检索，对比多篇文献的方法差异。
  - 实现 `resources`: 暴露 PDF 全文流。
- **前端**:
  - 开发 PDF 阅读器组件，支持侧边栏 AI 对话（选中文本 -> 发起提问）。
#### Sprint 5 (Week 9-10): 知识管理自动化
- **功能**:
  - 自动标签系统：LLM 批量为新入库文献打标签（方法/领域/年份）。
  - 知识图谱可视化：提取作者、机构、方法节点，前端使用 Force Graph 展示关联。
- **交付物**:
  - **v0.5.0 Beta**: 完整的文献管理与阅读平台，支持私有化部署。
---
### 第三阶段：P2 学术写作辅助（第6-8个月）
**目标**：打通“读-写”链路，核心是学术写作 MCP 的实现。
#### Sprint 6 (Week 11-12): 学术写作MCP核心
- **核心开发**: 开发 **`mcp-server-academic-writing`** (Rust)。
  - 实现 `tools`:
    - `draft_section`: 基于文献库引用，生成初稿段落。
    - `improve_text`: 接入学术润色 Prompt（参考 Academic-Writing-Assistant）。
    - `check_logic`: 逻辑连贯性检查。
  - 定义 `resources`:
    - `template://nsfc`: 基金申请书模板。
    - `style://ieee`: 写作风格指南。
#### Sprint 7 (Week 13-14): 论文与基金撰写工作流
- **Agent编排**:
  - 实现“基金撰写”工作流：用户输入研究思路 -> Agent 检索背景 -> 调用 `draft_section` 生成各模块 -> 调用 `check_logic` 自检。
  - 实现“综述撰写”工作流：用户选文献 -> Agent 提取观点 -> 生成综述草稿。
- **前端**:
  - 开发“写作工作台”：左侧文献列表，中间 Markdown 编辑器，右侧 AI 助手。
#### Sprint 8 (Week 15-16): 编辑器集成
- **开发**:
  - 开发 **VS Code 插件**：连接后端 MCP Server，支持在代码/文档编辑器中直接调用写作辅助。
  - 开发 **Obsidian 插件**：支持在笔记软件中进行科研对话与写作。
- **交付物**:
  - **v0.8.0 RC**: 支持论文与基金撰写的完整功能版本。
---
### 第四阶段：P3 产品化与生态（第9-12个月）
**目标**：性能优化、用户体验打磨、开放生态。
#### Sprint 9-10 (Week 17-20): 性能与安全
- **性能**:
  - 针对 PDF 解析和向量化进行 Rust 性能调优（并发、零拷贝）。
  - 实施缓存策略（Redis）以减少 LLM API 调用成本。
- **安全**:
  - 实施用户认证与权限系统（RBAC）。
  - 增加审计日志与数据加密模块。
#### Sprint 11-12 (Week 21-24): 社区与SDK
- **生态**:
  - 发布 MCP Server 开发 SDK（Rust版），方便第三方开发者接入特有数据库。
  - 完善 Docker Compose / K8s 部署脚本，支持一键部署。
  - 编写详细的开发者文档与 API 文档。
- **交付物**:
  - **v1.0.0 Release**: 正式稳定版，公开源代码，发布二进制安装包。
---
## 三、 技术架构落地指引
### 3.1 Rust 核心代码结构规划
建议按以下结构组织代码，以解耦 Agent、MCP 和业务逻辑：
```text
sci-agent/
├── crates/
│   ├── core-agent/       # 核心Agent逻辑 (Rig Agent, Memory, Planner)
│   ├── mcp-host/         # MCP 客户端实现，负责维护与Server的连接
│   ├── mcp-servers/      # 自研的 MCP Servers (reading, writing)
│   │   ├── mcp-server-reading/
│   │   └── mcp-server-writing/
│   ├── rag-engine/       # 文献切片、向量化、检索逻辑
│   └── integrations/     # 外部依赖 (Zotero API, Semantic Scholar API)
├── web/                  # Web前端 (React/Next.js)
├── plugins/              # 编辑器插件
└── Cargo.toml
```
### 3.2 关键技术难点攻克
| 难点 | 解决方案 | 预研时间 |
| :--- | :--- | :--- |
| **PDF解析质量** | 不使用简单的文本提取，结合 `pdfium` 提取布局信息，保留段落结构。 | P0阶段 |
| **长文本RAG** | 采用 Rig 框架支持的混合检索策略（关键词+向量），并引入 Re-rank（重排序）模型。 | P1阶段 |
| **写作连贯性** | 在写作 MCP 中维护“文档上下文”，不仅仅是生成段落，还要维护全文的逻辑状态。 | P2阶段 |
---
## 四、 资源配置建议
### 4.1 团队角色（最小配置）
- **Rust 后端开发 x 2**: 负责核心框架、MCP Host/Servers 开发。
- **前端开发 x 1**: 负责 Web UI 和 编辑器插件。
- **算法/数据工程师 x 1** (兼职/顾问): 负责 RAG 流程调优、Prompt Engineering。
### 4.2 技术选型确认
- **LLM 框架**: Rig (Rust)
- **协议**: Model Context Protocol (MCP)
- **向量库**: Qdrant (性能优异，Rust生态友好)
- **前端**: Next.js + Tailwind CSS + Shadcn/UI
- **PDF引擎**: pdfium (通过 FFI 调用)
---
## 五、 风险监控与应对
在开发计划执行过程中，需每周 Review 以下风险项：
1.  **MCP 协议变动风险**:
    - *监控*: 订阅 MCP 官方 GitHub 更新日志。
    - *应对*: 在 `mcp-host` 层做好抽象封装，底层协议变动不影响上层 Agent。
2.  **LLM 输出不稳定**:
    - *监控*: 建立“测试集”，每次修改 Prompt 后自动化测试生成质量。
    - *应对*: 引入 LLM Ops 工具（如 Langfuse 本地部署）进行调试与追踪。
3.  **开源依赖维护中断**:
    - *监控*: 检查 Zotero MCP 等依赖的 Commit 活跃度。
    - *应对*: Fork 关键依赖库到自己的代码仓库，确保掌控力。
---
**下一步行动**：建议立即启动 **P0-Sprint 1**，搭建 Rust 工程脚手架，并跑通第一个 MCP 调用 Demo（`User -> Agent -> Zotero MCP -> "Hello Zotero"`）。
