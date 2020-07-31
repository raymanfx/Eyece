#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Format {
    pub width: u32,
    pub height: u32,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}
