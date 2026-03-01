//! XuanAgent Web 服务器
//!
//! 提供 HTTP API 和 Web 界面

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use xuan_agent::{Config, Error, XuanAgent};

/// 应用状态
#[derive(Clone)]
struct AppState {
    agent: Arc<Mutex<Option<XuanAgent>>>,
    config: Config,
}

/// 聊天请求
#[derive(Debug, Deserialize)]
struct ChatRequest {
    message: String,
    #[serde(default)]
    session_id: String,
}

/// 聊天响应
#[derive(Debug, Serialize)]
struct ChatResponse {
    response: String,
    session_id: String,
}

/// 搜索请求
#[derive(Debug, Deserialize)]
struct SearchRequest {
    query: String,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 {
    10
}

/// 搜索响应
#[derive(Debug, Serialize)]
struct SearchResponse {
    query: String,
    results: Vec<PaperSummary>,
    count: usize,
}

/// 文献摘要
#[derive(Debug, Serialize)]
struct PaperSummary {
    id: String,
    title: String,
    authors: Vec<String>,
    year: Option<i32>,
    journal: Option<String>,
}

/// 错误响应
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// API 错误
struct AppError(anyhow::Error);

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}

impl From<Error> for AppError {
    fn from(err: Error) -> Self {
        Self(anyhow::anyhow!(err))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("xuan_agent_web=debug".parse()?),
        )
        .init();

    // 加载配置
    let config = Config::from_env()?;

    // 创建应用状态
    let state = AppState {
        agent: Arc::new(Mutex::new(None)),
        config,
    };

    // 构建 CORS 层
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建路由
    let app = Router::new()
        // API 路由
        .route("/api/chat", post(chat_handler))
        .route("/api/search", post(search_handler))
        .route("/api/papers", get(list_papers_handler))
        .route("/api/health", get(health_handler))
        // 静态文件服务
        .nest_service("/assets", ServeDir::new("static"))
        // 首页
        .route("/", get(index_handler))
        .layer(cors)
        .with_state(state);

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("🚀 Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 聊天 API 处理器
async fn chat_handler(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    // 确保已初始化 Agent
    {
        let mut agent_guard = state.agent.lock().await;
        if agent_guard.is_none() {
            tracing::info!("初始化 XuanAgent...");
            let agent = XuanAgent::new(state.config.clone()).await?;
            *agent_guard = Some(agent);
        }
    }

    // 获取响应
    let mut agent_guard = state.agent.lock().await;
    let agent = agent_guard.as_mut().unwrap();

    let response = agent.chat(&req.message).await?;
    let session_id = if req.session_id.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        req.session_id
    };

    Ok(Json(ChatResponse {
        response,
        session_id,
    }))
}

/// 搜索 API 处理器
async fn search_handler(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, AppError> {
    use xuan_agent::storage::SurrealDBStorage;

    let storage = SurrealDBStorage::connect(&state.config.db).await?;

    let papers = storage.search_by_keyword(&req.query, req.limit).await?;

    let results: Vec<PaperSummary> = papers
        .into_iter()
        .map(|p| PaperSummary {
            id: p.id,
            title: p.title,
            authors: p.authors,
            year: p.year,
            journal: p.journal,
        })
        .collect();

    let count = results.len();

    Ok(Json(SearchResponse {
        query: req.query,
        results,
        count,
    }))
}

/// 列出文献 API 处理器
async fn list_papers_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<PaperSummary>>, AppError> {
    use xuan_agent::storage::SurrealDBStorage;

    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(20);

    let storage = SurrealDBStorage::connect(&state.config.db).await?;

    let papers = storage.list_papers(limit).await?;

    let results: Vec<PaperSummary> = papers
        .into_iter()
        .map(|p| PaperSummary {
            id: p.id,
            title: p.title,
            authors: p.authors,
            year: p.year,
            journal: p.journal,
        })
        .collect();

    Ok(Json(results))
}

/// 健康检查处理器
async fn health_handler() -> &'static str {
    "OK"
}

/// 首页处理器
async fn index_handler() -> Html<&'static str> {
    Html(include_str!("static/index.html"))
}
