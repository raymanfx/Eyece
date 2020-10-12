#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Device {
    pub uri: String,
}

impl core::convert::From<&str> for Device {
    fn from(uri: &str) -> Self {
        Device {
            uri: uri.to_string(),
        }
    }
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uri)
    }
}
