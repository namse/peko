use crate::watchdog::{host_infra::HostInfra, host_registry::InMemoryHostRegistry, Context};
use chrono::Utc;

pub async fn try_scale_out(
    _context: &Context,
    host_registry: &InMemoryHostRegistry,
    host_infra: &dyn HostInfra,
) -> color_eyre::Result<()> {
    let all_entries = host_registry.get_all_entries().await;
    let current_time = Utc::now();

    // Grace Period 이내의 호스트 수 (starting 중인 호스트)
    let starting_count = all_entries
        .iter()
        .filter(|(_, entry)| {
            let time_since_registration = current_time - entry.registered_at;
            time_since_registration < chrono::Duration::seconds(30)
                && entry.last_successful_health_check.is_none()
        })
        .count();

    println!("starting_count: {starting_count}");

    // 살아있는 호스트 수 (타임아웃되지 않은 호스트)
    let alive_host_len = host_registry.get_healthy_ips().await.len();

    println!("alive_host_len: {alive_host_len}");

    if alive_host_len >= 1 {
        println!("alive_host_len >= 1. No space to launch new host instance");
        return Ok(());
    }

    if starting_count > 0 {
        println!("starting_count > 0. Waiting for hosts to start");
        return Ok(());
    }

    println!("Launching new host instance");
    host_infra.launch_instances(1).await?;
    Ok(())
}
