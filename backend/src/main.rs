use axum::routing::{get, put};
use backend::handle_404;
use mimalloc::MiMalloc;
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, sync::Arc, time::Duration};

use crate::{
    articles::{get_post_digital, get_posts},
    common::AppState,
    search::{SearchService, get_search_results},
};

// 声明项目内的模块，以便编译器能够找到它们。
mod articles; // 文章相关模块
mod common; // 通用工具或定义模块
mod config; // 配置管理模块
mod db; // 数据库交互模块
mod search; // 搜索功能模块
mod some_errors; // 自定义错误处理模块

// 使用 #[global_allocator] 宏将 MiMalloc 设置为全局内存分配器。
// 这可以提高应用程序的内存分配性能。
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// 定义一些常量，用于设置服务器的默认值。
const DEFAULT_PORT: u16 = 8124; // 默认端口号
const DEFAULT_HOST: &str = "0.0.0.0"; // 默认主机地址，监听所有网络接口
const DEFAULT_INDEX_NAME: &str = "articles"; // 默认的搜索索引名称
const DEFAULT_MAX_CONNECTIONS: u32 = 1000; // 默认的最大连接数
// const DEFAULT_MAX_CONNECTIONS_PER_IP: u32 = 100; // 默认的每个 IP 的最大连接数

// 使用 #[tokio::main] 宏来标记异步主函数，tokio 运行时会自动处理。
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从 .env 文件中加载环境变量。如果文件不存在，.ok() 会忽略错误。
    dotenvy::dotenv().ok();
    // 从环境变量中加载配置。如果失败，程序会 panic 并显示错误信息。
    let config = config::Config::from_env().expect("无法加载配置。请检查您的环境变量。");

    // 获取主机地址，如果环境变量中未设置，则使用默认值。
    let host = config.host.as_deref().unwrap_or(DEFAULT_HOST);
    // 获取端口号，如果环境变量中未设置，则使用默认值。
    let port = config.port.unwrap_or(DEFAULT_PORT);
    // 格式化主机和端口为 "host:port" 形式的字符串。
    let addr = format!("{}:{}", host, port);
    // 将字符串解析为 SocketAddr 类型。如果格式无效，程序会 panic。
    let address: SocketAddr = addr.parse().expect("无效的 HOST:PORT 格式");

    // 绑定 TCP 监听器到指定的地址。
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("无法绑定到地址。端口可能已被占用。");

    // 打印监听地址，方便开发者查看。
    println!("正在监听 http://{}", address);

    // 克隆 JWT 密钥，用于后续的应用状态共享。
    let jwt_secret = config.jwt_secret.clone();
    // 异步初始化搜索服务。
    let search_service = SearchService::new(&config, DEFAULT_INDEX_NAME).await?;

    // 设置数据库连接池选项。
    let pool = PgPoolOptions::new()
        .max_connections(DEFAULT_MAX_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(3)) // 获取连接的超时时间为 3 秒
        .connect(&config.database_url) // 连接到数据库
        .await // 等待连接完成
        .expect("无法连接到数据库。");

    // 创建应用共享状态 AppState 的实例。
    let state = AppState {
        db_pool: pool,                  // 数据库连接池
        jwt_secret: jwt_secret,         // JWT 密钥
        search_service: search_service, // 搜索服务
    };

    // 使用 Arc (原子引用计数) 将 AppState 包装起来，使其可以在多个线程之间安全地共享。
    let state = Arc::new(state);

    // 创建一个 API 路由器，并定义路由。
    let api_router = axum::Router::new()
        // 当收到对 "/search" 的 GET 请求时，调用 get_search_results 函数处理。
        .route("/search", get(get_search_results))
        // 当收到对 "/posts" 的 GET 请求时，调用 get_posts 函数处理。
        .route("/posts", get(get_posts))
        // 当收到对 "/posts/:category/:id" 的 GET 请求时，调用 get_post_digital 函数处理。
        .route("/posts/:category/:id", get(get_post_digital));

    // .route("/blog-update/:id", put("")); // 这是一个被注释掉的路由示例

    // 创建主路由器，并将 API 路由器嵌套在 "/api" 路径下。
    let router = axum::Router::new()
        .nest("/api", api_router)
        // 将共享的应用状态 state 注入到路由器中，这样所有处理器都可以访问它。
        .with_state(state)
        .fallback(handle_404);

    // 启动 axum 服务器，监听传入的连接。
    if let Err(e) = axum::serve(listener, router).await {
        // 如果服务器启动失败，则 panic 并打印错误信息。
        panic!("服务器因错误退出: {}", e);
    }

    // 如果 main 函数成功完成，返回 Ok(())。
    Ok(())
}
