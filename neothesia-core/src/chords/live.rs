//! Sounding-note state, independent of the rendered keyboard range and MIDI I/O.
//! Sustain is tracked per channel. Channel 10 (zero-based 9) is percussion.

struct Channel {
    held: [bool; 128],
    sounding: [bool; 128],
    sustain: bool,
}

// Written explicitly for toolchains where large arrays do not implement Default.
impl Channel {
    fn new() -> Self {
        Self {
            held: [false; 128],
            sounding: [false; 128],
            sustain: false,
        }
    }
}

pub struct LiveNotes {
    channels: [Channel; 16],
}

impl Default for LiveNotes {
    fn default() -> Self {
        Self {
            channels: std::array::from_fn(|_| Channel::new()),
        }
    }
}

impl LiveNotes {
    pub fn note_on(&mut self, channel: u8, key: u8, velocity: u8) {
        if velocity == 0 {
            self.note_off(channel, key);
            return;
        }
        if channel == 9 || key > 127 {
            return;
        }
        if let Some(c) = self.channels.get_mut(channel as usize) {
            c.held[key as usize] = true;
            c.sounding[key as usize] = true;
        }
    }

    pub fn note_off(&mut self, channel: u8, key: u8) {
        if key > 127 {
            return;
        }
        if let Some(c) = self.channels.get_mut(channel as usize) {
            c.held[key as usize] = false;
            if !c.sustain {
                c.sounding[key as usize] = false;
            }
        }
    }

    pub fn controller(&mut self, channel: u8, controller: u8, value: u8) {
        let Some(c) = self.channels.get_mut(channel as usize) else {
            return;
        };
        match controller {
            64 => {
                c.sustain = value >= 64;
                if !c.sustain {
                    c.sounding = c.held;
                }
            }
            120 => {
                // All Sound Off immediately silences even sustained notes.
                c.held.fill(false);
                c.sounding.fill(false);
            }
            123 => {
                // All Notes Off behaves like releasing every key.
                c.held.fill(false);
                if !c.sustain {
                    c.sounding.fill(false);
                }
            }
            121 => {
                // Reset All Controllers releases sustain, preserves held keys.
                c.sustain = false;
                c.sounding = c.held;
            }
            _ => {}
        }
    }

    pub fn notes(&self) -> Vec<u8> {
        (0..128)
            .filter(|key| self.channels.iter().any(|c| c.sounding[*key]))
            .map(|k| k as u8)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn releases_zero_velocity_and_channels() {
        let mut live = LiveNotes::default();
        live.note_on(0, 60, 100);
        live.note_on(1, 60, 100);
        live.note_off(0, 60);
        assert_eq!(live.notes(), [60]);
        live.note_on(1, 60, 0);
        assert!(live.notes().is_empty());
        live.note_on(9, 60, 100);
        live.note_on(16, 60, 100);
        live.note_on(0, 128, 100);
        assert!(live.notes().is_empty());
    }
    #[test]
    fn sustain_retrigger_and_channel_isolation() {
        let mut live = LiveNotes::default();
        live.note_on(0, 60, 100);
        live.controller(0, 64, 127);
        live.note_off(0, 60);
        live.controller(1, 64, 0);
        assert_eq!(live.notes(), [60]);
        live.note_on(0, 60, 100);
        live.controller(0, 64, 0);
        assert_eq!(live.notes(), [60]);
        live.note_off(0, 60);
        assert!(live.notes().is_empty());
        live.note_on(0, 64, 100);
        live.controller(0, 64, 64);
        live.note_off(0, 64);
        live.controller(0, 64, 63);
        assert!(live.notes().is_empty());
    }
    #[test]
    fn panic_and_reset_controllers() {
        let mut live = LiveNotes::default();
        live.note_on(0, 60, 100);
        live.controller(0, 64, 127);
        live.controller(0, 123, 0);
        assert_eq!(live.notes(), [60]);
        live.controller(0, 121, 0);
        assert!(live.notes().is_empty());
        live.note_on(0, 64, 100);
        live.controller(0, 121, 0);
        assert_eq!(live.notes(), [64]);
        live.controller(0, 120, 0);
        assert!(live.notes().is_empty());
    }
}
