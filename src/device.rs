use std::io::{BufRead, BufReader};
use std::time::Duration;

use serialport::SerialPort;
use tokio_util::sync::CancellationToken;

use openaction::OUTBOUND_EVENT_MANAGER;

use crate::{
    DEVICES, TOKENS,
    inputs::{InputEvent, InputState, process_input},
    mappings::{COL_COUNT, CandidateDevice, ROW_COUNT},
};

pub struct Device {
    pub id: String,
    pub port_name: String,
}

pub async fn device_task(candidate: CandidateDevice, token: CancellationToken) {
    log::info!("Running device task for {:?}", candidate);

    let port = match connect(&candidate).await {
        Ok(p) => p,
        Err(e) => {
            handle_error(&candidate.id, e.to_string()).await;
            return;
        }
    };

    log::info!("Registering device {}", candidate.id);

    if let Some(outbound) = OUTBOUND_EVENT_MANAGER.lock().await.as_mut() {
        outbound
            .register_device(
                candidate.id.clone(),
                candidate.human_name(),
                ROW_COUNT as u8,
                COL_COUNT as u8,
                0,
                0,
            )
            .await
            .ok();
    }

    DEVICES.write().await.insert(candidate.id.clone(), ());

    let id = candidate.id.clone();

    tokio::task::spawn_blocking(move || device_events_task(id, port, token));

    log::info!("Device task started for {:?}", candidate);
}

pub async fn connect(candidate: &CandidateDevice) -> Result<Box<dyn SerialPort>, String> {
    log::info!("Connecting to {}", candidate.dev);

    let port = serialport::new(&candidate.dev, 115_200)
        .timeout(Duration::from_millis(50))
        .open()
        .map_err(|e| e.to_string())?;

    Ok(port)
}

fn device_events_task(id: String, mut port: Box<dyn SerialPort>, token: CancellationToken) {
    let reader = BufReader::new(port.try_clone().unwrap());
    let mut input_state = InputState::new();

    for line in reader.lines().flatten() {
        if token.is_cancelled() {
            break;
        }

        handle_line(&id, &line, &mut input_state);
    }
}

fn handle_line(id: &str, line: &str, state: &mut InputState) {
    log::debug!("{} -> {}", id, line);

    let mut guard = futures::executor::block_on(OUTBOUND_EVENT_MANAGER.lock());
    let Some(outbound) = guard.as_mut() else {
        return;
    };

    if let Some(event) = process_input(state, line) {
        if let InputEvent::ButtonState(buttons) = event {
            for (idx, pressed) in buttons.iter().enumerate() {
                if *pressed {
                    let _ =
                        futures::executor::block_on(outbound.key_down(id.to_string(), idx as u8));
                } else {
                    let _ = futures::executor::block_on(outbound.key_up(id.to_string(), idx as u8));
                }
            }
        }
    }
}

pub async fn handle_error(id: &String, err: String) -> bool {
    log::error!("Device {} error: {}", id, err);

    if let Some(outbound) = OUTBOUND_EVENT_MANAGER.lock().await.as_mut() {
        outbound.deregister_device(id.clone()).await.ok();
    }

    if let Some(token) = TOKENS.read().await.get(id) {
        token.cancel();
    }

    DEVICES.write().await.remove(id);

    false
}
