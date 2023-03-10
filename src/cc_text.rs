
pub struct CcText {
    data_109: std::collections::HashMap<u8, String>,
    data_110: std::collections::HashMap<u8, String>,
}
impl Default for CcText {
    fn default() -> Self {
        Self::new()
    }
}
impl CcText {
    #[rustfmt::skip]
    pub fn new() -> Self {
        let mut data_109: std::collections::HashMap<u8, String> = std::collections::HashMap::default();
        data_109.insert(0, "[Reboot device]".to_string());
        data_109.insert(1, "[Download failed, try again]".to_string());
        data_109.insert(2, "[Download in progress]".to_string());
        data_109.insert(4, "[Data download completed]".to_string());
        data_109.insert(5, "[End of Preset Group]".to_string());
        data_109.insert(6, "[Archive failed, try again]".to_string());
        data_109.insert(7, "[Query Kenton]".to_string());
        data_109.insert(8, "[End of Archive Retrieve]".to_string());
        data_109.insert(9, "[Reduce Gain]".to_string());
        data_109.insert(10, "[Reduce Polyphony]".to_string());
        data_109.insert(11, "[Factory Calibration In Progress]".to_string());
        data_109.insert(12, "[ERASE]".to_string());
        data_109.insert(13, "[AES Sync Failure]".to_string());
        data_109.insert(14, "[Turn On or Disconnect CVC]".to_string());
        data_109.insert(15, "[Firmware version mismatch]".to_string());
        data_109.insert(16, "[Config to MIDI]".to_string());
        data_109.insert(17, "[Begin firmware download]".to_string());
        data_109.insert(18, "[Begin data download]".to_string());
        data_109.insert(19, "[done with firmware 21364 download]".to_string());
        data_109.insert(20, "[End data download]".to_string());
        data_109.insert(21, "[MIDI loopback detected]".to_string());
        
        data_109.insert(24, "[begin CEE config send txDsp (from daisy=1 to 2,3)  ]".to_string());
        data_109.insert(25, "[end CEE config send txDsp (from daisy=1 to 2,3)  	]".to_string());
        data_109.insert(26, "[---- Begin preset ----]".to_string()); // handshake back: end of txDsp preset-sending process
        data_109.insert(27, "[config send txDsp failure - could try again?]".to_string());
        data_109.insert(28, "[after Update File 1 reboot, do Update]".to_string());
        data_109.insert(29, "[Yellow LED for archive create]".to_string());
        data_109.insert(30, "[HE->Dev: begin Midi stress test]".to_string());
        data_109.insert(31, "[HE<-Dev: error in Midi rx sequence]".to_string());
        data_109.insert(32, "[HE->Dev: preset names to Midi, then current config]".to_string());
        data_109.insert(33, "[old preset needs manually-implemented update]".to_string());
        data_109.insert(34, "[Reset Calibration]".to_string());
        data_109.insert(35, "[Refine Calibration]".to_string());
        data_109.insert(36, "[full midi transmission rate]".to_string());
        data_109.insert(37, "[one-third midi transmission rate]".to_string());
        data_109.insert(38, "[one-twentieth midi transmission rate]".to_string());
        data_109.insert(39, "[---- Begin system presets ----]".to_string());
        data_109.insert(40, "[---- End system presets ----]".to_string());
        data_109.insert(41, "[Factory Calibration]".to_string());
        data_109.insert(42, "[ready for update after recovery boot]".to_string());
        data_109.insert(43, "[done with firmware 21489 download, burn user flash]".to_string());
        data_109.insert(44, "[reboot after Firmware File 1]".to_string());
        data_109.insert(45, "[toggle Slim Continuum surface alignment mode]".to_string());
        data_109.insert(46, "[add currently-playing finger to Trim array]".to_string());
        data_109.insert(47, "[remove trim point closest to currently-playing finger]".to_string());
        data_109.insert(48, "[remove all trim data]".to_string());
        data_109.insert(49, "[---- Begin system presets ----]".to_string());
        data_109.insert(50, "[exit Combination Preset mode]".to_string());
        data_109.insert(51, "[store calib/global/userPresets to continuuMini factory setup]".to_string());
        data_109.insert(52, "[to prev sysPreset]".to_string());
        data_109.insert(53, "[to next sysPreset]".to_string());
        data_109.insert(54, "[---- Begin Preset Names ----]".to_string());
        data_109.insert(55, "[---- End Preset Names ----]".to_string());
        data_109.insert(56, "[save Combi preset to same slot or to disk]".to_string());
        data_109.insert(60, "[Remake QSPI data]".to_string());
        data_109.insert(63, "[Usb-Midi out from Mini did not get Ack]".to_string());			
        data_109.insert(64, "[Midi rx queue overflow]".to_string());
        data_109.insert(65, "[Midi tx queue overflow]".to_string());
        data_109.insert(66, "[Midi rx syntax error]".to_string());
        data_109.insert(67, "[Midi rx bad bit widths]".to_string());
        data_109.insert(68, "[serial sensors errors]".to_string());
        data_109.insert(69, "[output has nan]".to_string());
        data_109.insert(70, "[CEE comm glitch]".to_string());
        data_109.insert(71, "[End ContinuuMini firmware]".to_string());
        data_109.insert(72, "[end scrolling ascii log via Midi]".to_string());
        data_109.insert(73, "[daisy=0,1 scrolling ascii log via Midi]".to_string());
        data_109.insert(74, "[daisy=2 scrolling ascii log via Midi]".to_string());
        data_109.insert(75, "[daisy=3 scrolling ascii log via Midi]".to_string());
        data_109.insert(76, "[factory only]".to_string());
        data_109.insert(77, "[factory only]".to_string());
        data_109.insert(78, "[factory only]".to_string());
        data_109.insert(88, "[numDecMat| decrement numeric matrix point]".to_string());
        data_109.insert(89, "[numIncMat| increment numeric matrix point]".to_string());
        data_109.insert(90, "[mendDisco| mend discontinuity at note (outlier to Sensor Map)]".to_string());
        data_109.insert(91, "[rebootRecov| reboot in Recovery Mode]".to_string());
        data_109.insert(92, "[stageUp]".to_string());
        data_109.insert(93, "[stageDown]".to_string());
        data_109.insert(94, "[stageDownOk1]".to_string());
        data_109.insert(95, "[stageDownOk2]".to_string());
        data_109.insert(96, "[stageDownOk3]".to_string());
        data_109.insert(97, "[stageDownFail1]".to_string());
        data_109.insert(98, "[stageDownFail2]".to_string());
        data_109.insert(99, "[stageDownFail3]".to_string());
        data_109.insert(100, "[rebootUser|]".to_string());
        data_109.insert(101, "[gridToFlash|]".to_string());
        data_109.insert(102, "[Mend divided note]".to_string());
        data_109.insert(103, "[startUpdF2| <-HE: beginning of Update File 2]".to_string());
        data_109.insert(104, "[Preset not first in a combination]".to_string());
        data_109.insert(105, "[Preset first in a dual combination]".to_string());
        data_109.insert(106, "[Preset first in triple combination]".to_string());
            
        let mut data_110: std::collections::HashMap<u8, String> = std::collections::HashMap::default();
        data_110.insert(0, "[profileEnd]".to_string());
        data_110.insert(100, "[Save preset 0]".to_string());
        data_110.insert(101, "[Save Preset 1]".to_string());
        data_110.insert(102, "[Save Preset 2]".to_string());
        data_110.insert(103, "[Save Preset 3]".to_string());
        data_110.insert(104, "[Save Preset 4]".to_string());
        data_110.insert(105, "[Save Preset 5]".to_string());
        data_110.insert(106, "[Save Preset 6]".to_string());
        data_110.insert(107, "[Save Preset 7]".to_string());
        data_110.insert(108, "[Save Preset 8]".to_string());
        data_110.insert(109, "[Save Preset 9]".to_string());
        data_110.insert(110, "[Save Preset 10]".to_string());
        data_110.insert(111, "[Save Preset 11]".to_string());
        data_110.insert(112, "[Save Preset 12]".to_string());
        data_110.insert(113, "[Save Preset 13]".to_string());
        data_110.insert(114, "[Save Preset 14]".to_string());
        data_110.insert(115, "[Save Preset 15]".to_string());
        data_110.insert(116, "[Save Preset 16]".to_string());
        data_110.insert(118, "Download in progress. Please wait".to_string());
        data_110.insert(119, "[archiveNop]".to_string());
        data_110.insert(120, "[edRecordArchive]".to_string());
        data_110.insert(121, "[cfRetrieveArch]".to_string());
        data_110.insert(123, "[archiveEof]".to_string());
        data_110.insert(124, "[archiveToFile]".to_string());
        data_110.insert(125, "[Finalizing]".to_string());
        data_110.insert(126, "[Initializing]".to_string());
        data_110.insert(127, "[Profile is being generated. Please wait.]".to_string());

        Self { data_109, data_110 }
    }
    pub fn get(&self, cc:u8, key:u8) -> Option<String> {
        match cc {
            109 => {
                return match key {
                    80..=87 => Some(format!("[Download tuning grid {}]", 1 + key - 80)),
                    107..=114 => Some(format!("[Demo assortment to group {}]", 1 + key - 107)),
                    115..=122 =>  Some(format!("[Erase group {}]", 1 + key - 115)),
                    _=> self.data_109.get(&key).cloned(),
                }
            }
            110 => {
                return match key {
                    1..=99 => Some(format!("{key}%")),
                    101..=116 => Some(format!("[Save preset {}]", 1 + key - 101)),
                    _=> self.data_110.get(&key).cloned(),
                };
            }
            _=> None,
        }
    }
}

