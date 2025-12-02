//! Типы данных для UI
//! 
//! Структуры для состояния NTP-сервера.
//! Дублируем здесь, чтобы не зависеть от Linux-специфичного демона ntpd.

use serde::{Deserialize, Serialize};
use ntp_proto::{SystemSnapshot, NtpTimestamp};
use std::net::SocketAddr;

/// Состояние NTP-сервера
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ObservableState {
    /// Информация о программе (версия, время работы)
    pub program: ProgramData,
    
    /// Состояние синхронизации (stratum, offset и т.д.)
    pub system: SystemSnapshot,
    
    /// Список источников времени
    pub sources: Vec<ObservableSourceState>,
    
    /// Список серверов (если работает в серверном режиме)
    pub servers: Vec<ObservableServerState>,
    
    /// История для графика (не приходит с сервера, заполняется в UI)
    #[serde(skip)] 
    pub history: Vec<f64>,
}

impl Default for ObservableState {
    fn default() -> Self {
        Self {
            program: ProgramData::default(),
            system: SystemSnapshot {
                stratum: 16, // 16 = не синхронизирован
                ..Default::default() 
            },
            sources: vec![],
            servers: vec![],
            history: vec![0.0; 60], // 60 точек для графика
        }
    }
}

/// Информация о программе
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ProgramData {
    pub version: String,
    pub build_commit: String,
    pub build_commit_date: String,
    pub uptime_seconds: f64,
    pub now: NtpTimestamp,
}

/// Один источник времени
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ObservableSourceState {
    pub name: String,
    pub address: String,
    /// Доступность (0-255, где 255 = все опросы успешны)
    pub reachability: u8,
}

/// Состояние серверного режима
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ObservableServerState {
    pub address: SocketAddr,
    pub stats: ServerStats,
}

/// Статистика обработки запросов
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ServerStats {
    pub received_packets: u64,
    pub accepted_packets: u64,
    pub denied_packets: u64,
    pub rate_limited_packets: u64,
}
