use crate::watchdog::{host_infra::HostHealthResponse, Context, HostId};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, sync::Arc};

const HEALTH_CHECK_TIMEOUT: Duration = Duration::seconds(10);
const GRACE_PERIOD: Duration = Duration::seconds(30);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostRegistryEntry {
    pub registered_at: DateTime<Utc>,
    pub last_successful_health_check: Option<DateTime<Utc>>,
    pub ip: Option<IpAddr>,
}

pub enum HostAction {
    None,
    Terminate,
}

pub struct InMemoryHostRegistry {
    entries: Arc<DashMap<HostId, HostRegistryEntry>>,
}

impl InMemoryHostRegistry {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
        }
    }

    pub async fn register_or_get(
        &self,
        context: &Context,
        host_id: &HostId,
    ) -> HostRegistryEntry {
        self.entries
            .entry(host_id.clone())
            .or_insert_with(|| HostRegistryEntry {
                registered_at: context.start_time,
                last_successful_health_check: None,
                ip: None,
            })
            .clone()
    }

    pub async fn update_health_check(
        &self,
        context: &Context,
        host_id: &HostId,
        health_response: Option<HostHealthResponse>,
    ) -> color_eyre::Result<HostAction> {
        let mut entry = self
            .entries
            .entry(host_id.clone())
            .or_insert_with(|| HostRegistryEntry {
                registered_at: context.start_time,
                last_successful_health_check: None,
                ip: None,
            });

        // 헬스체크 성공 시에만 업데이트
        if let Some(health_resp) = health_response {
            entry.last_successful_health_check = Some(context.start_time);
            entry.ip = Some(health_resp.ip);
        }

        // 타임아웃 체크
        let should_terminate = self.should_terminate(&entry, context.start_time);

        if should_terminate {
            Ok(HostAction::Terminate)
        } else {
            Ok(HostAction::None)
        }
    }

    fn should_terminate(&self, entry: &HostRegistryEntry, current_time: DateTime<Utc>) -> bool {
        let time_since_registration = current_time - entry.registered_at;

        // Grace Period 이내면 보호
        if time_since_registration < GRACE_PERIOD {
            return false;
        }

        // Grace Period 경과 후
        match entry.last_successful_health_check {
            None => {
                // 한 번도 성공하지 않았으면 terminate
                true
            }
            Some(last_success) => {
                // 마지막 성공으로부터 10초 이상 경과하면 terminate
                current_time - last_success > HEALTH_CHECK_TIMEOUT
            }
        }
    }

    pub async fn mark_as_removed(&self, host_id: &HostId) {
        self.entries.remove(host_id);
    }

    pub async fn get_healthy_ips(&self) -> Vec<IpAddr> {
        let current_time = Utc::now();
        self.entries
            .iter()
            .filter_map(|entry| {
                let host_entry = entry.value();
                // 타임아웃되지 않은 호스트만
                if !self.should_terminate(host_entry, current_time) {
                    host_entry.ip
                } else {
                    None
                }
            })
            .collect()
    }

    pub async fn get_all_entries(&self) -> Vec<(HostId, HostRegistryEntry)> {
        self.entries
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    pub async fn cleanup_old_entries(&self, _current_time: DateTime<Utc>) {
        // 필요시 오래된 항목 정리 (현재는 제거된 호스트만 정리)
        // 추가 로직이 필요하면 여기에 구현
    }
}
