use crate::{
    acquire_device::OutPortDescription, gather_state::GatherState, haken_midi::cc16, midi_file::*,
    options::*,
};
use crate::{
    continuum_preset::*,
    data_kind::DataKind,
    midi::{CHANNEL15, CHANNEL16},
    midi_handler::*,
};
use std::io::Write;
use windows::{core::*, Devices::Midi::*};

#[derive(Clone, Copy, Default, PartialOrd, PartialEq)]
#[allow(dead_code)]
enum Noise {
    Silent,
    Terse,
    #[default]
    Verbose,
}
#[allow(dead_code)]
impl Noise {
    fn terse(&self) -> bool {
        *self >= Self::Terse
    }
    fn verbose(&self) -> bool {
        *self == Self::Verbose
    }
    fn silent(&self) -> bool {
        *self == Self::Silent
    }
}

#[derive(Clone, Copy, Default, PartialOrd, PartialEq)]
pub enum ArchiveState {
    #[default]
    Unknown,
    Ok,
    Fail,
}

pub struct MatrixHandler {
    output: OutPortDescription,
    verb: Action,
    gather_state: GatherState,
    bin_type: DataKind,
    preset_builder: PresetBuilder,
    presets: Vec<ContinuumPreset>,
    in_preset_names: bool,
    in_archive: bool,
    progress_count: usize,
    next_bank: u8,
    midi_file: MidiFile,
    done: bool,
    noise: Noise,
    tick_tock: bool,
    receive_sync: bool,
    archive_state: ArchiveState,
}

impl Drop for MatrixHandler {
    fn drop(&mut self) {
        if let Err(error) = self.output.port.Close() {
            println!("Error closing MIDI out handle: {}", error.message());
        }
    }
}

impl MatrixHandler {
    pub fn new(output: OutPortDescription) -> Self {
        Self {
            output,
            verb: Action::Nothing,
            gather_state: GatherState::default(),
            bin_type: DataKind::default(),
            preset_builder: PresetBuilder::default(),
            presets: Vec::default(),
            in_preset_names: false,
            in_archive: false,
            progress_count: 0,
            next_bank: u8::MAX,
            midi_file: MidiFile::default(),
            done: false,
            noise: Noise::Verbose,
            tick_tock: true,
            receive_sync: false,
            archive_state: ArchiveState::Unknown,
        }
    }
    pub fn output_port(&self) -> &MidiOutPort {
        &self.output.port
    }

    pub fn is_ready(&self) -> bool {
        self.done
    }
    pub fn unready(&mut self) {
        self.done = false;
    }

    pub fn get_archive_data(&mut self) -> Vec<u8> {
        self.midi_file.finish()
    }

    pub fn archive_state(&self) -> ArchiveState {
        self.archive_state
    }
    pub fn clear_archive_state(&mut self) {
        self.archive_state = ArchiveState::Unknown
    }

    pub fn get_presets(&self) -> &Vec<ContinuumPreset> {
        &self.presets
    }

    pub fn clear_presets(&mut self) {
        self.presets.clear();
    }

    fn is_saving(&self) -> bool {
        self.verb == Action::SaveCurrent || self.verb == Action::Save
    }

    pub fn terse(&self) -> bool {
        self.noise.terse()
    }
    pub fn verbose(&self) -> bool {
        self.noise.verbose()
    }
    //pub fn silent(&self) -> bool { self.noise.silent() }
    fn terse_message(&self, message: &str) {
        if self.terse() {
            println!("{message}");
        }
    }

    pub fn on_idle(&mut self) {
        if self.verb == Action::Load {
            self.done = true;
        }
    }

    pub fn send_cc(&self, channel: u8, cc: u8, value: u8) -> Result<()> {
        self.output
            .port
            .SendMessage(&MidiControlChangeMessage::CreateMidiControlChangeMessage(
                channel, cc, value,
            )?)
    }

    pub fn send_string(&self, kind: u8, text: &str) -> Result<()> {
        self.send_cc(CHANNEL16, 56, kind)?;
        for ch in text.bytes() {
            self.output.port.SendMessage(
                &MidiChannelPressureMessage::CreateMidiChannelPressureMessage(CHANNEL16, ch)?,
            )?;
        }
        self.send_cc(CHANNEL16, 56, 127)?;
        Ok(())
    }

    pub fn start_action(&mut self, act: Action) -> Result<()> {
        match act {
            Action::Nothing | Action::Usage | Action::Docs | Action::ListMidi | Action::Monitor => {
                unreachable!()
            }
            Action::ListNames => self.start_list_names(),
            Action::SaveCurrent => self.start_save_current(),
            Action::Save => self.start_save_presets(),
            Action::Load => self.start_load_presets(),
            Action::Clear => self.start_clear(),
        }
    }
    fn action_prelude(&mut self, act: Action) -> Result<()> {
        self.verb = act;
        self.done = false;
        self.gather_state = GatherState::default();
        self.bin_type = DataKind::default();
        self.presets.clear();
        self.archive_state = ArchiveState::Unknown;
        self.transmit_quiet()?;
        self.send_cc(CHANNEL16, 116, 85)?; // editor present
        Ok(())
    }

    fn start_list_names(&mut self) -> Result<()> {
        self.action_prelude(Action::ListNames)?;
        self.transmit_names()?;
        self.send_cc(CHANNEL16, 116, 85) // editor present ensures End of Preset Names is sent.
    }

    fn start_save_current(&mut self) -> Result<()> {
        self.action_prelude(Action::SaveCurrent)?;
        self.transmit_archive_current()?;
        Ok(())
    }

    fn start_save_presets(&mut self) -> Result<()> {
        self.action_prelude(Action::Save)?;
        self.transmit_names()?;
        Ok(())
    }

    fn start_load_presets(&mut self) -> Result<()> {
        self.action_prelude(Action::Load)?;
        Ok(())
    }

    fn start_clear(&mut self) -> Result<()> {
        self.action_prelude(Action::Clear)?;
        self.receive_sync = false;
        self.clear_bank(0)?;
        self.next_bank = 1;
        Ok(())
    }

    pub fn editor_present(&mut self) -> Result<()> {
        let value = if self.tick_tock { 85 } else { 42 };
        self.tick_tock = !self.tick_tock;
        self.send_cc(CHANNEL16, 116, value) // editor present
    }

    pub fn choose_edit_slot(&self) -> Result<()> {
        self.send_cc(CHANNEL16, 0, 126)?;
        self.send_cc(CHANNEL16, 32, 0)?;
        self.output
            .port
            .SendMessage(&MidiProgramChangeMessage::CreateMidiProgramChangeMessage(
                CHANNEL16, 0,
            )?)
    }

    pub fn set_edit_slot(&self) -> Result<()> {
        self.send_cc(CHANNEL16, 0, 126)?;
        self.send_cc(CHANNEL16, 32, 0)?;
        self.output
            .port
            .SendMessage(&MidiProgramChangeMessage::CreateMidiProgramChangeMessage(
                CHANNEL15, 1,
            )?)
    }

    pub fn choose_preset(&self, index: u8) -> Result<()> {
        // bank
        self.send_cc(CHANNEL16, 0, 0)?;
        // category
        self.send_cc(CHANNEL16, 32, 0)?;
        // preset#
        self.output
            .port
            .SendMessage(&MidiProgramChangeMessage::CreateMidiProgramChangeMessage(
                CHANNEL16, index,
            )?)
    }

    /// bank = 0-based user bank 0..7
    pub fn clear_bank(&self, bank: u8) -> Result<()> {
        println!("[>Clearing preset bank {bank}]");
        self.send_cc(CHANNEL16, 109, 115 + bank)
    }

    fn transmit_names(&mut self) -> Result<()> {
        self.terse_message("[>Request names]");
        self.send_cc(CHANNEL16, 109, 32)?; // user presets
        self.editor_present()?; // editor present
        Ok(())
    }

    fn transmit_quiet(&self) -> Result<()> {
        for channel in 0..=12 {
            for cc in [120, 121, 122] {
                self.send_cc(channel, cc, 0)?;
            }
        }
        Ok(())
    }

    fn transmit_archive_current(&self) -> Result<()> {
        self.terse_message("[>Archive active preset]");
        self.send_cc(CHANNEL16, 110, 100)?;
        Ok(())
    }

    fn gather_state_for_data(kind: DataKind) -> GatherState {
        match kind {
            DataKind::Name => GatherState::Name,
            DataKind::ControlText => GatherState::Text,
            DataKind::Category => GatherState::Category,
            _ => GatherState::Binary,
        }
    }

    fn on_ch16_control_change(&mut self, _channel: u8, cc: u8, value: u8) {
        match cc {
            cc16::BankSelect => {
                self.preset_builder.set_bank_hi(value);
            }
            cc16::PresetGroup => {
                self.preset_builder.set_bank_lo(value);
            }
            cc16::DataStream => {
                self.bin_type = DataKind::new(value);
                self.gather_state = Self::gather_state_for_data(self.bin_type);
            }
            cc16::DownloadControl => match value {
                cc16::DownloadControl_ArchiveOk => {
                    self.archive_state = ArchiveState::Ok;
                    if Action::Load == self.verb {
                        println!(">>AchiveOk");
                        self.done = true;
                    }
                }
                cc16::DownloadControl_ArchiveFail => {
                    self.archive_state = ArchiveState::Fail;
                    if Action::Load == self.verb {
                        println!(">>AchiveFail");
                        self.done = true;
                    }
                }
                cc16::DownloadControl_DspDone => {
                    self.receive_sync = true;

                    if Action::Clear == self.verb {
                        if self.next_bank < 8 {
                            self.receive_sync = false;
                            _ = self.clear_bank(self.next_bank);
                            self.next_bank += 1;
                        } else {
                            self.next_bank = u8::MAX;
                            self.done = true;
                        }
                    }
                }
                cc16::DownloadControl_BeginSystemNames | cc16::DownloadControl_BeginUserNames => {
                    self.terse_message("[---- Begin preset names ----]");
                    self.in_preset_names = true;
                    self.progress_count = 0;
                }
                cc16::DownloadControl_EndSystemNames | cc16::DownloadControl_EndUserNames => {
                    if self.in_preset_names && self.verbose() && self.progress_count > 0 {
                        println!();
                    }
                    self.terse_message("[---- End preset names ----]");
                    self.in_preset_names = false;
                    self.done = true;
                }
                _ => {}
            },
            cc16::DownloadInfo => {
                match value {
                    cc16::DownloadInfo_BeginArchive => {
                        self.terse_message("[---- Begin archive ----]");
                        if self.is_saving() {
                            self.in_archive = true;
                            self.midi_file.clear();
                        }
                    }
                    cc16::DownloadInfo_EndArchive => {
                        self.terse_message("[---- End archive ----]");
                        if self.is_saving() {
                            self.in_archive = false;
                            self.done = true;
                        }
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    }
}

#[allow(unused_variables)]
impl MidiHandler for MatrixHandler {
    fn on_note_off(&mut self, ticks: i64, channel: u8, note: u8, velocity: u8) -> Result<()> {
        Ok(())
    }

    fn on_note_on(&mut self, ticks: i64, channel: u8, note: u8, velocity: u8) -> Result<()> {
        Ok(())
    }

    fn on_polyphonic_key_pressure(
        &mut self,
        ticks: i64,
        channel: u8,
        note: u8,
        pressure: u8,
    ) -> Result<()> {
        unreachable!();
        // let channel = 1 + channel;
        // Self::log_channel_message2(ticks, "PolyKeyPressure", channel, note, pressure);
        // Ok(())
    }

    fn on_control_change(&mut self, ticks: i64, channel: u8, cc: u8, value: u8) -> Result<()> {
        if self.in_archive {
            self.midi_file
                .on_control_change(ticks, channel, cc, value)?;
        }
        let channel = 1 + channel;
        //Self::log_channel_message2(ticks, "CC", channel, cc, value);
        if channel == 16 {
            self.on_ch16_control_change(channel, cc, value);
        }
        // else if channel == 15 {
        //     self.on_ch15_control_change(channel, cc, value);
        // }
        Ok(())
    }

    fn on_program_change(&mut self, ticks: i64, channel: u8, program: u8) -> Result<()> {
        if self.in_archive {
            self.midi_file.on_program_change(ticks, channel, program)?;
        }
        let channel = channel + 1;
        if channel == 16 {
            self.preset_builder.set_number(program);
            if let Some(preset) = self.preset_builder.finish() {
                if self.in_preset_names {
                    if self.verbose() {
                        self.progress_count += 1;
                        _ = std::io::stdout().write(&[b'.']);
                        if self.progress_count == 16 {
                            println!();
                            self.progress_count = 0;
                        } else {
                            _ = std::io::stdout().flush();
                        }
                    }
                    if preset.name != "-" {
                        self.presets.push(preset);
                    }
                } else if self.terse() {
                    preset.print();
                    //preset.print_friendly_categories(&self.catcode)
                }
            }
            if self.verb == Action::Load && 2 == self.presets.len() {
                self.done = true;
            }
        }
        Ok(())
    }

    fn on_channel_pressure(&mut self, ticks: i64, channel: u8, pressure: u8) -> Result<()> {
        if self.in_archive {
            self.midi_file
                .on_channel_pressure(ticks, channel, pressure)?;
        }
        let channel = channel + 1;
        if channel == 16 {
            match self.gather_state {
                GatherState::Name => {
                    self.preset_builder.name_add(pressure as char);
                }
                GatherState::Text => {
                    self.preset_builder.text_add(pressure as char);
                }
                GatherState::Category => {
                    self.preset_builder.category_add(pressure as char);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn on_pitch_bend_change(&mut self, ticks: i64, channel: u8, bend: u16) -> Result<()> {
        Ok(())
    }

    fn on_system_exclusive(&mut self, ticks: i64, data: Vec<u8>) -> Result<()> {
        unreachable!();
        // Ok(())
    }

    fn on_midi_time_code(&mut self, ticks: i64, frame: u8, values: u8) -> Result<()> {
        unreachable!();
        // Ok(())
    }

    fn on_song_position_pointer(&mut self, ticks: i64, beats: u16) -> Result<()> {
        unreachable!();
        // Ok(())
    }

    fn on_song_select(&mut self, ticks: i64, song: u8) -> Result<()> {
        unreachable!();
        // Ok(())
    }

    fn on_tune_request(&mut self, ticks: i64) -> Result<()> {
        unreachable!();
        // Ok(())
    }

    fn on_end_system_exclusive(&mut self, ticks: i64, data: Vec<u8>) -> Result<()> {
        unreachable!();
        // Ok(())
    }

    fn on_timing_clock(&mut self, ticks: i64) -> Result<()> {
        unreachable!();
        // Ok(())
    }

    fn on_start(&mut self, ticks: i64) -> Result<()> {
        unreachable!();
        // Ok(())
    }

    fn on_continue(&mut self, ticks: i64) -> Result<()> {
        unreachable!();
        // Ok(())
    }

    fn on_stop(&mut self, ticks: i64) -> Result<()> {
        unreachable!();
        // Ok(())
    }

    fn on_active_sensing(&mut self, ticks: i64) -> Result<()> {
        unreachable!();
        // Ok(())
    }

    fn on_system_reset(&mut self, ticks: i64) -> Result<()> {
        unreachable!();
        // Ok(())
    }
}
