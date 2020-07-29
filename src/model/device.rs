use eye::device::Info as DeviceInfo;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Info {
    pub index: u32,
    pub name: String,
}

impl core::convert::From<&DeviceInfo> for Info {
    fn from(info: &DeviceInfo) -> Self {
        Info {
            index: info.index,
            name: info.name.clone(),
        }
    }
}

impl std::fmt::Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.index, self.name)
    }
}
