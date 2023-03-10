#[repr(u8)]
#[derive(Clone, Copy, Default, PartialEq)]
pub enum GatherState {
    #[default]
    None,
    Name,
    Text,
    Category,
    Binary,
}
