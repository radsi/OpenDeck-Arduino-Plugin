use std::{collections::HashSet, time::Duration};

use serialport::available_ports;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{
    DEVICES, TOKENS, TRACKER,
    device::device_task,
    mappings::{DEVICE_NAMESPACE, is_arduino_port, port_to_candidate},
};

pub async fn watcher_task(token: CancellationToken) -> Result<(), anyhow::Error> {
    let tracker = TRACKER.lock().await.clone();

    let mut known_ports: HashSet<String> = HashSet::new();

    log::info!("Arduino serial watcher started");

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                log::info!("Watcher shutting down");
                break;
            }

            _ = sleep(Duration::from_secs(2)) => {
                let ports = match available_ports() {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("Failed to list serial ports: {}", e);
                        continue;
                    }
                };

                let current_ports: HashSet<String> = ports
                    .iter()
                    .filter(|p| is_arduino_port(p))
                    .map(|p| p.port_name.clone())
                    .collect();

                for port in current_ports.difference(&known_ports) {
                    log::info!("New Arduino detected: {}", port);

                    let port_info = match ports.iter().find(|p| &p.port_name == port) {
                        Some(p) => p.clone(),
                        None => continue,
                    };

                    let candidate = match port_to_candidate(port_info) {
                        Some(c) => c,
                        None => continue,
                    };

                    if DEVICES.read().await.contains_key(&candidate.id) {
                        continue;
                    }

                    let token = CancellationToken::new();

                    TOKENS
                        .write()
                        .await
                        .insert(candidate.id.clone(), token.clone());

                    tracker.spawn(device_task(candidate, token));
                }

                for port in known_ports.difference(&current_ports) {
                    log::info!("Arduino disconnected: {}", port);

                    let id = format!("{}-{}", DEVICE_NAMESPACE, port);

                    if let Some(token) = TOKENS.write().await.remove(&id) {
                        token.cancel();
                    }

                    DEVICES.write().await.remove(&id);

                    if let Some(outbound) = crate::OUTBOUND_EVENT_MANAGER.lock().await.as_mut() {
                        let _ = outbound.deregister_device(id.clone()).await;
                    }
                }

                known_ports = current_ports;
            }
        }
    }

    Ok(())
}
