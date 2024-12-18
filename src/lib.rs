pub struct IbusParser {
    buffer: [u8; 32],
    index: usize,
    state: IbusState,
}

enum IbusState {
    Sync,
    Data,
}

impl IbusParser {
    pub fn new() -> Self {
        Self {
            buffer: [0; 32],
            index: 0,
            state: IbusState::Sync,
        }
    }

    pub fn parse_byte(&mut self, byte: u8) -> Option<IbusMessage> {
        match self.state {
            IbusState::Sync => {
                if byte == 0x20 {
                    self.buffer[0] = byte;
                    self.index = 1;
                    self.state = IbusState::Data;
                }
            }
            IbusState::Data => {
                self.buffer[self.index] = byte;
                self.index += 1;

                if self.index == 32 {
                    let mut checksum: u16 = 0xffff;
                    for byte in &self.buffer[0..30] {
                        checksum -= *byte as u16;
                    }

                    if checksum == ((self.buffer[31] as u16) << 8) | (self.buffer[30] as u16) {
                        let message = IbusMessage::from_buffer(&self.buffer);
                        self.state = IbusState::Sync;
                        return Some(message);
                    } else {
                        self.state = IbusState::Sync;
                    }
                }
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct IbusMessage {
    pub channels: [u16; 14],
}

impl IbusMessage {
    pub fn from_buffer(buffer: &[u8]) -> Self {
        let mut channels = [0; 14];
        for ch in 0..14 {
            channels[ch] =
                ((buffer[2 * (ch + 1) + 1] as u16) << 8) | buffer[2 * (ch + 1) + 0] as u16;
        }
        Self { channels }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_message() {
        let stream: [u8; 32] = [
            0x20, 0x40, 0xDB, 0x5, 0xDC, 0x5, 0x54, 0x5, 0xDC, 0x5, 0xE8, 0x3, 0xD0, 0x7, 0xD2,
            0x5, 0xE8, 0x3, 0xDC, 0x5, 0xDC, 0x5, 0xDC, 0x5, 0xDC, 0x5, 0xDC, 0x5, 0xDC, 0x5, 0xDA,
            0xF3,
        ];
        let ref_msg: [u16; 14] = [
            1499, 1500, 1364, 1500, 1000, 2000, 1490, 1000, 1500, 1500, 1500, 1500, 1500, 1500,
        ];

        let mut ibus = IbusParser::new();
        for byte in stream {
            if let Some(message) = ibus.parse_byte(byte) {
                let channels = message.channels;
                for i in 0..14 {
                    assert_eq!(channels[i], ref_msg[i]);
                }
            }
        }
    }
}
