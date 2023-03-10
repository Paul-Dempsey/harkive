use crate::{cc_text::*, gather_state::GatherState, haken_midi::cc16};
use crate::{
    continuum_preset::*, data_kind::DataKind, midi::*, midi_handler::*, midi_traits::*,
    util::make_hex_string,
};
use windows::core::*;

const PITCH_BEND_CENTER: u16 = 8192;
const PITCH_BEND_CENTER_EX: i64 = ((PITCH_BEND_CENTER as u64) << 7) as i64;

fn mpe_pitch_bend(bend: u16, lsb: u16) -> i64 {
    ((((bend as u64) << 7) | (lsb as u64)) as i64) - PITCH_BEND_CENTER_EX
}

pub struct MidiMonitor {
    gather: GatherState,
    bin_type: DataKind,
    binbuild: BinBuild,
    preset_builder: PresetBuilder,
    cc_text: CcText,
    catcode: HCCategoryCode,
    firmware_version: u16,
    cc87: u8,
    bend_range: u8,
    in_matrix: bool,
}

impl Default for MidiMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiMonitor {
    pub fn new() -> Self {
        Self {
            gather: GatherState::None,
            bin_type: DataKind::Unknown,
            binbuild: BinBuild::default(),
            preset_builder: PresetBuilder::default(),
            cc_text: CcText::default(),
            catcode: HCCategoryCode::default(),
            firmware_version: u16::MAX,
            cc87: 0,
            bend_range: 96,
            in_matrix: false,
        }
    }
    fn on_ch16_control_change(&mut self, ticks: i64, channel: u8, cc: u8, value: u8) -> Result<()> {
        Self::continuum_cc(ticks, channel, cc, value);
        match (cc, value) {
            (cc16::BankSelect, _) => {
                self.preset_builder.set_bank_hi(value);
            }
            (cc16::PresetGroup, _) => {
                self.preset_builder.set_bank_lo(value);
            }
            (cc16::BendRange, _) => {
                self.bend_range = value;
                let range = match value {
                    1..=96 => value.to_string(),
                    _ => format!("MPE+ ch1 {}", (value as i32) - 96),
                };
                println!(
                    "{:>6}| ch{:<2} cc{:<3} [Pitch bend range] {}",
                    ticks / 10_000,
                    channel,
                    cc,
                    range
                );
            }

            (cc16::DataStream, _) => {
                self.stream(value);
            }

            (cc16::FirmwareVersionHi, _) => {
                self.firmware_version = value as u16;
            }
            (cc16::FirmwareVersionLo, _) => {
                self.firmware_version = self.firmware_version << 7 | value as u16;
                println!("Firmware version: {}", self.firmware_version);
            }

            (cc16::DownloadControl, 104) => {
                self.preset_builder.set_nofn(NofN::Single);
            }
            (cc16::DownloadControl, 105) => {
                self.preset_builder.set_nofn(NofN::Double);
            }
            (cc16::DownloadControl, 106) => {
                self.preset_builder.set_nofn(NofN::Triple);
            }
            (cc16::DownloadControl..=cc16::DownloadInfo, _) => {
                if let Some(message) = self.cc_text.get(cc, value) {
                    println!("{message}");
                }
            }
            (cc16::DeviceStatus, _) => {
                let led = value & 0x0F;
                println!(
                    "[LED {}]",
                    match led {
                        0 => "Off",          //ledOff
                        1 => "Blue",         //ledBlue
                        2 => "Red",          //ledRed
                        3 => "Bright Green", //ledBrightGreen
                        4 => "Green",        //ledGreen
                        5 => "White",        //ledWhite
                        6 => "Yellow",       //ledYellow
                        7 => "Purple",       //ledPurple
                        8 => "Blue Green",   //ledBlueGreen
                        _ => "?",
                    }
                );
                let aes = (value & 0x70) >> 4;
                if 0 != aes {
                    println!(
                        "[AES {} kHz]",
                        match aes {
                            1 => "non-standard",
                            2 => "44.1",
                            3 => "48.0",
                            4 => "88.2",
                            5 => "96.0",
                            6 => "176.4",
                            7 => "192.0",
                            _ => "?",
                        }
                    );
                }
            }
            (cc16::DspPercent, _) => {
                let dsp = value >> 5;
                let pct = (value & 0x1F) * 4;
                println!("DSP {dsp} {pct}%")
            }
            _ => {}
        }
        Ok(())
    }

    fn continuum_cc(ticks: i64, channel: u8, cc: u8, value: u8) {
        println!(
            "{:>6}| ch{:<2} cc{:<3} [{}] {}",
            ticks / 10_000,
            channel,
            cc,
            continuum_cc_name(cc),
            value
        );
    }

    // fn on_ch15_control_change(&mut self, ticks:i64, channel:u8, cc:u8, value:u8) -> Result<()> {
    //     println!("{:>6}| ch{:<2} cc{:<3} [Matrix] {}", ticks/10_000, channel, cc, value);
    //     Ok(())
    // }

    fn stream(&mut self, value: u8) {
        match value {
            0 => {
                self.gather = GatherState::Name;
            }
            1 => {
                self.gather = GatherState::Text;
            }
            127 => self.end_stream(),
            _ => {
                // other data
                self.bin_type = DataKind::new(value);
                if self.bin_type == DataKind::Unknown {
                    println!("?Binary data {value}");
                }
                self.gather = GatherState::Binary;
            }
        }
    }
    fn end_stream(&mut self) {
        match self.gather {
            GatherState::Binary => {
                let data = self.binbuild.flush();
                println!(
                    "Binary data {}: {} [{}]",
                    self.bin_type.name(),
                    data.len(),
                    make_hex_string(&data)
                );
                self.bin_type = DataKind::Unknown;
            }
            GatherState::None | GatherState::Name | GatherState::Text | GatherState::Category => {}
        };
        self.gather = GatherState::None;
    }

    fn simple_message(ticks: i64, label: &str) -> Result<()> {
        println!("{:>6}| {}", ticks / 10_000, label);
        Ok(())
    }
    fn log_message1(ticks: i64, label: &str, value: u8) -> Result<()> {
        println!("{:>6}| {} {}", ticks / 10_000, label, value);
        Ok(())
    }
    fn log_channel_message1(ticks: i64, label: &str, channel: u8, value: u8) -> Result<()> {
        println!(
            "{:>6}| ch{:<2} {} {}",
            ticks / 10_000,
            channel,
            label,
            value
        );
        Ok(())
    }
    fn log_channel_message2(
        ticks: i64,
        label: &str,
        channel: u8,
        value: u8,
        value2: u8,
    ) -> Result<()> {
        println!(
            "{:>6}| ch{:<2} {} {} {}",
            ticks / 10_000,
            channel,
            label,
            value,
            value2
        );
        Ok(())
    }
}

impl MidiHandler for MidiMonitor {
    fn on_note_off(&mut self, ticks: i64, channel: u8, note: u8, velocity: u8) -> Result<()> {
        println!(
            "{:>6}| ch{:<2} Note off {} (#{}) v={}",
            ticks / 10_000,
            1 + channel,
            MidiNote::new(note).describe(),
            note,
            velocity
        );
        self.cc87 = 0;
        Ok(())
    }

    fn on_note_on(&mut self, ticks: i64, channel: u8, note: u8, velocity: u8) -> Result<()> {
        println!(
            "{:>6}| ch{:<2} Note on {} (#{}) v={}",
            ticks / 10_000,
            1 + channel,
            MidiNote::new(note).describe(),
            note,
            velocity
        );
        Ok(())
    }

    fn on_polyphonic_key_pressure(
        &mut self,
        ticks: i64,
        channel: u8,
        note: u8,
        pressure: u8,
    ) -> Result<()> {
        Self::log_channel_message2(ticks, "Poly Key Pressure", 1 + channel, note, pressure)
    }

    fn on_control_change(&mut self, ticks: i64, channel: u8, cc: u8, value: u8) -> Result<()> {
        if cc == 87 {
            self.cc87 = value;
        }
        let channel = 1 + channel;
        if self.in_matrix && channel != 15 && cc != 56 {
            println!("[End Matrix data]");
            self.in_matrix = false;
        }
        match channel {
            1 => {
                Self::continuum_cc(ticks, channel, cc, value);
                return Ok(());
            }
            15 => {
                if !self.in_matrix {
                    println!("[Begin Matrix data (ch15)]");
                    self.in_matrix = true;
                }
                //return self.on_ch15_control_change(ticks, channel, cc, value);
                return Ok(());
            }
            16 => {
                return self.on_ch16_control_change(ticks, channel, cc, value);
            }
            _ => {}
        }
        println!(
            "{:>6}| ch{:<2} cc{:<3} [{}] {}",
            ticks / 10_000,
            channel,
            cc,
            standard_cc_name(cc),
            value
        );
        Ok(())
    }

    fn on_program_change(&mut self, ticks: i64, channel: u8, program: u8) -> Result<()> {
        let channel = 1 + channel;
        Self::log_channel_message1(ticks, "Program Change", channel, program)?;
        if channel == 16 {
            self.preset_builder.set_number(program);
            if let Some(preset) = self.preset_builder.finish() {
                preset.print();
                preset.print_friendly_categories(&self.catcode);
            }
        }
        Ok(())
    }

    fn on_channel_pressure(&mut self, ticks: i64, channel: u8, pressure: u8) -> Result<()> {
        let channel = channel + 1;
        if channel == 16 {
            match self.gather {
                GatherState::Name => {
                    self.preset_builder.name_add(pressure as char);
                }
                GatherState::Text => {
                    self.preset_builder.text_add(pressure as char);
                }
                GatherState::Category => {
                    self.preset_builder.category_add(pressure as char);
                }
                GatherState::Binary => {
                    self.binbuild.add(pressure);
                }
                _ => {
                    println!(
                        "{:>6}| ch{:<2} Channel Pressure {}",
                        ticks / 10_000,
                        channel,
                        pressure
                    );
                    debug_assert!(false);
                }
            }
        } else {
            let pressure = (((pressure as u16) << 7) | (self.cc87 as u16)) as f64 / 1024.0;
            self.cc87 = 0;
            println!(
                "{:>6}| ch{:<2} Channel Pressure (Z) {:.4}",
                ticks / 10_000,
                channel,
                pressure
            );
        }
        Ok(())
    }

    fn on_pitch_bend_change(&mut self, ticks: i64, channel: u8, bend: u16) -> Result<()> {
        let channel = channel + 1;
        let hi_bend = mpe_pitch_bend(bend, self.cc87 as u16) as f64 / self.bend_range as f64;
        println!(
            "{:>6}| ch{:<2} Bend {:.3}",
            ticks / 10_000,
            channel,
            hi_bend
        );
        self.cc87 = 0;
        Ok(())
    }

    fn on_system_exclusive(&mut self, ticks: i64, data: Vec<u8>) -> Result<()> {
        println!(
            "{:>6}| SysEx {:>5}:[{}]",
            ticks / 10_000,
            data.len(),
            make_hex_string(&data)
        );
        Ok(())
    }

    fn on_midi_time_code(&mut self, ticks: i64, frame: u8, values: u8) -> Result<()> {
        println!(
            "{:>6}| MIDI time code frame={} values={}",
            ticks / 10_000,
            frame,
            values
        );
        Ok(())
    }

    fn on_song_position_pointer(&mut self, ticks: i64, beats: u16) -> Result<()> {
        println!("{:>6}| Song Position {} beats", ticks / 10_000, beats);
        Ok(())
    }

    fn on_song_select(&mut self, ticks: i64, song: u8) -> Result<()> {
        Self::log_message1(ticks, "Song Select", song)
    }

    fn on_tune_request(&mut self, ticks: i64) -> Result<()> {
        Self::simple_message(ticks, "Tune Request")
    }

    fn on_end_system_exclusive(&mut self, ticks: i64, data: Vec<u8>) -> Result<()> {
        println!(
            "{:>6}| End SysEx {:>5}:[{}]",
            ticks / 10_000,
            data.len(),
            make_hex_string(&data)
        );
        Ok(())
    }

    fn on_timing_clock(&mut self, ticks: i64) -> Result<()> {
        Self::simple_message(ticks, "Timing Clock")
    }

    fn on_start(&mut self, ticks: i64) -> Result<()> {
        Self::simple_message(ticks, "Start")
    }

    fn on_continue(&mut self, ticks: i64) -> Result<()> {
        Self::simple_message(ticks, "Continue")
    }

    fn on_stop(&mut self, ticks: i64) -> Result<()> {
        Self::simple_message(ticks, "Stop")
    }

    fn on_active_sensing(&mut self, ticks: i64) -> Result<()> {
        Self::simple_message(ticks, "Active Sensing")
    }

    fn on_system_reset(&mut self, ticks: i64) -> Result<()> {
        Self::simple_message(ticks, "System Reset")
    }
}
