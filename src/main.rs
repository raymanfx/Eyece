mod model;

use std::{collections::VecDeque, mem};

use eye::hal::traits::{Device, Stream};
use eye::prelude::*;

use ffimage::packed::dynamic::ImageView as DynamicImageView;

use iced::{
    executor, futures, pick_list, scrollable, Application, Button, Checkbox, Column, Command,
    Element, Length, PickList, Row, Scrollable, Settings, Slider, Text,
};

macro_rules! unwrap_or_return {
    ( $e:expr, $ret:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => return $ret,
        }
    };
    ( $e:expr, $ret:expr, $closure:tt ) => {
        match $e {
            Ok(x) => x,
            Err(err) => {
                $closure(err);
                return $ret;
            }
        }
    };
}

fn main() {
    Eyece::run(Settings::default())
}

#[derive(Default)]
struct Eyece<'a> {
    // Keep the order of these two!
    // The stream must be dropped before the device is.
    stream: Option<Box<dyn Stream<Item = DynamicImageView<'a>>>>,
    device: Option<Box<dyn Device>>,

    devices: Vec<model::device::Node>,
    device_list: pick_list::State<model::device::Node>,
    device_selection: Option<model::device::Node>,

    formats: Vec<model::device::Format>,
    format_list: pick_list::State<model::device::Format>,
    format_selection: Option<model::device::Format>,

    controls: Vec<model::device::Control>,
    control_pane: scrollable::State,

    log: scrollable::State,
    loglevel_list: pick_list::State<model::log::Level>,
    loglevel_selection: model::log::Level,
    log_buffer: VecDeque<(model::log::Level, String)>,
}

#[derive(Debug, Clone)]
enum Message {
    EnumerateDevices(Vec<model::device::Node>),
    DeviceSelected(model::device::Node),
    FormatSelected(model::device::Format),
    ControlChanged(model::device::Control),
    LogLevelSelected(model::log::Level),
    Log(model::log::Level, String),
}

impl<'a> Application for Eyece<'a> {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Perform initial device enumeration.
        // TODO: Async?
        let devices: Vec<model::device::Node> = DeviceFactory::enumerate()
            .iter()
            .map(|dev| model::device::Node::from(dev))
            .collect();

        (
            Eyece {
                loglevel_selection: model::log::Level::Warn,
                ..Default::default()
            },
            Command::perform(futures::future::ready(devices), Message::EnumerateDevices),
        )
    }

    fn title(&self) -> String {
        String::from("Eyece")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EnumerateDevices(devices) => {
                self.devices = devices;
                self.update(Message::Log(
                    model::log::Level::Info,
                    format!(
                        "Message::EnumerateDevices: Found {} device(s)",
                        self.devices.len()
                    ),
                ));
            }
            Message::DeviceSelected(dev) => {
                // dispose of any existing streams
                self.stream = None;
                self.device = None;

                // open the device
                let mut device = unwrap_or_return!(
                    DeviceFactory::create(dev.index as usize),
                    Command::none(),
                    (|err| self.update(Message::Log(
                        model::log::Level::Warn,
                        format!("Message::DeviceSelected: Failed to open device ({})", err),
                    )))
                );

                // read the current format
                let mut format = unwrap_or_return!(
                    device.format(),
                    Command::none(),
                    (|err| self.update(Message::Log(
                        model::log::Level::Warn,
                        format!("Message::DeviceSelected: Failed to read format ({})", err),
                    )))
                );

                // Iced only supports BGRA images, so request that exact format.
                // Eye-rs will transparently convert the images on-the-fly if necessary
                // (and possible).
                format.pixfmt = PixelFormat::Bgra(32);
                let format = unwrap_or_return!(
                    device.set_format(&format),
                    Command::none(),
                    (|err| self.update(Message::Log(
                        model::log::Level::Warn,
                        format!("Message::DeviceSelected: Failed to write format ({})", err),
                    )))
                );

                if format.pixfmt == PixelFormat::Bgra(32) {
                    // enumerate formats
                    self.formats = Vec::new();
                    let formats = unwrap_or_return!(
                        device.query_formats(),
                        Command::none(),
                        (|err| self.update(Message::Log(
                            model::log::Level::Warn,
                            format!(
                                "Message::DeviceSelected: Failed to query resolutions ({})",
                                err
                            ),
                        )))
                    );

                    for info in formats {
                        if info.pixfmt == format.pixfmt {
                            for res in info.resolutions {
                                self.formats.push(model::device::Format {
                                    width: res.0,
                                    height: res.1,
                                });
                            }
                        }
                    }

                    // enumerate controls
                    let controls = unwrap_or_return!(
                        device.query_controls(),
                        Command::none(),
                        (|err| self.update(Message::Log(
                            model::log::Level::Warn,
                            format!(
                                "Message::DeviceSelected: Failed to query controls ({})",
                                err
                            ),
                        )))
                    );
                    self.controls = controls
                        .iter()
                        .map(|ctrl| model::device::Control::from(ctrl))
                        .collect();

                    // query control values
                    for ctrl in &mut self.controls {
                        if let Some(state) = &mut ctrl.state {
                            let value = unwrap_or_return!(
                                device.control(ctrl.id),
                                Command::none(),
                                (|err| self.update(Message::Log(
                                    model::log::Level::Warn,
                                    format!(
                                        "Message::DeviceSelected: Failed to read control ({})",
                                        err
                                    ),
                                )))
                            );
                            match state {
                                model::device::ControlState::Checkbox(state) => match value {
                                    eye::control::Value::Integer(val) => *state = val != 0,
                                    eye::control::Value::Boolean(val) => *state = val,
                                    _ => continue,
                                },
                                model::device::ControlState::Slider(state) => match value {
                                    eye::control::Value::Integer(val) => state.value = val as f64,
                                    _ => continue,
                                },
                                _ => continue,
                            }
                        }
                    }

                    self.device = Some(device);
                    unsafe {
                        self.stream = Some(mem::transmute(
                            self.device.as_mut().unwrap().stream().unwrap(),
                        ));
                    }

                    // update UI state
                    self.device_selection = Some(dev.clone());

                    self.update(Message::Log(
                        model::log::Level::Info,
                        format!("Message::DeviceSelected: Found suitable device (BGRA), resolution = {}x{}", format.width, format.height),
                    ));
                } else {
                    self.update(Message::Log(
                        model::log::Level::Warn,
                        format!("Message::DeviceSelected: Device does not offer BGRA buffers"),
                    ));
                }
            }
            Message::FormatSelected(fmt) => {
                // we need to destroy the stream to apply new parameters
                self.stream = None;

                // read the current format and set the resolution
                let device = self.device.as_mut().unwrap();
                let mut format = unwrap_or_return!(
                    device.format(),
                    Command::none(),
                    (|err| self.update(Message::Log(
                        model::log::Level::Warn,
                        format!("Message::FormatSelected: Failed to read format ({})", err),
                    )))
                );
                format.width = fmt.width;
                format.height = fmt.height;
                let format = unwrap_or_return!(
                    device.set_format(&format),
                    Command::none(),
                    (|err| self.update(Message::Log(
                        model::log::Level::Warn,
                        format!("Message::FormatSelected: Failed to write format ({})", err),
                    )))
                );

                // recreate the stream with the new format
                unsafe {
                    self.stream = Some(mem::transmute(device.stream().unwrap()));
                }

                // update UI state
                self.format_selection = Some(fmt);

                self.update(Message::Log(
                    model::log::Level::Info,
                    format!(
                        "Message::FormatSelected: {}x{}",
                        format.width, format.height
                    ),
                ));
            }
            Message::ControlChanged(control) => {
                let device = self.device.as_mut().unwrap();

                for ctrl in &mut self.controls {
                    if ctrl.id == control.id {
                        let value = match &control.state {
                            Some(state) => state.into(),
                            None => eye::control::Value::None,
                        };
                        unwrap_or_return!(
                            device.set_control(ctrl.id, &value),
                            Command::none(),
                            (|err| self.update(Message::Log(
                                model::log::Level::Warn,
                                format!(
                                    "Message::ControlChanged: Failed to write control ({})",
                                    err
                                ),
                            )))
                        );

                        // we successfully wrote the control value, now update its state
                        ctrl.state = control.state.clone();
                    }
                }

                let state_str = match &control.state {
                    Some(state) => state.to_string(),
                    None => "None".to_string(),
                };
                self.update(Message::Log(
                    model::log::Level::Info,
                    format!(
                        "Message::ControlChanged: ID: {}, State: {}",
                        control.id, state_str
                    ),
                ));
            }
            Message::LogLevelSelected(level) => {
                self.loglevel_selection = level;
            }
            Message::Log(level, message) => {
                if self.log_buffer.len() > 100 {
                    self.log_buffer.pop_front();
                }
                self.log_buffer.push_back((level, message));
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        // Uniform padding and spacing for all elements.
        const SPACING: u16 = 10;
        const PADDING: u16 = 10;

        // Device selection, format configuration, etc.
        let config = Row::new()
            .spacing(SPACING)
            .push(PickList::new(
                &mut self.device_list,
                &self.devices,
                self.device_selection.clone(),
                Message::DeviceSelected,
            ))
            .push(PickList::new(
                &mut self.format_list,
                &self.formats,
                self.format_selection.clone(),
                Message::FormatSelected,
            ));

        // Device controls
        let mut controls = Scrollable::new(&mut self.control_pane).width(Length::Fill);
        for control in &mut self.controls {
            let control_clone = control.clone();
            if let Some(state) = &mut control.state {
                match state {
                    model::device::ControlState::Button(state) => {
                        controls = controls.push(
                            Row::new()
                                .spacing(SPACING)
                                .push(Text::new(control.name.clone()))
                                .push(
                                    Button::new(state, Text::new("Toggle"))
                                        .on_press(Message::ControlChanged(control_clone)),
                                ),
                        );
                    }
                    model::device::ControlState::Checkbox(state) => {
                        controls = controls.push(
                            Row::new()
                                .spacing(SPACING)
                                .push(Text::new(control.name.clone()))
                                .push(Checkbox::new(*state, "", move |val| {
                                    //control_clone.state = Some(model::device::ControlState::Checkbox(val));
                                    let control = model::device::Control {
                                        id: control_clone.id,
                                        name: control_clone.name.clone(),
                                        state: Some(model::device::ControlState::Checkbox(val)),
                                    };
                                    Message::ControlChanged(control)
                                })),
                        );
                    }
                    model::device::ControlState::Slider(state) => {
                        let state_clone = state.clone();
                        controls = controls.push(
                            Row::new()
                                .spacing(SPACING)
                                .push(Text::new(control.name.clone()))
                                .push(
                                    Slider::new(
                                        &mut state.state,
                                        state.range.clone(),
                                        state.value,
                                        move |val| {
                                            //control_clone.state = Some(model::device::ControlState::Checkbox(val));
                                            let control = model::device::Control {
                                                id: control_clone.id,
                                                name: control_clone.name.clone(),
                                                state: Some(model::device::ControlState::Slider(
                                                    model::device::SliderState {
                                                        range: state_clone.range.clone(),
                                                        step: state_clone.step,
                                                        value: val,
                                                        state: state_clone.state,
                                                    },
                                                )),
                                            };
                                            Message::ControlChanged(control)
                                        },
                                    )
                                    .step(state.step),
                                ),
                        );
                    }
                }
            }
        }

        // Main area: live preview + controls
        let main = Row::new()
            .spacing(SPACING)
            .push(Column::new().push(controls));

        // Debug panel.
        let debug = Row::new().push(PickList::new(
            &mut self.loglevel_list,
            &model::log::Level::ALL[..],
            Some(self.loglevel_selection),
            Message::LogLevelSelected,
        ));

        // Log messages.
        let mut logs = Scrollable::new(&mut self.log)
            .width(Length::Fill)
            .height(Length::Units(100));

        for entry in &self.log_buffer {
            if entry.0 as u8 <= self.loglevel_selection as u8 {
                logs = logs.push(
                    Row::new()
                        .spacing(SPACING)
                        .push(Text::new(format!("[{}]", entry.0)))
                        .push(Text::new(entry.1.clone())),
                );
            }
        }

        Column::new()
            .padding(PADDING)
            .push(config)
            .push(main)
            .push(debug)
            .push(logs)
            .into()
    }
}
