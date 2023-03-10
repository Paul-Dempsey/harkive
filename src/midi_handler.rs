use windows::{core::*, Devices::Midi::*};

// IMidiMessage is neither Send nor Sync, but the concrete instances are,
// so to send a midi message across an mpsc channel (or other situation requiring Sync+Sync),
// we wrap the concrete Midi message (cast from IMiidiMessage) in an enum.
pub enum WinMidi {
    Unspecified(u8),
    NoteOff(MidiNoteOffMessage),
    NoteOn(MidiNoteOnMessage),
    PolyphonicKeyPressure(MidiPolyphonicKeyPressureMessage),
    ControlChange(MidiControlChangeMessage),
    ProgramChange(MidiProgramChangeMessage),
    ChannelPressure(MidiChannelPressureMessage),
    PitchBendChange(MidiPitchBendChangeMessage),
    SystemExclusive(MidiSystemExclusiveMessage),
    TimeCode(MidiTimeCodeMessage),
    SongPositionPointer(MidiSongPositionPointerMessage),
    SongSelect(MidiSongSelectMessage),
    TuneRequest(MidiTuneRequestMessage),
    EndSystemExclusive(MidiSystemExclusiveMessage),
    TimingClock(MidiTimingClockMessage),
    Start(MidiStartMessage),
    Continue(MidiContinueMessage),
    Stop(MidiStopMessage),
    ActiveSensing(MidiActiveSensingMessage),
    SystemReset(MidiSystemResetMessage),
}

impl WinMidi {
    pub fn send(&self, out: &MidiOutPort) -> Result<()> {
        match self {
            WinMidi::Unspecified(_) => Ok(()),
            WinMidi::NoteOff(msg) => out.SendMessage(msg),
            WinMidi::NoteOn(msg) => out.SendMessage(msg),
            WinMidi::PolyphonicKeyPressure(msg) => out.SendMessage(msg),
            WinMidi::ControlChange(msg) => out.SendMessage(msg),
            WinMidi::ProgramChange(msg) => out.SendMessage(msg),
            WinMidi::ChannelPressure(msg) => out.SendMessage(msg),
            WinMidi::PitchBendChange(msg) => out.SendMessage(msg),
            WinMidi::SystemExclusive(msg) => out.SendMessage(msg),
            WinMidi::TimeCode(msg) => out.SendMessage(msg),
            WinMidi::SongPositionPointer(msg) => out.SendMessage(msg),
            WinMidi::SongSelect(msg) => out.SendMessage(msg),
            WinMidi::TuneRequest(msg) => out.SendMessage(msg),
            WinMidi::EndSystemExclusive(msg) => out.SendMessage(msg),
            WinMidi::TimingClock(msg) => out.SendMessage(msg),
            WinMidi::Start(msg) => out.SendMessage(msg),
            WinMidi::Continue(msg) => out.SendMessage(msg),
            WinMidi::Stop(msg) => out.SendMessage(msg),
            WinMidi::ActiveSensing(msg) => out.SendMessage(msg),
            WinMidi::SystemReset(msg) => out.SendMessage(msg),
        }
    }
}

pub fn concrete_message(imsg: &IMidiMessage) -> WinMidi {
    match imsg.Type().unwrap_or_default() {
        MidiMessageType::None => WinMidi::Unspecified(0),
        MidiMessageType::NoteOff => WinMidi::NoteOff(imsg.cast().unwrap()),
        MidiMessageType::NoteOn => WinMidi::NoteOn(imsg.cast().unwrap()),
        MidiMessageType::PolyphonicKeyPressure => {
            WinMidi::PolyphonicKeyPressure(imsg.cast().unwrap())
        }
        MidiMessageType::ControlChange => WinMidi::ControlChange(imsg.cast().unwrap()),
        MidiMessageType::ProgramChange => WinMidi::ProgramChange(imsg.cast().unwrap()),
        MidiMessageType::ChannelPressure => WinMidi::ChannelPressure(imsg.cast().unwrap()),
        MidiMessageType::PitchBendChange => WinMidi::PitchBendChange(imsg.cast().unwrap()),
        MidiMessageType::SystemExclusive => WinMidi::SystemExclusive(imsg.cast().unwrap()),
        MidiMessageType::MidiTimeCode => WinMidi::TimeCode(imsg.cast().unwrap()),
        MidiMessageType::SongPositionPointer => WinMidi::SongPositionPointer(imsg.cast().unwrap()),
        MidiMessageType::SongSelect => WinMidi::SongSelect(imsg.cast().unwrap()),
        MidiMessageType::TuneRequest => WinMidi::TuneRequest(imsg.cast().unwrap()),
        MidiMessageType::EndSystemExclusive => WinMidi::EndSystemExclusive(imsg.cast().unwrap()),
        MidiMessageType::TimingClock => WinMidi::TimingClock(imsg.cast().unwrap()),
        MidiMessageType::Start => WinMidi::Start(imsg.cast().unwrap()),
        MidiMessageType::Continue => WinMidi::Continue(imsg.cast().unwrap()),
        MidiMessageType::Stop => WinMidi::Stop(imsg.cast().unwrap()),
        MidiMessageType::ActiveSensing => WinMidi::ActiveSensing(imsg.cast().unwrap()),
        MidiMessageType::SystemReset => WinMidi::SystemReset(imsg.cast().unwrap()),
        _ => WinMidi::Unspecified(0),
    }
}

#[allow(dead_code)]
pub fn dispatch_windows<T>(handler: &mut T, msg: &WinMidi) -> Result<()>
where
    T: WindowsMidiHandler,
{
    match msg {
        WinMidi::Unspecified(v) => handler.on_none(v),
        WinMidi::NoteOff(v) => handler.on_note_off(v),
        WinMidi::NoteOn(v) => handler.on_note_on(v),
        WinMidi::PolyphonicKeyPressure(v) => handler.on_polyphonic_key_pressure(v),
        WinMidi::ControlChange(v) => handler.on_control_change(v),
        WinMidi::ProgramChange(v) => handler.on_program_change(v),
        WinMidi::ChannelPressure(v) => handler.on_channel_pressure(v),
        WinMidi::PitchBendChange(v) => handler.on_pitch_bend_change(v),
        WinMidi::SystemExclusive(v) => handler.on_system_exclusive(v),
        WinMidi::TimeCode(v) => handler.on_midi_time_code(v),
        WinMidi::SongPositionPointer(v) => handler.on_song_position_pointer(v),
        WinMidi::SongSelect(v) => handler.on_song_select(v),
        WinMidi::TuneRequest(v) => handler.on_tune_request(v),
        WinMidi::EndSystemExclusive(v) => handler.on_end_system_exclusive(v),
        WinMidi::TimingClock(v) => handler.on_timing_clock(v),
        WinMidi::Start(v) => handler.on_start(v),
        WinMidi::Continue(v) => handler.on_continue(v),
        WinMidi::Stop(v) => handler.on_stop(v),
        WinMidi::ActiveSensing(v) => handler.on_active_sensing(v),
        WinMidi::SystemReset(v) => handler.on_system_reset(v),
        //_ => Err(windows::core::Error::new(HRESULT(0x80070057u32 as i32), h!("unknown MIDI message").clone()))
    }
}

pub trait WindowsMidiHandler {
    fn on_none(&mut self, _msg: &u8) -> Result<()> {
        Ok(())
    }
    fn on_note_off(&mut self, _msg: &MidiNoteOffMessage) -> Result<()> {
        Ok(())
    }
    fn on_note_on(&mut self, _msg: &MidiNoteOnMessage) -> Result<()> {
        Ok(())
    }
    fn on_polyphonic_key_pressure(
        &mut self,
        _msg: &MidiPolyphonicKeyPressureMessage,
    ) -> Result<()> {
        Ok(())
    }
    fn on_control_change(&mut self, _msg: &MidiControlChangeMessage) -> Result<()> {
        Ok(())
    }
    fn on_program_change(&mut self, _msg: &MidiProgramChangeMessage) -> Result<()> {
        Ok(())
    }
    fn on_channel_pressure(&mut self, _msg: &MidiChannelPressureMessage) -> Result<()> {
        Ok(())
    }
    fn on_pitch_bend_change(&mut self, _msg: &MidiPitchBendChangeMessage) -> Result<()> {
        Ok(())
    }
    fn on_system_exclusive(&mut self, _msg: &MidiSystemExclusiveMessage) -> Result<()> {
        Ok(())
    }
    fn on_midi_time_code(&mut self, _msg: &MidiTimeCodeMessage) -> Result<()> {
        Ok(())
    }
    fn on_song_position_pointer(&mut self, _msg: &MidiSongPositionPointerMessage) -> Result<()> {
        Ok(())
    }
    fn on_song_select(&mut self, _msg: &MidiSongSelectMessage) -> Result<()> {
        Ok(())
    }
    fn on_tune_request(&mut self, _msg: &MidiTuneRequestMessage) -> Result<()> {
        Ok(())
    }
    fn on_end_system_exclusive(&mut self, _msg: &MidiSystemExclusiveMessage) -> Result<()> {
        Ok(())
    }
    fn on_timing_clock(&mut self, _msg: &MidiTimingClockMessage) -> Result<()> {
        Ok(())
    }
    fn on_start(&mut self, _msg: &MidiStartMessage) -> Result<()> {
        Ok(())
    }
    fn on_continue(&mut self, _msg: &MidiContinueMessage) -> Result<()> {
        Ok(())
    }
    fn on_stop(&mut self, _msg: &MidiStopMessage) -> Result<()> {
        Ok(())
    }
    fn on_active_sensing(&mut self, _msg: &MidiActiveSensingMessage) -> Result<()> {
        Ok(())
    }
    fn on_system_reset(&mut self, _msg: &MidiSystemResetMessage) -> Result<()> {
        Ok(())
    }
}

pub trait MidiHandler {
    fn on_note_off(&mut self, _ticks: i64, _channel: u8, _note: u8, _velocity: u8) -> Result<()> {
        Ok(())
    }
    fn on_note_on(&mut self, _ticks: i64, _channel: u8, _note: u8, _velocity: u8) -> Result<()> {
        Ok(())
    }
    fn on_polyphonic_key_pressure(
        &mut self,
        _ticks: i64,
        _channel: u8,
        _note: u8,
        _pressure: u8,
    ) -> Result<()> {
        Ok(())
    }
    fn on_control_change(&mut self, _ticks: i64, _channel: u8, _cc: u8, _value: u8) -> Result<()> {
        Ok(())
    }
    fn on_program_change(&mut self, _ticks: i64, _channel: u8, _program: u8) -> Result<()> {
        Ok(())
    }
    fn on_channel_pressure(&mut self, _ticks: i64, _channel: u8, _pressure: u8) -> Result<()> {
        Ok(())
    }
    fn on_pitch_bend_change(&mut self, _ticks: i64, _channel: u8, _bend: u16) -> Result<()> {
        Ok(())
    }
    fn on_system_exclusive(&mut self, _ticks: i64, _data: Vec<u8>) -> Result<()> {
        Ok(())
    }
    fn on_midi_time_code(&mut self, _ticks: i64, _frame: u8, _values: u8) -> Result<()> {
        Ok(())
    }
    fn on_song_position_pointer(&mut self, _ticks: i64, _beats: u16) -> Result<()> {
        Ok(())
    }
    fn on_song_select(&mut self, _ticks: i64, _song: u8) -> Result<()> {
        Ok(())
    }
    fn on_tune_request(&mut self, _ticks: i64) -> Result<()> {
        Ok(())
    }
    fn on_end_system_exclusive(&mut self, _ticks: i64, _data: Vec<u8>) -> Result<()> {
        Ok(())
    }
    fn on_timing_clock(&mut self, _ticks: i64) -> Result<()> {
        Ok(())
    }
    fn on_start(&mut self, _ticks: i64) -> Result<()> {
        Ok(())
    }
    fn on_continue(&mut self, _ticks: i64) -> Result<()> {
        Ok(())
    }
    fn on_stop(&mut self, _ticks: i64) -> Result<()> {
        Ok(())
    }
    fn on_active_sensing(&mut self, _ticks: i64) -> Result<()> {
        Ok(())
    }
    fn on_system_reset(&mut self, _ticks: i64) -> Result<()> {
        Ok(())
    }
}

pub fn dispatch_midi<T>(handler: &mut T, msg: &WinMidi) -> Result<()>
where
    T: MidiHandler,
{
    match msg {
        WinMidi::Unspecified(_) => Ok(()), // ignored
        WinMidi::NoteOff(v) => handler.on_note_off(
            v.Timestamp()?.Duration,
            v.Channel()?,
            v.Note()?,
            v.Velocity()?,
        ),
        WinMidi::NoteOn(v) => handler.on_note_on(
            v.Timestamp()?.Duration,
            v.Channel()?,
            v.Note()?,
            v.Velocity()?,
        ),
        WinMidi::PolyphonicKeyPressure(v) => handler.on_polyphonic_key_pressure(
            v.Timestamp()?.Duration,
            v.Channel()?,
            v.Note()?,
            v.Pressure()?,
        ),
        WinMidi::ControlChange(v) => handler.on_control_change(
            v.Timestamp()?.Duration,
            v.Channel()?,
            v.Controller()?,
            v.ControlValue()?,
        ),
        WinMidi::ProgramChange(v) => {
            handler.on_program_change(v.Timestamp()?.Duration, v.Channel()?, v.Program()?)
        }
        WinMidi::ChannelPressure(v) => {
            handler.on_channel_pressure(v.Timestamp()?.Duration, v.Channel()?, v.Pressure()?)
        }
        WinMidi::PitchBendChange(v) => {
            handler.on_pitch_bend_change(v.Timestamp()?.Duration, v.Channel()?, v.Bend()?)
        }
        WinMidi::SystemExclusive(msg) => {
            let data = msg.RawData()?;
            let reader = windows::Storage::Streams::DataReader::FromBuffer(&data).unwrap();
            let mut bytes = vec![0u8; data.Length().unwrap() as usize];
            reader.ReadBytes(bytes.as_mut_slice()).unwrap();
            handler.on_system_exclusive(msg.Timestamp()?.Duration, bytes)
        }
        WinMidi::TimeCode(v) => {
            handler.on_midi_time_code(v.Timestamp()?.Duration, v.FrameType()?, v.Values()?)
        }
        WinMidi::SongPositionPointer(v) => {
            handler.on_song_position_pointer(v.Timestamp()?.Duration, v.Beats()?)
        }
        WinMidi::SongSelect(v) => handler.on_song_select(v.Timestamp()?.Duration, v.Song()?),
        WinMidi::TuneRequest(v) => handler.on_tune_request(v.Timestamp()?.Duration),
        WinMidi::EndSystemExclusive(msg) => {
            let data = msg.RawData()?;
            let reader = windows::Storage::Streams::DataReader::FromBuffer(&data).unwrap();
            let mut bytes = vec![0u8; data.Length().unwrap() as usize];
            reader.ReadBytes(bytes.as_mut_slice()).unwrap();
            handler.on_system_exclusive(msg.Timestamp()?.Duration, bytes)
        }
        WinMidi::TimingClock(v) => handler.on_timing_clock(v.Timestamp()?.Duration),
        WinMidi::Start(v) => handler.on_start(v.Timestamp()?.Duration),
        WinMidi::Continue(v) => handler.on_continue(v.Timestamp()?.Duration),
        WinMidi::Stop(v) => handler.on_stop(v.Timestamp()?.Duration),
        WinMidi::ActiveSensing(v) => handler.on_active_sensing(v.Timestamp()?.Duration),
        WinMidi::SystemReset(v) => handler.on_system_reset(v.Timestamp()?.Duration),
        //_ => Err(windows::core::Error::new(HRESULT(0x80070057u32 as i32), h!("unknown MIDI message").clone()))
    }
}
