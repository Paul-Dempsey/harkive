#[allow(non_upper_case_globals, dead_code)]
#[rustfmt::skip]
pub mod cc16 {
pub const BankSelect:u8         = 0;

pub const OctaveShift:u8        = 8;
pub const MonoSwitch:u8         = 9;
pub const FineTune:u8           = 10;

pub const Macro_i:u8            = 12;
pub const Macro_ii:u8           = 13;
pub const Macro_iii:u8          = 14;
pub const Macro_iv:u8           = 15;
pub const Macro_v:u8            = 16;
pub const Macro_vi:u8           = 17;
pub const PostLevel:u8          = 18;
pub const AudioLevelIn:u8       = 19;
pub const ReciR1:u8             = 20;
pub const ReciR2:u8             = 21;
pub const ReciR3:u8             = 22;
pub const ReciR4:u8             = 23;
pub const ReciR5:u8             = 24;
pub const RoundRate:u8          = 25;
pub const PreLevel:u8           = 26;
pub const OutputAttenuation:u8  = 27;
pub const RoundInitial:u8       = 28;
pub const Pedal1Value:u8        = 29;
pub const Pedal2Value:u8        = 30;
pub const AdvanceNextPreset:u8  = 31;
pub const PresetGroup:u8        = 32;
pub const AesBigFontNoRecirc:u8 = 33;
pub const PresetAlgprithm:u8    = 34;
pub const MidiProgramNumber:u8  = 35;
pub const MidiRouting:u8        = 36;
pub const PedalType:u8          = 37;

pub const Polyphony:u8          = 39;
pub const BendRange:u8          = 40;


pub const DataStream:u8         = 56;

pub const FirmwareVersionHi:u8  = 102;
pub const FirmwareVersionLo:u8  = 103;

pub const DownloadControl:u8    = 109;
pub const DownloadInfo:u8       = 110;
pub const DeviceStatus:u8       = 111;

pub const DspPercent:u8         = 114;

// ----  values  ------------------------------

// DataStream values
pub const DataStream_Name:u8           = 0;
pub const DataStream_Text:u8           = 1;
pub const DataStream_Graph:u8           = 2;
pub const DataStream_GraphOffset1:u8    = 3;
pub const DataStream_GraphOffset2:u8    = 4;
pub const DataStream_GraphT0:u8         = 5;
pub const DataStream_GraphT1:u8         = 6;
pub const DataStream_Log:u8             = 7;
pub const DataStream_Category:u8        = 8;
pub const DataStream_DemoAssort:u8      = 9;
pub const DataStream_Float:u8           = 10;
pub const DataStream_Kinetic:u8         = 11;
pub const DataStream_BiquadSin:u8       = 12;
pub const DataStream_System:u8          = 13;
pub const DataStream_Convolution:u8     = 14;
pub const DataStream_End:u8             = 127;

// DownloadControl values
pub const DownloadControl_ArchiveOk:u8   = 5;
pub const DownloadControl_ArchiveFail:u8 = 6;

pub const DownloadControl_DspDone:u8     = 26;

pub const DownloadControl_BeginUserNames:u8   = 54;
pub const DownloadControl_EndUserNames:u8     = 55;

pub const DownloadControl_EndSystemNames:u8   = 40;
pub const DownloadControl_BeginSystemNames:u8 = 49;

// DownloadInfo values
pub const DownloadInfo_BeginArchive:u8  = 120;
pub const DownloadInfo_EndArchive:u8    = 124;

}
