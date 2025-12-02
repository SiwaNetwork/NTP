//! Веб-интерфейс для мониторинга NTP-сервера
//! 
//! Использую Dioxus - фреймворк на Rust, который компилируется в WebAssembly.
//! По сути это типо React, но на Rust.

#![cfg(feature = "frontend")]
#![allow(non_snake_case)]

mod types;
mod math;
use types::ObservableState;
use dioxus::prelude::*;
use std::collections::VecDeque;

/// Вход в приложение
fn main() {
    // Здесь настраиваем логирование для браузера
    #[cfg(target_arch = "wasm32")]
    {
        tracing_wasm::set_as_global_default();
    }

    // Запускаем приложение
    launch(App);
}

/// Главный компонент приложения
fn App(cx: Scope) -> Element {
    // Храним текущее состояние сервера
    let state = use_state(cx, || ObservableState::default());
    
    // История для графика (последние взятые 60 точек)
    let history = use_state(cx, || VecDeque::from(vec![0.0; 60]));
    
    // Тут будут ошибки, если что-то пошло не так
    let error = use_state(cx, || Option::<String>::None);
    
    // Обновляем данные каждую секунду
    use_future(cx, (), |_| {
        let state = state.clone();
        let history = history.clone();
        let error = error.clone();
        async move {
            loop {
                // Запрашиваем данные с сервера
                match fetch_status().await {
                    Ok(data) => {
                        let new_offset = data.system.time_snapshot.root_delay.to_seconds();
                        
                        // Обновляем состояние
                        state.set(data);
                        error.set(None);
                        
                        // Добавляем новую точку в график
                        history.with_mut(|h| {
                            h.push_back(new_offset);
                            // Храним только последние 60 точек
                            if h.len() > 60 {
                                h.pop_front();
                            }
                        });
                    }
                    Err(e) => {
                        // Показываем ошибку пользователю
                        error.set(Some(format!("Ошибка подключения: {}", e)));
                    }
                }
                
                // Ждем секунду перед следующим обновлением
                gloo_timers::future::TimeoutFuture::new(1000).await;
            }
        }
    });

    // Рендерим интерфейс 
    cx.render(rsx! {
        style { include_str!("style.css") }
        div { class: "container",
            Header { program: state.program.clone() }
            
            // Показываем ошибку, если есть
            if let Some(err_msg) = error.get() {
                div { class: "error-banner",
                    "⚠️ {err_msg}"
                }
            }
            
            div { class: "grid",
                // Карточка со статусом системы и графиком
                SystemCard { 
                    system: state.system.clone(),
                    history: history.iter().cloned().collect(),
                    now: state.program.now
                }
                
                // Карточка со списком источников времени
                SourcesCard { sources: state.sources.clone() }
            }
        }
    })
}

/// Получает данные с сервера через HTTP
async fn fetch_status() -> Result<ObservableState, String> {
    let client = reqwest::Client::new();
    let url = "/api/status";
    
    // Отправляем GET запрос
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Сетевая ошибка: {}", e))?;
    
    // Проверяем, что запрос успешен
    if !response.status().is_success() {
        return Err(format!("HTTP ошибка: {}", response.status()));
    }
    
    // Парсим JSON в структуру
    let data: ObservableState = response
        .json()
        .await
        .map_err(|e| format!("Ошибка парсинга JSON: {}", e))?;
    
    Ok(data)
}

/// Заголовок страницы
#[component]
fn Header(cx: Scope, program: types::ProgramData) -> Element {
    cx.render(rsx! {
        header {
            h1 { "NTPD-RS Dashboard" }
            div { class: "meta",
                span { "Version: {program.version}" }
                span { " | Uptime: {program.uptime_seconds:.1}s" }
            }
        }
    })
}

/// Карточка со статусом системы
#[component]
fn SystemCard(cx: Scope, system: ntp_proto::SystemSnapshot, history: Vec<f64>, now: ntp_proto::NtpTimestamp) -> Element {
    // Считаем тренд изменения времени
    let regression = math::calculate_linear_regression(&history);
    
    // Прогнозируем, какое будет смещение через час
    let prediction_1h = regression.map(|r| {
        // r.slope - насколько быстро меняется время (секунд в секунду)
        // Если положительное - часы спешат, отрицательное - отстают
        let current = history.last().copied().unwrap_or(0.0);
        current + r.slope * 3600.0 // текущее + изменение за час
    });
    
    // Считаем dispersion (погрешность времени)
    let dispersion = system.time_snapshot.root_dispersion(now).to_seconds();
    
    // Считаем jitter (вариативность offset'ов)
    let jitter = math::calculate_jitter(&history);

    cx.render(rsx! {
        div { class: "card",
            h2 { "System Status" }
            div { class: "stat-row",
                div { class: "stat",
                    span { class: "label", "Stratum" }
                    span { class: "value", "{system.stratum}" }
                }
                div { class: "stat",
                    span { class: "label", "Offset" }
                    span { class: "value", "{system.time_snapshot.root_delay.to_seconds():.6}s" }
                }
            }
            
            div { class: "stat-row",
                div { class: "stat",
                    span { class: "label", "Dispersion" }
                    span { class: "value", "{dispersion:.6}s" }
                }
                if let Some(jit) = jitter {
                    div { class: "stat",
                        span { class: "label", "Jitter" }
                        span { class: "value", "{jit:.6}s" }
                    }
                }
            }
            
            // Показываем прогноз, если удалось посчитать
            if let Some(pred) = prediction_1h {
                div { class: "prediction",
                    span { class: "label", "Predicted Drift (1h): " }
                    span { class: "value-small", "{pred*1000.0:+.2} ms" }
                }
            }

            // График изменения смещения
            OffsetChart { data: history.clone() }
        }
    })
}

/// График изменения смещения времени
#[component]
fn OffsetChart(cx: Scope, data: Vec<f64>) -> Element {
    let width = 100.0;
    let height = 50.0;
    
    // Находим максимальное значение для масштабирования
    let max_val = data.iter().fold(0.0f64, |a, &b| a.max(b.abs())).max(0.0001);
    
    // Преобразуем данные в координаты для SVG
    let points: String = data.iter()
        .enumerate()
        .map(|(i, val)| {
            // Распределяем точки равномерно по ширине
            let x = (i as f64 / (data.len() - 1) as f64) * width;
            
            // Центр графика = 0, положительные значения выше, отрицательные ниже
            let y = (height / 2.0) - (val / max_val * (height / 2.0)); 
            
            format!("{:.1},{:.1}", x, y)
        })
        .collect::<Vec<_>>()
        .join(" ");

    cx.render(rsx! {
        div { class: "chart-container",
            div { class: "chart-label", "Offset Stability (±{max_val:.6}s)" }
            svg {
                view_box: "0 0 100 50",
                preserve_aspect_ratio: "none",
                class: "chart-svg",
                
                // Линия нуля (для ориентира)
                line { x1: "0", y1: "25", x2: "100", y2: "25", stroke: "#eee", stroke_width: "0.5" }
                
                // Сам график
                polyline {
                    points: "{points}",
                    fill: "none",
                    stroke: "#0066cc",
                    stroke_width: "1.5",
                    vector_effect: "non-scaling-stroke"
                }
            }
        }
    })
}

/// Карточка со списком источников времени
#[component]
fn SourcesCard(cx: Scope, sources: Vec<types::ObservableSourceState>) -> Element {
    cx.render(rsx! {
        div { class: "card",
            h2 { "Sources ({sources.len()})" }
            if sources.is_empty() {
                div { class: "empty", "No sources configured or connected" }
            } else {
                table {
                    thead {
                        tr {
                            th { "Address" }
                            th { "Reachability" }
                        }
                    }
                    tbody {
                        for source in sources {
                            tr {
                                td { "{source.address}" }
                                td { "{source.reachability}" }
                            }
                        }
                    }
                }
            }
        }
    })
}
