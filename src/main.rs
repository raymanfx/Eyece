mod model;

mod stream;

use std::collections::VecDeque;

use eye::hal::traits::Device;
use eye::prelude::*;

use iced::widget::image;
use iced::{
    button, executor, pick_list, scrollable, slider, Application, Button, Checkbox, Column,
    Command, Element, Image, Length, PickList, Row, Scrollable, Settings, Slider, Subscription,
    Text,
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

macro_rules! unwrap_or_continue {
    ( $e:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => continue,
        }
    };
    ( $e:expr, $closure:tt ) => {
        match $e {
            Ok(x) => x,
            Err(err) => {
                $closure(err);
                continue;
            }
        }
    };
}

fn main() {
    Eyece::run(Settings::default())
}

#[derive(Default)]
struct Eyece {
    device: Option<Box<dyn Device>>,
    image: Option<image::Handle>,

    config: Config,
    controls: Controls,
    log: Log,

    streaming: bool,
}

#[derive(Debug, Clone)]
enum Message {
    DeviceSelected(model::device::Device),
    FormatSelected(model::format::Format),
    ControlChanged(model::control::Control),
    ConfigMessage(ConfigMessage),
    ControlsMessage(ControlsMessage),
    LogMessage(LogMessage),
    Stream(bool),
    StreamEvent(stream::Event),
}

impl Application for Eyece {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Perform initial device enumeration.
        // TODO: Async?
        let devices: Vec<model::device::Device> = DeviceFactory::enumerate()
            .iter()
            .map(|dev| model::device::Device::from(dev))
            .collect();

        let mut eyece = Eyece {
            ..Default::default()
        };

        eyece.config.devices = devices;
        eyece.log.level = model::log::Level::Warn;

        (eyece, Command::none())
    }

    fn title(&self) -> String {
        String::from("Eyece")
    }

    fn subscription(&self) -> Subscription<Message> {
        if !self.streaming {
            return Subscription::none();
        }

        match &self.device {
            Some(dev) => {
                iced::Subscription::from_recipe(stream::ImageStream::new(dev.stream().unwrap()))
                    .map(Message::StreamEvent)
            }
            _ => Subscription::none(),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::DeviceSelected(dev) => {
                // dispose of any existing streams
                self.streaming = false;
                self.device = None;

                // open the device
                let mut device = unwrap_or_return!(
                    DeviceFactory::create(dev.index as usize),
                    Command::none(),
                    (|err| self.log.update(LogMessage::Log(
                        model::log::Level::Warn,
                        format!("Message::DeviceSelected: Failed to open device ({})", err),
                    )))
                );

                // read the current format
                let mut format = unwrap_or_return!(
                    device.format(),
                    Command::none(),
                    (|err| self.log.update(LogMessage::Log(
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
                    (|err| self.log.update(LogMessage::Log(
                        model::log::Level::Warn,
                        format!("Message::DeviceSelected: Failed to write format ({})", err),
                    )))
                );

                if format.pixfmt == PixelFormat::Bgra(32) {
                    // enumerate formats
                    let formats = unwrap_or_return!(
                        device.query_formats(),
                        Command::none(),
                        (|err| self.log.update(LogMessage::Log(
                            model::log::Level::Warn,
                            format!(
                                "Message::DeviceSelected: Failed to query resolutions ({})",
                                err
                            ),
                        )))
                    );

                    let mut resolutions = Vec::new();
                    for info in formats {
                        if info.pixfmt == format.pixfmt {
                            for res in info.resolutions {
                                resolutions.push(model::format::Format {
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
                        (|err| self.log.update(LogMessage::Log(
                            model::log::Level::Warn,
                            format!(
                                "Message::DeviceSelected: Failed to query controls ({})",
                                err
                            ),
                        )))
                    );
                    self.controls.controls = controls
                        .iter()
                        .map(|ctrl| {
                            let model = model::control::Control::from(ctrl);
                            let state = match &model.representation {
                                model::control::Representation::Button => {
                                    Some(ControlState::Button(button::State::default()))
                                }
                                model::control::Representation::Boolean => None,
                                model::control::Representation::Integer(_) => {
                                    Some(ControlState::Slider(slider::State::default()))
                                }
                                _ => None,
                            };
                            (model, state)
                        })
                        .collect();

                    // query control values
                    for (control, _) in &mut self.controls.controls {
                        let value;
                        match &control.representation {
                            model::control::Representation::Boolean
                            | model::control::Representation::Integer(_) => {
                                value = unwrap_or_continue!(device.control(control.id));
                                control.value = value;
                            }
                            _ => continue,
                        }
                    }

                    // create the buffer stream
                    self.device = Some(device);
                    self.update(Message::Stream(true));

                    // update UI state
                    self.config.device = Some(dev);
                    self.config.formats = resolutions;

                    self.log.update(LogMessage::Log(
                        model::log::Level::Info,
                        format!("Message::DeviceSelected: Found suitable device (BGRA), resolution = {}x{}", format.width, format.height),
                    ));
                } else {
                    self.log.update(LogMessage::Log(
                        model::log::Level::Warn,
                        format!("Message::DeviceSelected: Device does not offer BGRA buffers"),
                    ));
                }
            }
            Message::FormatSelected(fmt) => {
                // we need to destroy the stream to apply new parameters
                self.streaming = false;

                // read the current forma    };t and set the resolution
                let device = self.device.as_mut().unwrap();
                let mut format = unwrap_or_return!(
                    device.format(),
                    Command::none(),
                    (|err| self.log.update(LogMessage::Log(
                        model::log::Level::Warn,
                        format!("Message::FormatSelected: Failed to read format ({})", err),
                    )))
                );
                format.width = fmt.width;
                format.height = fmt.height;
                let format = unwrap_or_return!(
                    device.set_format(&format),
                    Command::none(),
                    (|err| self.log.update(LogMessage::Log(
                        model::log::Level::Warn,
                        format!("Message::FormatSelected: Failed to write format ({})", err),
                    )))
                );

                // recreate the stream with the new format
                self.update(Message::Stream(true));

                // update UI state
                self.config.format = Some(fmt);

                self.log.update(LogMessage::Log(
                    model::log::Level::Info,
                    format!(
                        "Message::FormatSelected: {}x{}",
                        format.width, format.height
                    ),
                ));
            }
            Message::ControlChanged(control) => {
                let device = self.device.as_mut().unwrap();

                for ctrl in &mut self.controls.controls {
                    if ctrl.0.id == control.id {
                        unwrap_or_return!(
                            device.set_control(ctrl.0.id, &control.value),
                            Command::none(),
                            (|err| self.log.update(LogMessage::Log(
                                model::log::Level::Warn,
                                format!(
                                    "Message::ControlChanged: Failed to write control ({})",
                                    err
                                ),
                            )))
                        );

                        // we successfully wrote the control value, now update its state
                        ctrl.0.value = control.value.clone();
                    }
                }

                let value_str = match &control.value {
                    model::control::Value::None => "None".to_string(),
                    model::control::Value::String(val) => val.to_string(),
                    model::control::Value::Boolean(val) => val.to_string(),
                    model::control::Value::Integer(val) => val.to_string(),
                };
                self.log.update(LogMessage::Log(
                    model::log::Level::Info,
                    format!(
                        "Message::ControlChanged: ID: {}, Value: {}",
                        control.id, value_str
                    ),
                ));
            }
            Message::ConfigMessage(msg) => {
                for msg in self.config.update(msg) {
                    self.update(msg);
                }
            }
            Message::ControlsMessage(msg) => {
                for msg in self.controls.update(msg) {
                    self.update(msg);
                }
            }
            Message::LogMessage(msg) => {
                self.log.update(msg);
            }
            Message::Stream(enable) => {
                self.streaming = enable;
            }
            Message::StreamEvent(event) => match event {
                stream::Event::Advanced(handle) => self.image = Some(handle),
                _ => {}
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        // Uniform padding and spacing for all elements.
        const SPACING: u16 = 10;
        const PADDING: u16 = 10;

        let mut image = Row::new();
        if let Some(handle) = &self.image {
            image = image.push(Image::new(handle.clone()));
        }

        Column::new()
            .padding(PADDING)
            .push(self.config.view().map(|msg| Message::ConfigMessage(msg)))
            .push(
                Row::new().spacing(SPACING).push(image).push(
                    self.controls
                        .view()
                        .map(|msg| Message::ControlsMessage(msg)),
                ),
            )
            .push(self.log.view().map(|msg| Message::LogMessage(msg)))
            .into()
    }
}

#[derive(Debug, Default, Clone)]
struct Config {
    devices: Vec<model::device::Device>,
    device: Option<model::device::Device>,
    device_list: pick_list::State<model::device::Device>,

    formats: Vec<model::format::Format>,
    format: Option<model::format::Format>,
    format_list: pick_list::State<model::format::Format>,
}

#[derive(Debug, Clone)]
enum ConfigMessage {
    DeviceSelected(model::device::Device),
    FormatSelected(model::format::Format),
}

impl Config {
    fn update(&mut self, message: ConfigMessage) -> Vec<Message> {
        match message {
            ConfigMessage::DeviceSelected(dev) => vec![
                Message::LogMessage(LogMessage::Log(
                    model::log::Level::Info,
                    format!("ConfigMessage::DeviceSelected: {}: {}", dev.index, dev.name),
                )),
                Message::DeviceSelected(dev),
            ],
            ConfigMessage::FormatSelected(fmt) => vec![
                Message::LogMessage(LogMessage::Log(
                    model::log::Level::Info,
                    format!(
                        "ConfigMessage::FormatSelected: {}x{}",
                        fmt.width, fmt.height
                    ),
                )),
                Message::FormatSelected(fmt),
            ],
        }
    }

    fn view(&mut self) -> Element<ConfigMessage> {
        // Uniform padding and spacing for all elements.
        const PADDING: u16 = 10;

        // Device selection, format configuration, etc.
        Row::new()
            .padding(PADDING)
            .push(PickList::new(
                &mut self.device_list,
                &self.devices,
                self.device.clone(),
                ConfigMessage::DeviceSelected,
            ))
            .push(PickList::new(
                &mut self.format_list,
                &self.formats,
                self.format.clone(),
                ConfigMessage::FormatSelected,
            ))
            .into()
    }
}

#[derive(Debug, Default, Clone)]
struct Controls {
    state: scrollable::State,
    controls: Vec<(model::control::Control, Option<ControlState>)>,
}

#[derive(Debug, Clone)]
enum ControlState {
    Button(button::State),
    Slider(slider::State),
}

#[derive(Debug, Clone)]
enum ControlsMessage {
    ControlChanged(model::control::Control),
}

impl Controls {
    fn update(&mut self, message: ControlsMessage) -> Vec<Message> {
        match message {
            ControlsMessage::ControlChanged(ctrl) => vec![
                Message::LogMessage(LogMessage::Log(
                    model::log::Level::Info,
                    format!("ControlsMessage::ControlChanged: ID: {}", ctrl.id),
                )),
                Message::ControlChanged(ctrl),
            ],
        }
    }

    fn view(&mut self) -> Element<ControlsMessage> {
        // Uniform padding and spacing for all elements.
        const SPACING: u16 = 10;
        const PADDING: u16 = 10;

        // Device controls
        let mut controls = Scrollable::new(&mut self.state).width(Length::Fill);
        for (control, state) in &mut self.controls {
            let control_clone = control.clone();

            match &control.representation {
                model::control::Representation::Button => {
                    let state = match state.as_mut().unwrap() {
                        ControlState::Button(state) => state,
                        _ => panic!("Wrong button state"),
                    };
                    controls = controls.push(
                        Row::new()
                            .spacing(SPACING)
                            .push(Text::new(control.name.clone()))
                            .push(
                                Button::new(state, Text::new("Toggle"))
                                    .on_press(ControlsMessage::ControlChanged(control_clone)),
                            ),
                    );
                }
                model::control::Representation::Boolean => {
                    let checked = match control.value {
                        model::control::Value::Boolean(val) => Some(val),
                        model::control::Value::Integer(val) => Some(val != 0),
                        _ => None,
                    };
                    controls = controls.push(
                        Row::new()
                            .spacing(SPACING)
                            .push(Text::new(control.name.clone()))
                            .push(Checkbox::new(checked.unwrap(), "", move |val| {
                                let mut control = control_clone.clone();
                                control.value = model::control::Value::Boolean(val);
                                ControlsMessage::ControlChanged(control)
                            })),
                    );
                }
                model::control::Representation::Integer(repr) => {
                    let state = match state.as_mut().unwrap() {
                        ControlState::Slider(state) => state,
                        _ => panic!("Wrong slider state"),
                    };
                    let value = match control.value {
                        model::control::Value::Integer(val) => Some(val),
                        _ => None,
                    };
                    controls = controls.push(
                        Row::new()
                            .spacing(SPACING)
                            .push(Text::new(control.name.clone()))
                            .push(
                                Slider::new(
                                    state,
                                    (repr.range.0 as f64)..=(repr.range.1 as f64),
                                    value.unwrap() as f64,
                                    move |val| {
                                        let mut control = control_clone.clone();
                                        control.value = model::control::Value::Integer(val as i64);
                                        ControlsMessage::ControlChanged(control)
                                    },
                                )
                                .step(repr.step as f64),
                            ),
                    );
                }
                _ => continue,
            }
        }

        Column::new().padding(PADDING).push(controls).into()
    }
}

#[derive(Debug, Default, Clone)]
struct Log {
    state: scrollable::State,
    level: model::log::Level,
    level_list: pick_list::State<model::log::Level>,
    buffer: VecDeque<(model::log::Level, String)>,
}

#[derive(Debug, Clone)]
enum LogMessage {
    Log(model::log::Level, String),
    LevelSelected(model::log::Level),
}

impl Log {
    fn update(&mut self, message: LogMessage) {
        match message {
            LogMessage::Log(level, message) => {
                if self.buffer.len() > 100 {
                    self.buffer.pop_front();
                }
                self.buffer.push_back((level, message));
            }
            LogMessage::LevelSelected(level) => {
                self.level = level;
            }
        }
    }

    fn view(&mut self) -> Element<LogMessage> {
        // Uniform padding and spacing for all elements.
        const SPACING: u16 = 10;
        const PADDING: u16 = 10;

        let settings = Row::new().push(PickList::new(
            &mut self.level_list,
            &model::log::Level::ALL[..],
            Some(self.level),
            LogMessage::LevelSelected,
        ));

        let mut logs = Scrollable::new(&mut self.state)
            .width(Length::Fill)
            .height(Length::Units(100));

        for entry in &self.buffer {
            if entry.0 as u8 <= self.level as u8 {
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
            .push(settings)
            .push(logs)
            .into()
    }
}
