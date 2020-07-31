use eye::device::{ControlInfo, Info as DeviceInfo};

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

#[derive(Clone)]
pub struct Control {
    pub id: u32,
    pub name: String,

    pub representation: ControlRepresentation,
    pub value: ControlValue,
}

impl core::convert::From<&ControlInfo> for Control {
    fn from(info: &ControlInfo) -> Self {
        Control {
            id: info.id,
            name: info.name.clone(),
            representation: info.repr.clone(),
            value: ControlValue::None,
        }
    }
}

impl std::fmt::Debug for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Control")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

impl std::fmt::Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub type ControlRepresentation = eye::control::Representation;
pub type ControlValue = eye::control::Value;
