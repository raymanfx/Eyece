use eye::control::Control as Control_;

#[derive(Clone)]
pub struct Control {
    pub id: u32,
    pub name: String,

    pub representation: Representation,
    pub value: Value,
}

impl core::convert::From<&Control_> for Control {
    fn from(ctrl: &Control_) -> Self {
        Control {
            id: ctrl.id,
            name: ctrl.name.clone(),
            representation: ctrl.repr.clone(),
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
