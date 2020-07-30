use eye::device::Info as DeviceInfo;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Node {
    pub index: u32,
    pub name: String,
}

impl core::convert::From<&DeviceInfo> for Node {
    fn from(info: &DeviceInfo) -> Self {
        Node {
            index: info.index,
            name: info.name.clone(),
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.index, self.name)
    }
}

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
