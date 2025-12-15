use color_eyre::eyre::Result;
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::watchdog::{self, host_infra::HostHealthResponse, Context, HostId};

const HEARTBEAT_TIMEOUT_SECS: u64 = 10;

enum HealthCheckMessage {
    CheckResult {
        host_id: HostId,
        health_response: Option<HostHealthResponse>,
    },
}

struct TaskInfo {
    _join_handle: JoinHandle<()>,
    heartbeat: Arc<AtomicU64>,
    cancel_token: CancellationToken,
}

pub async fn run() -> Result<()> {
    let host_infra = Arc::new(watchdog::host_infra::oci::OciHostInfra::new().await);
    let host_registry = Arc::new(watchdog::host_registry::InMemoryHostRegistry::new());
    let context = Arc::new(Context::new());

    let (tx, mut rx) = mpsc::unbounded_channel();
    let tasks: Arc<DashMap<HostId, TaskInfo>> = Arc::new(DashMap::new());

    let mut discovery_interval = tokio::time::interval(Duration::from_secs(10));
    discovery_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            }
            _ = discovery_interval.tick() => {
                manage_host_tasks(
                    &tasks,
                    host_infra.as_ref(),
                    &host_registry,
                    &tx,
                    &context
                ).await;
            }
            Some(msg) = rx.recv() => {
                handle_health_check_result(
                    msg,
                    &host_registry,
                    host_infra.as_ref(),
                    &tasks,
                    &context
                ).await;
            }
        }
    }

    // Graceful shutdown
    for entry in tasks.iter() {
        entry.value().cancel_token.cancel();
    }

    Ok(())
}

async fn health_check_task(
    host_info: watchdog::host_infra::HostInfo,
    domain: String,
    tx: mpsc::UnboundedSender<HealthCheckMessage>,
    heartbeat: Arc<AtomicU64>,
    cancel_token: CancellationToken,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(3));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                break;
            }
            _ = interval.tick() => {
                // Heartbeat 갱신
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                heartbeat.store(now, Ordering::Relaxed);

                // Health check 수행
                let health_response = watchdog::check_health_single(&host_info, &domain).await;

                // 결과 전송
                let _ = tx.send(HealthCheckMessage::CheckResult {
                    host_id: host_info.id.clone(),
                    health_response,
                });
            }
        }
    }
}

async fn manage_host_tasks(
    tasks: &DashMap<HostId, TaskInfo>,
    host_infra: &dyn watchdog::host_infra::HostInfra,
    host_registry: &watchdog::host_registry::InMemoryHostRegistry,
    tx: &mpsc::UnboundedSender<HealthCheckMessage>,
    context: &Context,
) {
    // 1. 현재 인프라의 호스트 목록 가져오기
    let host_infos = match host_infra.get_host_infos().await {
        Ok(infos) => infos,
        Err(e) => {
            eprintln!("Failed to get host infos: {e:?}");
            return;
        }
    };

    let current_hosts: HashSet<HostId> = host_infos.iter().map(|h| h.id.clone()).collect();

    // 2. 인프라에서 제거된 호스트의 태스크 정리
    let removed_hosts: Vec<HostId> = tasks
        .iter()
        .filter_map(|entry| {
            if !current_hosts.contains(entry.key()) {
                Some(entry.key().clone())
            } else {
                None
            }
        })
        .collect();

    for host_id in removed_hosts {
        if let Some((_, task_info)) = tasks.remove(&host_id) {
            task_info.cancel_token.cancel();
            host_registry.mark_as_removed(&host_id).await;
        }
    }

    // 3. 각 호스트에 대해 태스크 확인 및 생성
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for host_info in host_infos {
        let should_spawn = if let Some(task_info) = tasks.get(&host_info.id) {
            // 태스크가 존재하면 heartbeat 확인
            let last_heartbeat = task_info.heartbeat.load(Ordering::Relaxed);
            let elapsed = now.saturating_sub(last_heartbeat);

            if elapsed > HEARTBEAT_TIMEOUT_SECS {
                // Heartbeat가 오래됨 → 기존 태스크 취소하고 새로 생성
                task_info.cancel_token.cancel();
                true
            } else {
                // Heartbeat 정상 → 유지
                false
            }
        } else {
            // 태스크가 없음 → 생성 필요
            true
        };

        if should_spawn {
            let heartbeat = Arc::new(AtomicU64::new(now));
            let cancel_token = CancellationToken::new();

            let join_handle = tokio::spawn(health_check_task(
                host_info.clone(),
                context.domain.clone(),
                tx.clone(),
                heartbeat.clone(),
                cancel_token.clone(),
            ));

            tasks.insert(
                host_info.id.clone(),
                TaskInfo {
                    _join_handle: join_handle,
                    heartbeat,
                    cancel_token,
                },
            );
        }
    }
}

async fn handle_health_check_result(
    msg: HealthCheckMessage,
    host_registry: &watchdog::host_registry::InMemoryHostRegistry,
    host_infra: &dyn watchdog::host_infra::HostInfra,
    tasks: &DashMap<HostId, TaskInfo>,
    context: &Context,
) {
    match msg {
        HealthCheckMessage::CheckResult {
            host_id,
            health_response,
        } => {
            match host_registry
                .update_health_check(context, &host_id, health_response)
                .await
            {
                Ok(watchdog::host_registry::HostAction::Terminate) => {
                    // 호스트 종료 요청
                    if let Err(e) = host_infra.terminate(&host_id).await {
                        eprintln!("Failed to terminate host {host_id:?}: {e:?}");
                    }

                    // 태스크도 즉시 종료
                    if let Some(task_info) = tasks.get(&host_id) {
                        task_info.cancel_token.cancel();
                    }
                    tasks.remove(&host_id);
                }
                Ok(watchdog::host_registry::HostAction::None) => {
                    // 정상 - 아무 작업 없음
                }
                Err(e) => {
                    eprintln!("Failed to update health check for {host_id:?}: {e:?}");
                }
            }
        }
    }
}
