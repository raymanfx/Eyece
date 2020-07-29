mod model;

use std::{collections::VecDeque, mem};

use eye::hal::traits::{Device, Stream};
use eye::prelude::*;

use ffimage::packed::dynamic::ImageView as DynamicImageView;

use iced::{
    executor, futures, pick_list, scrollable, Application, Column, Command, Element, Length,
    PickList, Row, Scrollable, Settings, Text,
};

fn main() {
    Eyece::run(Settings::default())
}

#[derive(Default)]
struct Eyece<'a> {
    device: Option<Box<dyn Device>>,
    stream: Option<Box<dyn Stream<Item = DynamicImageView<'a>>>>,

    devices: Vec<model::device::Info>,
    device_list: pick_list::State<model::device::Info>,
    device_selection: Option<model::device::Info>,

    log: scrollable::State,
    loglevel_list: pick_list::State<model::log::Level>,
    loglevel_selection: model::log::Level,
    log_buffer: VecDeque<(model::log::Level, String)>,
}

#[derive(Debug, Clone)]
enum Message {
    EnumerateDevices(Vec<model::device::Info>),
    DeviceSelected(model::device::Info),
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
        let devices: Vec<model::device::Info> = DeviceList::enumerate()
            .iter()
            .map(|dev| model::device::Info::from(dev))
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
                // update UI state
                self.device_selection = Some(dev.clone());

                // dispose of any existing streams
                self.stream = None;
                self.device = None;

                // open the device and read its current format
                let mut device = DeviceFactory::create(dev.index as usize).unwrap();
                let mut format = device.get_format().unwrap();

                // Iced only supports BGRA images, so request that exact format.
                // Eye-rs will transparently convert the images on-the-fly if necessary
                // (and possible).
                format.pixfmt = PixelFormat::Bgra(32);
                format = device.set_format(&format).unwrap();
                if format.pixfmt == PixelFormat::Bgra(32) {
                    self.device = Some(device);
                    unsafe {
                        self.stream = Some(mem::transmute(
                            self.device.as_mut().unwrap().stream().unwrap(),
                        ));
                    }

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
        let config = Row::new().spacing(SPACING).push(PickList::new(
            &mut self.device_list,
            &self.devices,
            self.device_selection.clone(),
            Message::DeviceSelected,
        ));

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
            .push(debug)
            .push(logs)
            .into()
    }
}
