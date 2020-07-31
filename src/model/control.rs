use eye::device::ControlInfo;

#[derive(Clone)]
pub struct Control {
    pub id: u32,
    pub name: String,

    pub representation: Representation,
    pub value: Value,
}

impl core::convert::From<&ControlInfo> for Control {
    fn from(info: &ControlInfo) -> Self {
        Control {
            id: info.id,
            name: info.name.clone(),
            representation: info.repr.clone(),
            value: Value::None,
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

pub type Representation = eye::control::Representation;
pub type Value = eye::control::Value;
