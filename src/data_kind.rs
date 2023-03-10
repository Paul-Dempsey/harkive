use crate::midi_traits::*;

#[repr(u8)]
#[derive(Copy, Clone, Default, PartialEq)]
pub enum DataKind {
    Name,
    ControlText,
    Graph,
    GraphOffset1,
    GraphOffset2,
    GraphT0,
    GraphT1,
    Log,
    Category,
    DemoAssort,
    Float,
    Kinetic,
    BiquadSin,
    System,
    Convolution,
    #[default]
    Unknown,
}
impl DataKind {
    pub fn new(raw: u8) -> Self {
        if raw <= Self::Convolution as u8 {
            unsafe { ::std::mem::transmute(raw) }
        } else {
            DataKind::Unknown
        }
    }
}
impl Named for DataKind {
    #[rustfmt::skip]
    fn name(&self) -> &'static str {
        match self {
            Self::Name          => "Name",
            Self::ControlText   => "Control Text",
            Self::Graph         => "Graph",
            Self::GraphOffset1  => "Graph Offset 1",
            Self::GraphOffset2  => "Graph Offset 2",
            Self::GraphT0       => "Graph T0",
            Self::GraphT1       => "Graph T1",
            Self::Log           => "Log",
            Self::Category      => "Category",
            Self::DemoAssort    => "Demo Assortment",
            Self::Float         => "Float",
            Self::Kinetic       => "Kinetic",
            Self::BiquadSin     => "Biquad Sine",
            Self::System        => "System",
            Self::Convolution   => "Convolution",
            Self::Unknown       => "Unknown",
        }
    }
}
