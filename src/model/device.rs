use std::ops::{Deref, RangeInclusive};

use eye::device::{ControlInfo, Info as DeviceInfo};
use iced::{button, slider};

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

#[derive(Debug, Default, Clone)]
pub struct Control {
    pub id: u32,
    pub name: String,

    pub state: Option<ControlState>,
}

impl core::convert::From<&ControlInfo> for Control {
    fn from(info: &ControlInfo) -> Self {
        let state = match &info.repr {
            eye::control::Representation::Button => {
                Some(ControlState::Button(button::State::default()))
            }
            eye::control::Representation::Boolean => Some(ControlState::Checkbox(false)),
            eye::control::Representation::Integer(int) => Some(ControlState::Slider(SliderState {
                range: RangeInclusive::new(int.range.0 as f64, int.range.1 as f64),
                step: int.step as f64,
                value: int.default as f64,
                state: slider::State::default(),
            })),
            _ => None,
        };

        Control {
            id: info.id,
            name: info.name.clone(),
            state,
        }
    }
}

impl std::fmt::Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub enum ControlState {
    Button(button::State),
    Checkbox(bool),
    Slider(SliderState<f64>),
}

impl core::convert::From<&ControlState> for eye::control::Value {
    fn from(state: &ControlState) -> Self {
        match state {
            ControlState::Button(_) => eye::control::Value::None,
            ControlState::Checkbox(state) => eye::control::Value::Boolean(*state),
            ControlState::Slider(state) => eye::control::Value::Integer(state.value as i64),
        }
    }
}

impl std::fmt::Display for ControlState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlState::Button(_) => write!(f, "None"),
            ControlState::Checkbox(state) => write!(f, "{}", state),
            ControlState::Slider(state) => write!(f, "{}", state.value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SliderState<T> {
    pub state: slider::State,

    pub range: RangeInclusive<T>,
    pub step: T,
    pub value: T,
}

impl<T> Deref for SliderState<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
