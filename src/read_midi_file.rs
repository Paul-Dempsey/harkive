use std:: {
    time::Duration,
};
use windows::{
    core::*,
    Devices::Midi::*,
    Win32::Foundation::E_FAIL,
};
use crate:: {
     midi::*,
     util::*,
     midi_handler::WinMidi,
};

#[derive(Default)]
pub struct ReadMidiFile {
    data: Vec<u8>,
    decoder: VariableLengthValue,
    running_status: u8,
    index: usize,
    end: usize,
}

impl ReadMidiFile {
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
            decoder: VariableLengthValue::default(),
            running_status: 0,
            index: 0,
            end: data.len(),
        }
    }

    // pub fn start(&mut self, data: &[u8]) {
    //     self.data.clear();
    //     self.data.extend_from_slice(data);
    //     self.index = 0;
    //     self.running_status = 0;
    //     self.decoder.start();
    // }

    // pub fn reset(&mut self) {
    //     self.index = 0;
    //     self.running_status = 0;
    //     self.decoder.start();
    // }

    pub fn next(&mut self) -> Result<Option<(Duration, WinMidi)>> {
        if 0 == self.index {
            if 24 > self.data.len() {
                return Err(Error::new(E_FAIL, h!("Not a MIDI file").clone()));
            }
            self.read_header()?;
            self.read_track_header()?;
        }
        if self.index >= self.end { return Ok(None); }
        self.read_event()
    }
    
    fn running_status(&mut self) -> u8 {
        let status = self.data[self.index];
        if is_bit8(status) {
            self.index += 1;
            self.running_status = status;
            status
        } else {
            self.running_status
        }
    }

    // fn clear_running_status(&mut self) {
    //     self.running_status = 0;
    // }

    fn read_var_len(&mut self) -> Result<usize> {
        loop {
            let byte = self.data[self.index];
            self.index += 1;
            if !self.decoder.add_byte(byte) {
                break;
            }
        }
        let length = self.decoder.finish()? as usize;
        Ok(length)
    }

    fn read_header(&mut self) -> Result<()> {
        if !is_midi_header(&self.data[0..]) {
            Err(Error::new(E_FAIL, h!("Not a MIDI file").clone()))
        } else {
            // This commented code reads the full header info, but we don't use it.
            // Left here for future reference.

            //let mut index: usize = 4;
            //let length = get_u32(&data[index..index + 4]);
            //index += 4;
            //let format = get_u16(&data[index..index + 2]);
            //index += 2;
            //let track_count = get_u16(&data[index..index + 2]);
            //index += 2;
            //let division = get_u16(&data[index..index + 2]);
            //index += 2;
            // println!(
            //     "MThd: length {length}, format {format}, tracks {track_count}, division {division}"
            // );
            self.index = 14;
            Ok(())
        }
    }

    fn read_track_header(&mut self) -> Result<()> {
        if !is_midi_track_header(&self.data[self.index..]) {
            Err(Error::new(E_FAIL, h!("Expecting MTrk").clone()))
        } else {
            self.index += 4;
            let length = self.next_u32();
            //println!("MTrk: len {length}");
            self.end = self.index + length;
            debug_assert!(self.end <= self.data.len());
            Ok(())
        }
    }

    fn delta_to_ms(delta:usize) -> std::time::Duration {
        const STD_BEAT_MS:f32 = 500_000.0/96.0/1_000.0;
        let ms = ((delta as f32) * STD_BEAT_MS) as u64;
        // #[cfg(debug_assertions)]
        // {
        //     println!("Delta={ms}");
        // }
        std::time::Duration::from_millis(ms)
    }

    fn next_byte(&mut self) -> u8 {
        let next = self.data[self.index];
        self.index += 1;
        next
    }
    fn next_u32(&mut self) -> usize {
        let result = get_u32(&self.data[self.index..self.index+4]) as usize;
        self.index += 4;
        result
    }

    fn read_event(&mut self) -> Result<Option<(Duration, WinMidi)>> {
        let delta = self.read_var_len()?;
        let dt = if delta == 0 {
            std::time::Duration::ZERO
        } else { 
            //let delta = usize::max(delta, 10);
            Self::delta_to_ms(delta)
        };
        let status = self.running_status();
        match status {
            0xF0 //SysEx
            | 0xF1 // MIDI Time Code Quarter Frame
            | 0xF2 // Song Position Pointer
            | 0xF3 // Song Select
            | 0xF4 // undefined
            | 0xF5 // undefined
            | 0xF6 // Tune Request
            | 0xF7 // End SysEx
            | 0xF8 // Timing Clock
            | 0xFA // Start
            | 0xFB // Continue
            | 0xFC // Stop
            | 0xFE // Active Sensing
                => {
                    let msg = format!("Unsupported status {status:2X} in preset file");
                    Err(Error::new(E_FAIL, HSTRING::from(msg)))
                }

            0xFF => {
                let code = self.next_byte();
                if code == 0x2F {
                    let extra = self.next_byte();
                    debug_assert!(0 == extra);
                    //println!("F| End of track");
                    Ok(None)
                } else {
                    let msg = format!("Unsupported status {status:2X}:{code:2X} in preset file");
                    Err(Error::new(E_FAIL, HSTRING::from(msg)))
                }
            }

            _ => {
                let kind = hi_nybble(status);
                let channel = lo_nybble(status);
                match kind {
                    0x80 => {
                        //Note off
                        let note = self.next_byte();
                        let velocity = self.next_byte();
                        if let Ok(midi) = MidiNoteOffMessage::CreateMidiNoteOffMessage(channel, note, velocity) {
                            Ok(Some((dt, WinMidi::NoteOff(midi))))
                        } else {
                            Err(Error::new(E_FAIL, h!("Failed to create MidiNoteOffMessage").clone()))
                        }
                    },
                    0x90 => {
                        //Note on
                        let note = self.next_byte();
                        let velocity = self.next_byte();
                        if let Ok(midi) = MidiNoteOnMessage::CreateMidiNoteOnMessage(channel, note, velocity) {
                            Ok(Some((dt, WinMidi::NoteOn(midi))))
                        } else {
                            Err(Error::new(E_FAIL, h!("Failed to create MidiNoteOnMessage").clone()))
                        }
                    },
                    0xA0 => {
                        //Poly Key Pressure
                        let note = self.next_byte();
                        let pressure = self.next_byte();
                        if let Ok(midi) = MidiPolyphonicKeyPressureMessage::CreateMidiPolyphonicKeyPressureMessage(channel, note, pressure) {
                            Ok(Some((dt, WinMidi::PolyphonicKeyPressure(midi))))
                        } else {
                            Err(Error::new(E_FAIL, h!("Failed to create MidiPolyphonicKeyPressureMessage").clone()))
                        }
                    },
                    0xB0 => {
                        // CC
                        let cc = self.next_byte();
                        let value = self.next_byte();
                        if let Ok(midi) = MidiControlChangeMessage::CreateMidiControlChangeMessage(channel, cc, value) {
                            Ok(Some((dt, WinMidi::ControlChange(midi))))
                        } else {
                            Err(Error::new(E_FAIL, h!("Failed to create MidiControlChangeMessage").clone()))
                        }
                    },
                    0xC0 => {
                        // Program change
                        let program = self.next_byte();
                        if let Ok(midi) = MidiProgramChangeMessage::CreateMidiProgramChangeMessage(channel, program) {
                            Ok(Some((dt, WinMidi::ProgramChange(midi))))
                        } else {
                            Err(Error::new(E_FAIL, h!("Failed to create MidiProgramChangeMessage").clone()))
                        }
                    }
                    0xD0 => {
                        // Channel pressure
                        let pressure = self.next_byte();
                        if let Ok(midi) = MidiChannelPressureMessage::CreateMidiChannelPressureMessage(channel, pressure) {
                            Ok(Some((dt, WinMidi::ChannelPressure(midi))))
                        } else {
                            Err(Error::new(E_FAIL, h!("Failed to create MidiChannelPressureMessage").clone()))
                        }
                    }
                    0xE0 => {
                        //Pitch bend
                        let lo = self.next_byte();
                        let hi = self.next_byte();
                        let bend = u16_from_midi_bytes(lo, hi);
                        if let Ok(midi) = MidiPitchBendChangeMessage::CreateMidiPitchBendChangeMessage(channel, bend) {
                            Ok(Some((dt, WinMidi::PitchBendChange(midi))))
                        } else {
                            Err(Error::new(E_FAIL, h!("Failed to create MidiPitchBendChangeMessage").clone()))
                        }
                    }
                    _ => {
                        let msg = format!("Unknown status {status:2X}");
                        Err(Error::new(E_FAIL, HSTRING::from(msg)))
                    }
                }
            }
        }

    }

}