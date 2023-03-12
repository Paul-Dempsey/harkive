use crate::midi_traits::*;
use crate::util::short_hash;
use std::collections::HashMap;

#[repr(u8)]
#[derive(Clone, Copy, Default, PartialEq)]
pub enum NofN {
    #[default]
    Single,
    Double,
    Triple,
}

pub fn is_empty_preset_name(name: &str) -> bool {
    match name.len() {
        0..=4 => false,
        5 => name == "Empty",
        6.. => &(name[0..6]) == "Empty.",
        _ => false,
    }
}

pub fn make_preset_filename(preset: &str, data: &[u8]) -> String {
    let anon = is_empty_preset_name(preset);
    if anon {
        println!("Renaming Empty or un-named preset");
    }
    (if anon {
        format!("anon-{}", short_hash(data))
    } else {
        preset.to_string()
    } + ".mid")
}

#[derive(Clone)]
pub struct ContinuumPreset {
    pub name: String,
    pub text: String,
    pub category: String,
    pub bank_hi: u8, // MIDI cc0
    pub bank_lo: u8, // MIDI cc32
    pub number: u8,  // MIDI ProgramChange
    pub nofn: NofN,
}

impl ContinuumPreset {
    pub fn print(&self) {
        let preset_index = ((self.bank_lo as u16) << 7) | self.number as u16;
        println!(
            "Preset: [{}-{}-{} {} {}] \"{}\" {}{}",
            self.bank_hi,
            self.bank_lo,
            self.number,
            match self.bank_hi {
                0 => "User preset",
                126 => "Current editing slot",
                127 => "System preset",
                _ => "cat-code",
            },
            preset_index,
            self.name,
            match self.nofn {
                NofN::Single => "",
                NofN::Double => "first of 2 ",
                NofN::Triple => "first of 3 ",
            },
            self.text
        );
    }
    pub fn print_friendly_categories(&self, cats: &HCCategoryCode) {
        if let Some(friendly) = cats.decode(&self.text) {
            println!("  {friendly}");
        }
    }
}

// Preset metadata derived from "C:\HakenEditor\HE\Data\components\CatsColl.txt"
#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum PresetGroup {
    Unknown,
    Category,
    Type,
    Character,
    Matrix,
    Setting,
}
impl Named for PresetGroup {
    fn name(&self) -> &'static str {
        match self {
            PresetGroup::Unknown => "x",
            PresetGroup::Category => "category",
            PresetGroup::Type => "type",
            PresetGroup::Character => "character",
            PresetGroup::Matrix => "matrix",
            PresetGroup::Setting => "setting",
        }
    }
}

pub struct PresetMeta {
    pub code: &'static str,
    pub group: PresetGroup,
    pub index: u8,
    pub name: &'static str,
}

pub fn category_list(text: &str) -> Vec<String> {
    let mut result = Vec::<String>::new();
    for section in text.split([' ', '\n']) {
        if let Some(start) = section.find("C=") {
            for code in section[start + 2..].split('_') {
                result.push(code.to_string());
            }
        }
    }
    result
}

pub struct HCCategoryCode {
    data: HashMap<String, PresetMeta>,
}

impl Default for HCCategoryCode {
    fn default() -> Self {
        Self::new()
    }
}

impl HCCategoryCode {
    pub fn decode(&self, text: &str) -> Option<String> {
        if text.len() < 4 {
            return None;
        };

        let mut result = String::new();
        result.push('{');

        let mut group = PresetGroup::Unknown;
        let mut first = true;
        for code in category_list(text) {
            if let Some(meta) = self.data.get(&code.to_string()) {
                if group != meta.group {
                    match group {
                        PresetGroup::Category => {
                            result += ", ";
                        }
                        PresetGroup::Unknown => {}
                        _ => {
                            result += "], ";
                        }
                    };
                    first = true;
                    result += meta.group.name();
                    if meta.group == PresetGroup::Category {
                        result.push(':')
                    } else {
                        result += ":[";
                    }
                    group = meta.group;
                }
                if first {
                    first = false;
                } else {
                    result += ", ";
                }
                result.push('"');
                result += meta.name;
                result.push('"');
            }
        }

        result += "]}";
        Some(result)
    }

    fn add(&mut self, item: PresetMeta) {
        self.data.insert(item.code.to_string(), item);
    }

    pub fn get_by_category_name(&self, name: &str) -> Option<&PresetMeta> {
        self.data
            .values()
            .find(|v| v.group == PresetGroup::Category && v.name == name)
    }
    pub fn get_by_category_code(&self, code: &str) -> Option<&PresetMeta> {
        self.data
            .values()
            .find(|v| v.group == PresetGroup::Category && v.code == code)
    }

    pub fn new() -> Self {
        let mut result = HCCategoryCode {
            data: HashMap::new(),
        };
        result.add(PresetMeta {
            code: "ST",
            group: PresetGroup::Category,
            index: 1,
            name: "Strings",
        });
        result.add(PresetMeta {
            code: "WI",
            group: PresetGroup::Category,
            index: 2,
            name: "Winds",
        });
        result.add(PresetMeta {
            code: "VO",
            group: PresetGroup::Category,
            index: 3,
            name: "Vocal",
        });
        result.add(PresetMeta {
            code: "KY",
            group: PresetGroup::Category,
            index: 4,
            name: "Keyboard",
        });
        result.add(PresetMeta {
            code: "CL",
            group: PresetGroup::Category,
            index: 5,
            name: "Classic",
        });
        result.add(PresetMeta {
            code: "OT",
            group: PresetGroup::Category,
            index: 6,
            name: "Other",
        });
        result.add(PresetMeta {
            code: "PE",
            group: PresetGroup::Category,
            index: 7,
            name: "Percussion",
        });
        result.add(PresetMeta {
            code: "PT",
            group: PresetGroup::Category,
            index: 8,
            name: "Tuned Perc",
        });
        result.add(PresetMeta {
            code: "PR",
            group: PresetGroup::Category,
            index: 9,
            name: "Processor",
        });
        result.add(PresetMeta {
            code: "DO",
            group: PresetGroup::Category,
            index: 10,
            name: "Drone",
        });
        result.add(PresetMeta {
            code: "MD",
            group: PresetGroup::Category,
            index: 11,
            name: "Midi",
        });
        result.add(PresetMeta {
            code: "CV",
            group: PresetGroup::Category,
            index: 12,
            name: "Control Voltage",
        });
        result.add(PresetMeta {
            code: "UT",
            group: PresetGroup::Category,
            index: 13,
            name: "Utility",
        });
        result.add(PresetMeta {
            code: "AT",
            group: PresetGroup::Type,
            index: 0,
            name: "Atonal",
        });
        result.add(PresetMeta {
            code: "BA",
            group: PresetGroup::Type,
            index: 1,
            name: "Bass",
        });
        result.add(PresetMeta {
            code: "BO",
            group: PresetGroup::Type,
            index: 2,
            name: "Bowed",
        });
        result.add(PresetMeta {
            code: "BR",
            group: PresetGroup::Type,
            index: 3,
            name: "Brass",
        });
        result.add(PresetMeta {
            code: "DP",
            group: PresetGroup::Type,
            index: 4,
            name: "Demo Preset",
        });
        result.add(PresetMeta {
            code: "EP",
            group: PresetGroup::Type,
            index: 5,
            name: "Elec Piano",
        });
        result.add(PresetMeta {
            code: "FL",
            group: PresetGroup::Type,
            index: 6,
            name: "Flute",
        });
        result.add(PresetMeta {
            code: "LE",
            group: PresetGroup::Type,
            index: 7,
            name: "Lead",
        });
        result.add(PresetMeta {
            code: "OR",
            group: PresetGroup::Type,
            index: 8,
            name: "Organ",
        });
        result.add(PresetMeta {
            code: "PA",
            group: PresetGroup::Type,
            index: 9,
            name: "Pad",
        });
        result.add(PresetMeta {
            code: "PL",
            group: PresetGroup::Type,
            index: 10,
            name: "Plucked",
        });
        result.add(PresetMeta {
            code: "RD",
            group: PresetGroup::Type,
            index: 11,
            name: "Double Reed",
        });
        result.add(PresetMeta {
            code: "RS",
            group: PresetGroup::Type,
            index: 12,
            name: "Single Reed",
        });
        result.add(PresetMeta {
            code: "SU",
            group: PresetGroup::Type,
            index: 13,
            name: "Struck",
        });
        result.add(PresetMeta {
            code: "AC",
            group: PresetGroup::Character,
            index: 0,
            name: "Acoustic",
        });
        result.add(PresetMeta {
            code: "AG",
            group: PresetGroup::Character,
            index: 1,
            name: "Aggressive",
        });
        result.add(PresetMeta {
            code: "AI",
            group: PresetGroup::Character,
            index: 2,
            name: "Airy",
        });
        result.add(PresetMeta {
            code: "AN",
            group: PresetGroup::Character,
            index: 3,
            name: "Analog",
        });
        result.add(PresetMeta {
            code: "AR",
            group: PresetGroup::Character,
            index: 4,
            name: "Arpeggio",
        });
        result.add(PresetMeta {
            code: "BG",
            group: PresetGroup::Character,
            index: 5,
            name: "Big",
        });
        result.add(PresetMeta {
            code: "BI",
            group: PresetGroup::Character,
            index: 6,
            name: "Bright",
        });
        result.add(PresetMeta {
            code: "CH",
            group: PresetGroup::Character,
            index: 7,
            name: "Chords",
        });
        result.add(PresetMeta {
            code: "CN",
            group: PresetGroup::Character,
            index: 8,
            name: "Clean",
        });
        result.add(PresetMeta {
            code: "DA",
            group: PresetGroup::Character,
            index: 9,
            name: "Dark",
        });
        result.add(PresetMeta {
            code: "DI",
            group: PresetGroup::Character,
            index: 10,
            name: "Digital",
        });
        result.add(PresetMeta {
            code: "DT",
            group: PresetGroup::Character,
            index: 11,
            name: "Distorted",
        });
        result.add(PresetMeta {
            code: "DY",
            group: PresetGroup::Character,
            index: 12,
            name: "Dry",
        });
        result.add(PresetMeta {
            code: "EC",
            group: PresetGroup::Character,
            index: 13,
            name: "Echo",
        });
        result.add(PresetMeta {
            code: "EL",
            group: PresetGroup::Character,
            index: 14,
            name: "Electric",
        });
        result.add(PresetMeta {
            code: "EN",
            group: PresetGroup::Character,
            index: 15,
            name: "Ensemble",
        });
        result.add(PresetMeta {
            code: "EV",
            group: PresetGroup::Character,
            index: 16,
            name: "Evolving",
        });
        result.add(PresetMeta {
            code: "FM",
            group: PresetGroup::Character,
            index: 17,
            name: "FM",
        });
        result.add(PresetMeta {
            code: "HY",
            group: PresetGroup::Character,
            index: 18,
            name: "Hybrid",
        });
        result.add(PresetMeta {
            code: "IC",
            group: PresetGroup::Character,
            index: 19,
            name: "Icy",
        });
        result.add(PresetMeta {
            code: "IN",
            group: PresetGroup::Character,
            index: 20,
            name: "Intimate",
        });
        result.add(PresetMeta {
            code: "LF",
            group: PresetGroup::Character,
            index: 21,
            name: "Lo-fi",
        });
        result.add(PresetMeta {
            code: "LP",
            group: PresetGroup::Character,
            index: 22,
            name: "Looping",
        });
        result.add(PresetMeta {
            code: "LY",
            group: PresetGroup::Character,
            index: 23,
            name: "Layered",
        });
        result.add(PresetMeta {
            code: "MO",
            group: PresetGroup::Character,
            index: 24,
            name: "Morphing",
        });
        result.add(PresetMeta {
            code: "MT",
            group: PresetGroup::Character,
            index: 25,
            name: "Metallic",
        });
        result.add(PresetMeta {
            code: "NA",
            group: PresetGroup::Character,
            index: 26,
            name: "Nature",
        });
        result.add(PresetMeta {
            code: "NO",
            group: PresetGroup::Character,
            index: 27,
            name: "Noise",
        });
        result.add(PresetMeta {
            code: "RN",
            group: PresetGroup::Character,
            index: 28,
            name: "Random",
        });
        result.add(PresetMeta {
            code: "RV",
            group: PresetGroup::Character,
            index: 29,
            name: "Reverberant",
        });
        result.add(PresetMeta {
            code: "SD",
            group: PresetGroup::Character,
            index: 30,
            name: "Snd Design",
        });
        result.add(PresetMeta {
            code: "SE",
            group: PresetGroup::Character,
            index: 31,
            name: "Stereo",
        });
        result.add(PresetMeta {
            code: "SH",
            group: PresetGroup::Character,
            index: 32,
            name: "Shaking",
        });
        result.add(PresetMeta {
            code: "SI",
            group: PresetGroup::Character,
            index: 33,
            name: "Simple",
        });
        result.add(PresetMeta {
            code: "SO",
            group: PresetGroup::Character,
            index: 34,
            name: "Soft",
        });
        result.add(PresetMeta {
            code: "SR",
            group: PresetGroup::Character,
            index: 35,
            name: "Strumming",
        });
        result.add(PresetMeta {
            code: "SY",
            group: PresetGroup::Character,
            index: 36,
            name: "Synthetic",
        });
        result.add(PresetMeta {
            code: "WA",
            group: PresetGroup::Character,
            index: 37,
            name: "Warm",
        });
        result.add(PresetMeta {
            code: "WO",
            group: PresetGroup::Character,
            index: 38,
            name: "Woody",
        });
        result.add(PresetMeta {
            code: "AD",
            group: PresetGroup::Matrix,
            index: 0,
            name: "Additive",
        });
        result.add(PresetMeta {
            code: "BB",
            group: PresetGroup::Matrix,
            index: 1,
            name: "BiqBank",
        });
        result.add(PresetMeta {
            code: "BH",
            group: PresetGroup::Matrix,
            index: 2,
            name: "BiqGraph",
        });
        result.add(PresetMeta {
            code: "BM",
            group: PresetGroup::Matrix,
            index: 3,
            name: "BiqMouth",
        });
        result.add(PresetMeta {
            code: "CM",
            group: PresetGroup::Matrix,
            index: 4,
            name: "Cutoff Mod",
        });
        result.add(PresetMeta {
            code: "DF",
            group: PresetGroup::Matrix,
            index: 5,
            name: "Formula Delay",
        });
        result.add(PresetMeta {
            code: "DM",
            group: PresetGroup::Matrix,
            index: 6,
            name: "Micro Delay",
        });
        result.add(PresetMeta {
            code: "DS",
            group: PresetGroup::Matrix,
            index: 7,
            name: "Sum Delay",
        });
        result.add(PresetMeta {
            code: "DV",
            group: PresetGroup::Matrix,
            index: 8,
            name: "Voice Delay",
        });
        result.add(PresetMeta {
            code: "HM",
            group: PresetGroup::Matrix,
            index: 9,
            name: "HarMan",
        });
        result.add(PresetMeta {
            code: "KI",
            group: PresetGroup::Matrix,
            index: 10,
            name: "Kinetic",
        });
        result.add(PresetMeta {
            code: "MM",
            group: PresetGroup::Matrix,
            index: 11,
            name: "ModMan",
        });
        result.add(PresetMeta {
            code: "OJ",
            group: PresetGroup::Matrix,
            index: 12,
            name: "Osc Jenny",
        });
        result.add(PresetMeta {
            code: "OP",
            group: PresetGroup::Matrix,
            index: 13,
            name: "Osc Phase",
        });
        result.add(PresetMeta {
            code: "OS",
            group: PresetGroup::Matrix,
            index: 14,
            name: "Osc DSF",
        });
        result.add(PresetMeta {
            code: "SB",
            group: PresetGroup::Matrix,
            index: 15,
            name: "SineBank",
        });
        result.add(PresetMeta {
            code: "SS",
            group: PresetGroup::Matrix,
            index: 16,
            name: "SineSpray",
        });
        result.add(PresetMeta {
            code: "WB",
            group: PresetGroup::Matrix,
            index: 17,
            name: "WaveBank",
        });
        result.add(PresetMeta {
            code: "C1",
            group: PresetGroup::Setting,
            index: 0,
            name: "Channel 1",
        });
        result.add(PresetMeta {
            code: "EM",
            group: PresetGroup::Setting,
            index: 1,
            name: "Ext Midi Clk",
        });
        result.add(PresetMeta {
            code: "MI",
            group: PresetGroup::Setting,
            index: 2,
            name: "Mono Interval",
        });
        result.add(PresetMeta {
            code: "PO",
            group: PresetGroup::Setting,
            index: 3,
            name: "Portamento",
        });
        result.add(PresetMeta {
            code: "RO",
            group: PresetGroup::Setting,
            index: 4,
            name: "Rounding",
        });
        result.add(PresetMeta {
            code: "SP",
            group: PresetGroup::Setting,
            index: 5,
            name: "Split Voice",
        });
        result.add(PresetMeta {
            code: "SV",
            group: PresetGroup::Setting,
            index: 6,
            name: "Single Voice",
        });
        result.add(PresetMeta {
            code: "TA",
            group: PresetGroup::Setting,
            index: 7,
            name: "Touch Area",
        });
        result
    }
}

pub struct PresetBuilder {
    name: String,
    text: String,
    category: String,
    bank_hi: u8,
    bank_lo: u8,
    number: u8,
    nofn: NofN,
}
impl Default for PresetBuilder {
    fn default() -> Self {
        Self {
            name: String::new(),
            text: String::new(),
            category: String::new(),
            bank_hi: 0,
            bank_lo: 0,
            number: u8::MAX,
            nofn: NofN::default(),
        }
    }
}
impl PresetBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn name_add(&mut self, ch: char) {
        self.name.push(ch);
    }
    pub fn text_add(&mut self, ch: char) {
        self.text.push(ch);
    }
    pub fn category_add(&mut self, ch: char) {
        self.category.push(ch);
    }
    pub fn add_name_chars(&mut self, name: &str) {
        for ch in name.chars() {
            self.name_add(ch);
        }
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }
    pub fn set_category(&mut self, text: String) {
        self.category = text;
    }
    pub fn set_bank_hi(&mut self, hi: u8) {
        self.bank_hi = hi;
    }
    pub fn set_bank_lo(&mut self, lo: u8) {
        self.bank_lo = lo;
    }
    pub fn set_number(&mut self, num: u8) {
        self.number = num;
    }
    pub fn set_nofn(&mut self, nofn: NofN) {
        self.nofn = nofn;
    }
    pub fn start(&mut self) {
        self.name.clear();
        self.text.clear();
        self.category.clear();
        self.bank_hi = 0;
        self.bank_lo = 0;
        self.number = u8::MAX;
        self.nofn = NofN::default();
    }
    pub fn finish(&mut self) -> Option<ContinuumPreset> {
        if self.number == u8::MAX || self.name.is_empty() {
            self.start();
            return None;
        }
        let result = Some(ContinuumPreset {
            name: self.name.clone(),
            text: self.text.clone(),
            category: self.category.clone(),
            bank_hi: self.bank_hi,
            bank_lo: self.bank_lo,
            number: self.number,
            nofn: self.nofn,
        });
        self.start();
        result
    }
}
