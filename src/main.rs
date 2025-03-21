mod ac;
mod filter;

use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use filter::SensitiveFilter;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

// 定义命令行参数结构体
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 监听端口
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// 绑定的主机地址
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,

    /// 模型目录路径
    #[arg(short, long, default_value = "./")]
    path: PathBuf,

    /// 启动时是否重建索引
    #[arg(short, long)]
    rebuild: bool,
}

// API 过滤请求结构体
#[derive(Debug, Serialize, Deserialize)]
struct FilterRequest {
    // 需要过滤的文本
    text: String,
}

// API 过滤响应结构体
#[derive(Debug, Serialize, Deserialize)]
struct FilterResponse {
    // 过滤后的文本
    filtered: String,
}

// API 检查请求结构体
#[derive(Debug, Serialize, Deserialize)]
struct CheckRequest {
    // 需要检查的文本
    text: String,
}

// API 检查响应结构体
#[derive(Debug, Serialize, Deserialize)]
struct CheckResponse {
    // 是否包含敏感词
    contains_sensitive: bool,
    // 发现的敏感词列表
    words: Vec<String>,
}

// API 状态响应结构体
#[derive(Debug, Serialize, Deserialize)]
struct StatusResponse {
    // 服务状态信息
    status: String,
}

// 程序入口点
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志系统
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // 解析命令行参数
    let args = Args::parse();

    // 初始化敏感词过滤器
    let filter = Arc::new(SensitiveFilter::new(&args.path).await?);

    // 根据参数决定是否重建索引
    if args.rebuild {
        info!("Rebuilding index as requested");
        filter.rebuild_index().await?;
    } else {
        // 尝试初始化现有索引
        filter.init().await?;
    }

    // 构建Web应用路由
    let app = Router::new()
        .route("/filter", post(filter_text))
        .route("/check", post(check_text))
        .route("/rebuild", post(rebuild_index))
        .route("/status", get(status))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10)) // 限制请求体大小为10MB
        .layer(TraceLayer::new_for_http()) // 添加HTTP请求追踪
        .with_state(filter); // 注入敏感词过滤器状态

    // 启动Web服务器
    let addr = format!("{}:{}", args.host, args.port).parse::<SocketAddr>()?;
    info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// 文本过滤API处理函数
async fn filter_text(
    // 从应用状态获取过滤器
    State(filter): State<Arc<SensitiveFilter>>,
    // 从请求体解析JSON
    Json(request): Json<FilterRequest>,
) -> impl IntoResponse {
    // 过滤文本
    let filtered = filter.filter(&request.text).await;
    // 返回过滤后的文本
    Json(FilterResponse { filtered })
}

// 敏感词检查API处理函数
async fn check_text(
    // 从应用状态获取过滤器
    State(filter): State<Arc<SensitiveFilter>>,
    // 从请求体解析JSON
    Json(request): Json<CheckRequest>,
) -> impl IntoResponse {
    // 查找文本中的敏感词
    let words = filter.find_sensitive_words(&request.text).await;
    // 判断是否含有敏感词
    let contains_sensitive = !words.is_empty();
    // 返回检查结果
    Json(CheckResponse {
        contains_sensitive,
        words,
    })
}

// 重建索引API处理函数
async fn rebuild_index(
    // 从应用状态获取过滤器
    State(filter): State<Arc<SensitiveFilter>>,
) -> Result<impl IntoResponse, StatusCode> {
    // 尝试重建索引
    match filter.rebuild_index().await {
        Ok(_) => Ok(Json(StatusResponse {
            status: "Index rebuilt successfully".to_string(),
        })),
        Err(e) => {
            // 记录重建失败的原因
            warn!("Failed to rebuild index: {:?}", e);
            // 返回服务器内部错误
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 服务状态API处理函数
async fn status() -> impl IntoResponse {
    // 返回服务运行状态
    Json(StatusResponse {
        status: "Service is running".to_string(),
    })
}
