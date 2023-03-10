pub static UNKNOWN:&str = "(unknown)";

pub trait Named {
    fn name(&self) -> &'static str;
}
pub trait Described {
    fn describe(&self) -> String;
}

pub trait ParseByte<T> {
    fn parse(raw:u8) -> Option<T>;
}
