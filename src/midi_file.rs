use crate::{midi::*, midi_handler::*};

#[derive(Default)]
pub struct MidiFile {
    running_status: u8,
    bytes: Vec<u8>,
    last_tick: i64,
}

const HEADER_LENGTH: usize = 14;
const TRACK_HEADER_LENGTH: usize = 8;
const TRACK_END_LENGTH: usize = 4; // (varlen delta time of 0 = 1 byte) + FF 2F 00

impl MidiFile {
    pub fn clear(&mut self) {
        self.bytes.clear();
        self.running_status = 0;
        self.last_tick = 0;
    }

    pub fn finish(&mut self) -> Vec<u8> {
        let data_length = self.bytes.len();
        let capacity = HEADER_LENGTH + TRACK_HEADER_LENGTH + data_length + TRACK_END_LENGTH;

        let mut result = Vec::with_capacity(capacity);
        Self::add_file_header(&mut result);
        Self::add_track_header(&mut result, data_length as u32);
        result.extend_from_slice(&self.bytes[0..]);
        Self::add_end_of_track(&mut result);
        debug_assert_eq!(capacity, result.len()); // if this fires, then capacity needs adjustment
        self.clear();

        result
    }
    fn add_file_header(bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(b"MThd");
        Self::add_u32(bytes, 6); // length
        Self::add_word(bytes, 0); // format
        Self::add_word(bytes, 1); // #tracks
        Self::add_word(bytes, 96); // division
    }
    fn add_track_header(bytes: &mut Vec<u8>, length: u32) {
        bytes.extend_from_slice(b"MTrk");
        Self::add_u32(bytes, length);
    }
    fn add_end_of_track(bytes: &mut Vec<u8>) {
        bytes.push(0); // variable length delta time of 0 (same as in test mid files)
        bytes.push(0xFF);
        bytes.push(0x2F);
        bytes.push(0);
    }
    fn unexpected(msg: &str) -> windows::core::Result<()> {
        println!("Unexpected {msg}");
        Ok(())
    }
    pub fn add_var_len(&mut self, value: u32) {
        if let Ok(bytes) = VariableLengthValue::encode(value) {
            self.bytes.extend_from_slice(&bytes);
        }
    }
    fn add_word(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    fn add_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn msec_to_delta_ticks(ms: u64) -> u32 {
        let dt = ms / 1920;
        debug_assert!(dt <= u32::MAX as u64);
        dt as u32
    }
    fn next_tick(&self, tick: i64) -> u32 {
        assert!(tick >= 0);
        assert!(tick >= self.last_tick);
        Self::msec_to_delta_ticks((tick - self.last_tick) as u64)
    }
    pub fn add_tick(&mut self, tick: i64) {
        if self.last_tick == 0 {
            self.last_tick = tick;
            self.add_var_len(0);
        } else {
            let next = self.next_tick(tick);
            self.add_var_len(next);
            if next > 0 {
                self.last_tick = tick;
            }
        }
    }

    // fn clear_running_status(&mut self) {
    //     self.running_status = 0;
    // }
    fn add_running_status(&mut self, status: u8) {
        if status != self.running_status {
            self.running_status = status;
            self.bytes.push(status);
        }
    }
    pub fn add_message_one(&mut self, ticks: i64, status: u8, byte: u8) {
        self.add_tick(ticks);
        self.add_running_status(status);
        self.bytes.push(byte);
    }
    pub fn add_message_two(&mut self, ticks: i64, status: u8, byte1: u8, byte2: u8) {
        self.add_tick(ticks);
        self.add_running_status(status);
        self.bytes.push(byte1);
        self.bytes.push(byte2);
    }
}

impl MidiHandler for MidiFile {
    fn on_note_off(
        &mut self,
        _ticks: i64,
        _channel: u8,
        _note: u8,
        _velocity: u8,
    ) -> windows::core::Result<()> {
        Self::unexpected("note_off")
    }
    fn on_note_on(
        &mut self,
        _ticks: i64,
        _channel: u8,
        _note: u8,
        _velocity: u8,
    ) -> windows::core::Result<()> {
        Self::unexpected("note_on")
    }
    fn on_polyphonic_key_pressure(
        &mut self,
        ticks: i64,
        channel: u8,
        note: u8,
        pressure: u8,
    ) -> windows::core::Result<()> {
        self.add_message_two(ticks, STATUS_POLY_KEY_PRESSURE | channel, note, pressure);
        Ok(())
    }

    fn on_control_change(
        &mut self,
        ticks: i64,
        channel: u8,
        cc: u8,
        value: u8,
    ) -> windows::core::Result<()> {
        self.add_message_two(ticks, STATUS_CC | channel, cc, value);
        Ok(())
    }
    fn on_program_change(
        &mut self,
        ticks: i64,
        channel: u8,
        program: u8,
    ) -> windows::core::Result<()> {
        self.add_message_one(ticks, STATUS_PROGRAM_CHANGE | channel, program);
        Ok(())
    }
    fn on_channel_pressure(
        &mut self,
        ticks: i64,
        channel: u8,
        pressure: u8,
    ) -> windows::core::Result<()> {
        self.add_message_one(ticks, STATUS_CHANNEL_PRESSURE | channel, pressure);
        Ok(())
    }
    fn on_pitch_bend_change(
        &mut self,
        ticks: i64,
        channel: u8,
        bend: u16,
    ) -> windows::core::Result<()> {
        let byte2 = (bend & 0x7F) as u8;
        let byte1 = ((bend & 0x7f80) >> 7) as u8;
        self.add_message_two(ticks, STATUS_PITCH_BEND | channel, byte1, byte2);
        Ok(())
    }
    fn on_system_exclusive(&mut self, _ticks: i64, _data: Vec<u8>) -> windows::core::Result<()> {
        Self::unexpected("system_exclusive")
    }
    fn on_midi_time_code(
        &mut self,
        _ticks: i64,
        _frame: u8,
        _values: u8,
    ) -> windows::core::Result<()> {
        Self::unexpected("midi_time_code")
    }
    fn on_song_position_pointer(&mut self, _ticks: i64, _beats: u16) -> windows::core::Result<()> {
        Self::unexpected("song_position_pointer")
    }
    fn on_song_select(&mut self, _ticks: i64, _song: u8) -> windows::core::Result<()> {
        Self::unexpected("song_select")
    }
    fn on_tune_request(&mut self, _ticks: i64) -> windows::core::Result<()> {
        Self::unexpected("tune_request")
    }
    fn on_end_system_exclusive(
        &mut self,
        _ticks: i64,
        _data: Vec<u8>,
    ) -> windows::core::Result<()> {
        Self::unexpected("end_system_exclusive")
    }
    fn on_timing_clock(&mut self, _ticks: i64) -> windows::core::Result<()> {
        Self::unexpected("timing_clock")
    }
    fn on_start(&mut self, _ticks: i64) -> windows::core::Result<()> {
        Self::unexpected("start")
    }
    fn on_continue(&mut self, _ticks: i64) -> windows::core::Result<()> {
        Self::unexpected("continue")
    }
    fn on_stop(&mut self, _ticks: i64) -> windows::core::Result<()> {
        Self::unexpected("stop")
    }
    fn on_active_sensing(&mut self, _ticks: i64) -> windows::core::Result<()> {
        Self::unexpected("active_sensing")
    }
    fn on_system_reset(&mut self, _ticks: i64) -> windows::core::Result<()> {
        Self::unexpected("system_reset")
    }
}
