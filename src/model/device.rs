use eye::device::Info as DeviceInfo;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Device {
    pub index: u32,
    pub name: String,
}

impl core::convert::From<&DeviceInfo> for Device {
    fn from(info: &DeviceInfo) -> Self {
        Device {
            index: info.index,
            name: info.name.clone(),
        }
    }
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.index, self.name)
    }
}
