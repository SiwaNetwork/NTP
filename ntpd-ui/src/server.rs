//! HTTP сервер для веб-интерфейса
//! 
//! Прокси между браузером и демоном ntpd-rs.
//! Читает данные из Unix Socket и отдает через HTTP.

#![cfg(feature = "backend")]

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use ntpd::daemon::{sockets::read_json, ObservableState};
use std::path::PathBuf;
use tokio::net::UnixStream;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Хранит путь к Unix Socket демона
#[derive(Clone)]
struct AppState {
    socket_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Настраиваем логирование
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Путь к Unix Socket (можно задать через переменную NTPD_OBSERVE_SOCKET)
    let socket_path = std::env::var("NTPD_OBSERVE_SOCKET")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/run/ntpd-rs/observe"));

    let state = AppState { socket_path };

    // Настраиваем маршруты
    let app = Router::new()
        .route("/api/status", get(get_status))
        .nest_service("/", ServeDir::new("dist"))  // статические файлы
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Адрес и порт 
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse::<u16>()?;
    let addr = format!("{}:{}", host, port).parse()?;

    tracing::info!("🚀 NTP UI Server запущен на http://{}", addr);
    tracing::info!("📡 Подключение к демону через: {:?}", socket_path);
    tracing::info!("🌐 Открой в браузере: http://localhost:{}", port);

    // Запускаем сервер
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Получает состояние сервера из демона
async fn get_status(State(state): State<AppState>) -> Result<Json<ObservableState>, StatusCode> {
    // Подключаемся к Unix Socket
    let mut stream = UnixStream::connect(&state.socket_path)
        .await
        .map_err(|e| {
            tracing::error!("Не удалось подключиться к демону {:?}: {}", state.socket_path, e);
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // Читаем JSON из сокета
    let mut buffer = Vec::with_capacity(16 * 1024);
    let status: ObservableState = read_json(&mut stream, &mut buffer)
        .await
        .map_err(|e| {
            tracing::error!("Ошибка чтения данных из демона: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(status))
}

