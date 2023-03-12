use crate::midi_traits::*;
use windows::core::*;

#[derive(Default, Clone)]
pub struct MidiDeviceInfo {
    pub name: HSTRING,
    pub id: HSTRING,
}

pub const STATUS_NOTE_OFF: u8 = 0x80;
pub const STATUS_NOTE_ON: u8 = 0x90;
pub const STATUS_POLY_KEY_PRESSURE: u8 = 0xA0;
pub const STATUS_CC: u8 = 0xB0;
pub const STATUS_PROGRAM_CHANGE: u8 = 0xC0;
pub const STATUS_CHANNEL_PRESSURE: u8 = 0xD0;
pub const STATUS_PITCH_BEND: u8 = 0xE0;
pub const STATUS_SYSTEM: u8 = 0xF0;

pub const CHANNEL15: u8 = 14; // 1-based naming
pub const CHANNEL16: u8 = 15; // 1-based naming

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
#[rustfmt::skip]
pub enum Note { C, Cs, D, Eb, E, F, Fs, G, Ab, A, Bb, B }
impl From<u8> for Note {
    fn from(value: u8) -> Self {
        unsafe { ::std::mem::transmute(value % 12) }
    }
}
impl Note {
    const NOTE_NAME: [&str; 12] = [
        "C",
        "C\u{266F}", // C#
        "D",
        "E\u{266D}", // Eb
        "E",
        "F",
        "F\u{266F}", // F#
        "G",
        "A\u{266D}", // Ab
        "A",
        "B\u{266D}", // Bb
        "B",
    ];
}
impl Named for Note {
    fn name(&self) -> &'static str {
        Self::NOTE_NAME[*self as usize]
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct MidiNote(u8);
impl MidiNote {
    pub fn raw(&self) -> u8 {
        self.0
    }
    pub fn new(n: u8) -> Self {
        Self(n)
    }
    pub fn make(note: Note, octave: u8) -> Self {
        MidiNote(octave * 12 + (note as u8))
    }
    pub fn octave(&self) -> i32 {
        self.0 as i32 / 12
    }
    pub fn note_name(&self) -> &'static str {
        self.note().name()
    }
    pub fn note(&self) -> Note {
        self.0.into()
    }
    pub fn set_octave(&mut self, octave: u8) {
        let note = self.note();
        self.0 = octave * 12 + (note as u8);
    }
}
impl Described for MidiNote {
    fn describe(&self) -> String {
        let mut buf = self.note_name().to_string();
        buf += &self.octave().to_string();
        buf
    }
}

// $consider: separate channel-specific purposes
pub fn continuum_cc_name(cc: u8) -> &'static str {
    match cc {
        0 => "Bank Select MSB",
        8 => "Octave shift",
        9 => "Mono switch",
        10 => "Fine tune",
        11 => "Expression?",
        12 => "i",
        13 => "ii",
        14 => "iii",
        15 => "iv",
        16 => "v",
        17 => "vi",
        18 => "Post Master level",
        19 => "Audio input level",
        20 => "R-1",
        21 => "R-2",
        22 => "R-3",
        23 => "R-4",
        24 => "R-Mix",
        25 => "Round rate",
        26 => "Pre Master level",
        27 => "Output attenuation",
        28 => "Round initial",
        29 => "Pedal Jack 1",
        30 => "Pedal Jack 2",
        31 => "Preset advance",
        32 => "Bank LSB",
        33 => "Action/AES",
        34 => "Algorithm",
        35 => "Program #",
        36 => "Routing",
        37 => "Pedal type",
        38 => "Data LSB (logs, custom tuning, ...)",
        39 => "Polyphony",
        40 => "Pitch bend range (semitones)",
        41 => "Y cc",
        42 => "Z cc",
        43 => "Note handliing",
        44 => "Middle C position",
        45 => "Split point (note number)",
        46 => "Mono function",
        47 => "Recirculator column",
        48 => "Mono Interval",
        49 => "Note Priority",

        51 => "Tuning: 0 default, 1-50 n-tone equal, 60-71 just",
        52 => "Pedal 1 cc",
        53 => "Pedal 2 cc",
        54 => "Pedal octave shift amount",
        55 => "Setting Preservation",
        56 => "Data Stream <type> (127 = end)",

        59 => "Dim menu",

        60 => "Touch center",
        61 => "Reverse pitch",
        62 => "Recirculator type",
        63 => "CVC configuration",
        64 => "Sustain",
        65 => "Rounding override",
        66 => "Sos 1",
        67 => "Headphone level",
        68 => "Line level",
        69 => "Sos 2",
        70 => "Actuation",
        71 => "Total traditional polyphony",
        72 => "Total DSP polyphony",
        73 => "Total CVC polyphony",

        75 => "Stress test",
        76 => "Pedal 1 min",
        77 => "Pedal 1 max",
        78 => "Pedal 2 min",
        79 => "Pedal 2 max",
        80 => "Q Bias (obsolete)",
        81 => "(old) Compression rate",
        82 => "(old) Compression time",
        83 => "Tilt EQ",
        84 => "EQ Freq",
        85 => "EQ Mix",
        90 => "Compressor Threshhold",
        91 => "Compressor Attack",
        92 => "Compressor Ratio",
        93 => "Compressor Mix",

        98 => "MPE+ lo NRPN select",
        99 => "MPE+ hi NRPN select",
        100 => "MPE lo RPN select",
        101 => "MPE hi RPN select",
        102 => "Firmware version hi",
        103 => "Firmware version lo",
        104 => "Hardware/CVC hi",
        105 => "CVC mid",
        106 => "CVC lo",
        107 => "SNBN a",
        109 => "Editor message",
        110 => "HE<>Device info",
        111 => "Device status",

        113 => "SNBN b",
        114 => "DSP %",
        115 => "Log dump",
        116 => "Haken editor presence",
        117 => "Loopback detect",
        118 => "Editor reply",
        119 => "archive no-op",
        120 => "All sound off",
        122 => "CRC 0 7'",
        123 => "CRC 1 7'",
        124 => "CRC 2 7'",
        125 => "CRC 3 7'",
        126 => "CRC 5 4'",
        127 => "MPE Polyphony",
        _ => "(available)",
    }
}

// standard MIDI
pub fn standard_cc_name(cc: u8) -> &'static str {
    match cc {
        0 => "Bank Select MSB",
        1 => "Mod Wheel MSB",
        2 => "Breath MSB",
        3 => "(undefined) MSB",
        4 => "Pedal MSB",
        5 => "Portamento Time MSB",
        6 => "Data Entry MSB",
        7 => "Volume MSB",
        8 => "Balance MSB",
        9 => "(undefined) MSB",
        10 => "Pan MSB",
        11 => "Expression MSB",
        12 => "Effect 1 MSB",
        13 => "Effect 2 MSB",
        14..=15 => "(undefined) MSB",
        16..=19 => "General MSB",
        20..=31 => "(undefined) MSB",
        32 => "cc00 Bank Select LSB",
        33 => "cc01 Mod wheel LSB",
        34 => "cc02 Breath LSB",
        35 => "cc03 (undefined) LSB",
        36 => "cc04 Pedal LSB",
        37 => "cc05 Portamento Time LSB",
        38 => "cc06 Data entry LSB",
        39 => "cc07 Volume LSB",
        40 => "cc08 Balance LSB",
        41 => "cc09 (undefined) LSB",
        42 => "cc10 Pan LSB",
        43 => "cc11 Expression LSB",
        44 => "cc12 Effect 1 LSB",
        45 => "cc13 Effect 2 LSB",
        46 => "cc14 (undefined) LSB",
        47 => "cc15 (undefined) LSB",
        48 => "cc16 General LSB",
        49 => "cc17 General LSB",
        50 => "cc18 General LSB",
        51 => "cc19 General LSB",
        52 => "cc20 (undefined) LSB",
        53 => "cc21 (undefined) LSB",
        54 => "cc22 (undefined) LSB",
        55 => "cc23 (undefined) LSB",
        56 => "cc24 (undefined) LSB",
        57 => "cc25 (undefined) LSB",
        58 => "cc26 (undefined) LSB",
        59 => "cc27 (undefined) LSB",
        60 => "cc28 (undefined) LSB",
        61 => "cc29 (undefined) LSB",
        62 => "cc30 (undefined) LSB",
        63 => "cc31 (undefined) LSB",
        64 => "Damper pedal (sustain)",
        65 => "Portamento on/off",
        66 => "Sostenuto on/off",
        67 => "Soft Pedal on/off",
        68 => "Legato footswitch",
        69 => "Hold 2",
        70 => "Sound 1 (sound variation)",
        71 => "Sound 2 (timbre/harmonic intensity/resonance)",
        72 => "Sound 3 (release time)",
        73 => "Sound 4 (attack time)",
        74 => "Sound 5 (brightness)",
        75 => "Sound 6",
        76 => "Sound 7",
        77 => "Sound 8",
        78 => "Sound 9",
        79 => "Sound 10",
        80 => "Generic on/off (decay)",
        81 => "Generic on/off (HPF freq)",
        82 => "Generic on/off",
        83 => "Generic on/off",
        84 => "Portamento",
        85 => "(undefined)",
        86 => "(undefined)",
        87 => "Multipurpose LSB",
        88 => "Hi-res velocity prefix",
        89 => "(undefined)",
        90 => "(undefined)",
        91 => "Effect 1 depth (reverb)",
        92 => "Effect 2 depth (tremolo)",
        93 => "Effect 3 depth (chorus)",
        94 => "Effect 4 depth (detune)",
        95 => "Effect 5 depth (phaser)",
        96 => "Data increment (+1)",
        97 => "Data decrement (-1)",
        98 => "NRPN LSB",
        99 => "NRPN MSB",
        100 => "RPN LSB",
        101 => "RPN MSB",
        102..=119 => "(undefined)",
        120 => "All sound off",
        121 => "Reset all",
        122 => "Local on/off",
        123 => "All notes off",
        124 => "Omni mode off",
        125 => "Omni mode on",
        126 => "Mono (#channels, 0=all)",
        127 => "Poly mode",
        _ => "(invalid cc)",
    }
}

pub fn is_midi_header(data: &[u8]) -> bool {
    if data.len() < 4 {
        false
    } else {
        data[0] == b'M' && data[1] == b'T' && data[2] == b'h' && data[3] == b'd'
    }
}

pub fn is_midi_track_header(data: &[u8]) -> bool {
    if data.len() < 4 {
        false
    } else {
        data[0] == b'M' && data[1] == b'T' && data[2] == b'r' && data[3] == b'k'
    }
}

pub struct VariableLengthValue {
    value: u32,
    pending: bool,
}
impl Default for VariableLengthValue {
    fn default() -> Self {
        Self {
            value: 0,
            pending: true,
        }
    }
}
impl VariableLengthValue {
    #[allow(dead_code)]
    pub const MAX_VALUE: u32 = 0x0FFFFFFFu32;

    #[allow(dead_code)]
    pub fn value_in_range(value: u32) -> Result<u32> {
        if value > Self::MAX_VALUE {
            return Err(Error::new(
                windows::Win32::Foundation::DISP_E_OVERFLOW, // hack: reuse IDispatch error code
                h!("value out of range for MIDI variable-length value").clone(),
            ));
        }
        Ok(value)
    }

    /// Generate a variable-length-encoded Vec<u8> representing the value
    ///
    /// The value must be in the range 0 - 0x0FFFFFFF
    #[allow(dead_code)]
    pub fn encode(value: u32) -> Result<Vec<u8>> {
        _ = Self::value_in_range(value)?;
        let mut bytes = value.to_be_bytes();
        debug_assert!(bytes.len() == 4);
        Self::encode_slice(value, &mut bytes)?;
        let mut result = Vec::default();
        for byte in bytes {
            result.push(byte);
            if !Self::is_lead_byte(byte) {
                break;
            }
        }
        Ok(result)
    }

    /// Write a variable-length-encoded value to a slice of 4 bytes.
    ///
    /// Returns the number of bytes written.
    #[allow(dead_code)]
    pub fn encode_slice(value: u32, slice: &mut [u8; 4]) -> Result<usize> {
        let value = Self::value_in_range(value)?;
        let mut acc = value & 0x0000007F;
        let mut value = value >> 7;
        while value > 0 {
            acc = (acc << 8) | 0x00000080 | (value & 0x0000007F);
            value >>= 7;
        }
        let mut index: usize = 0;
        loop {
            let byte: u8 = (acc & 0x000000FF) as u8;
            slice[index] = byte;
            index += 1;
            if 0 == byte & 0x80 {
                break;
            }
            acc >>= 8;
        }
        // zero-pad slice
        //slice.iter_mut().take(4).skip(index).for_each(|byte| {*byte = 0;});
        Ok(index)
    }

    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_pending(&self) -> bool {
        self.pending
    }
    pub fn start(&mut self) {
        self.value = 0;
        self.pending = true;
    }

    pub fn finish(&mut self) -> Result<u32> {
        if self.pending {
            return Err(Error::new(
                HRESULT(0x8000000Au32 as i32), //E_PENDING
                h!("need more data for MIDI variable-length value").clone(),
            ));
        }
        let result = self.value;
        self.start();
        Ok(result)
    }

    /// Add bytes from encoded data until the first non-leading byte.
    #[inline]
    #[allow(dead_code)]
    pub fn is_lead_byte(byte: u8) -> bool {
        0 != byte & 0x80
    }

    /// Add a byte to the value.
    ///
    /// Returns true when more bytes are expected.
    /// Returns false when the data is complete and you can
    /// safely call finish to get the completed u32.
    pub fn add_byte(&mut self, byte: u8) -> bool {
        assert!(self.pending);
        self.pending = 0 != byte & 0x80;
        self.value <<= 7;
        self.value |= byte as u32 & 0x0000007F;
        self.pending
    }
}

#[derive(Default)]
pub struct BinBuild {
    encoded: bool,
    binary: Vec<u8>,
    decoder: VariableLengthValue,
}

impl BinBuild {
    pub fn new() -> Self {
        BinBuild::default()
    }
    fn flush_decoder(&mut self) {
        if self.decoder.is_pending() {
            while self.decoder.add_byte(0) {} //zero pad
            let value = self.decoder.finish().unwrap();
            for decoded_byte in value.to_be_bytes() {
                self.binary.push(decoded_byte);
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_encoded(&mut self, encoded: bool) -> Result<()> {
        if self.encoded != encoded {
            if self.encoded {
                self.flush_decoder()
            }
            self.encoded = encoded;
        }
        Ok(())
    }

    pub fn start(&mut self, encoded: bool) {
        self.encoded = encoded;
        self.binary.clear();
        self.decoder.start();
    }

    fn add_raw(&mut self, byte: u8) {
        debug_assert!(self.encoded);
        self.binary.push(byte);
    }

    fn add_encoded(&mut self, byte: u8) {
        debug_assert!(!self.encoded);
        if !self.decoder.add_byte(byte) {
            let value = self.decoder.finish().unwrap();
            for decoded_byte in value.to_be_bytes() {
                self.binary.push(decoded_byte);
            }
        }
    }

    pub fn add(&mut self, byte: u8) {
        if self.encoded {
            self.add_raw(byte);
        } else {
            self.add_encoded(byte);
        }
    }

    pub fn flush(&mut self) -> Vec<u8> {
        if !self.encoded {
            self.flush_decoder();
        }
        let result = self.binary.clone();
        self.start(true);
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::midi::VariableLengthValue;

    fn round_trip(value: u32) {
        let mut decoder = VariableLengthValue::new();

        let bytes = VariableLengthValue::encode(value).unwrap();
        assert!(bytes.len() <= 4);

        decoder.start();
        let mut pending = true;
        for byte in bytes.iter() {
            pending = decoder.add_byte(*byte);
        }
        assert!(!pending);
        let result = decoder.finish().unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn variable_length_value_works() {
        let value = 0u32;
        let bytes = VariableLengthValue::encode(value).unwrap();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes.first(), Some(&0u8));
        round_trip(value);

        let value = 0x0000007F;
        let bytes = VariableLengthValue::encode(value).unwrap();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0x7Fu8);
        round_trip(value);

        let value = 0x00000080;
        let bytes = VariableLengthValue::encode(value).unwrap();
        assert_eq!(bytes.len(), 2);
        assert_eq!(bytes[0], 0x81u8);
        assert_eq!(bytes[1], 0x00u8);
        round_trip(value);

        let value = 0x00004000;
        let bytes = VariableLengthValue::encode(0x00004000).unwrap();
        assert_eq!(bytes.len(), 3);
        round_trip(value);

        let value = VariableLengthValue::MAX_VALUE;
        let bytes = VariableLengthValue::encode(value).unwrap();
        assert_eq!(bytes.len(), 4);
        assert_eq!(bytes[0], 0xFFu8);
        assert_eq!(bytes[1], 0xFFu8);
        assert_eq!(bytes[2], 0xFFu8);
        assert_eq!(bytes[3], 0x7Fu8);
        round_trip(value);

        round_trip(255);
        round_trip(256);
        round_trip(32767);
        round_trip(32768);
        round_trip(0x00123456);
        round_trip(VariableLengthValue::MAX_VALUE - 13);
    }
}
