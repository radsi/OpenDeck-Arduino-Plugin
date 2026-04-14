use serialport::SerialPortInfo;

pub const ARDUINO_VID: u16 = 0x1A86;

pub const ROW_COUNT: usize = 2;
pub const COL_COUNT: usize = 4;
pub const KEY_COUNT: usize = ROW_COUNT * COL_COUNT;
pub const ENCODER_COUNT: usize = 0;

pub const DEVICE_NAMESPACE: &str = "ar";

#[derive(Debug, Clone)]
pub struct CandidateDevice {
    pub id: String,
    pub dev: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
}

impl CandidateDevice {
    pub fn human_name(&self) -> String {
        "Arduino Stream Deck".to_string()
    }
}

pub fn is_arduino_port(port: &SerialPortInfo) -> bool {
    match &port.port_type {
        serialport::SerialPortType::UsbPort(info) => info.vid == ARDUINO_VID,
        _ => false,
    }
}

pub fn port_to_candidate(port: SerialPortInfo) -> Option<CandidateDevice> {
    let name = port.port_name;

    let (vid, pid) = match &port.port_type {
        serialport::SerialPortType::UsbPort(info) => (Some(info.vid), Some(info.pid)),
        _ => (None, None),
    };

    if let Some(v) = vid {
        if v != ARDUINO_VID {
            return None;
        }
    }

    Some(CandidateDevice {
        id: format!("{}-{}", DEVICE_NAMESPACE, name),
        dev: name,
        vid,
        pid,
    })
}

pub struct InputHandler {
    button_states: [bool; KEY_COUNT],
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            button_states: [false; KEY_COUNT],
        }
    }

    pub fn process_input(&mut self, row: u8, col: u8, state: u8) -> Option<Vec<bool>> {
        if row as usize >= ROW_COUNT || col as usize >= COL_COUNT {
            return None;
        }

        let index = (row as usize * COL_COUNT) + col as usize;

        self.button_states[index] = state != 0;

        Some(self.button_states.to_vec())
    }
}
