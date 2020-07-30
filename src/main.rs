mod model;

use std::{collections::VecDeque, mem};

use eye::hal::traits::{Device, Stream};
use eye::prelude::*;

use ffimage::packed::dynamic::ImageView as DynamicImageView;

use iced::widget::image;
use iced::{
    executor, futures, pick_list, scrollable, time, Application, Column, Command, Element, Image,
    Length, PickList, Row, Scrollable, Settings, Subscription, Text,
};

fn main() {
    Eyece::run(Settings::default())
}

#[derive(Default)]
struct Eyece<'a> {
    // Keep the order of these two!
    // The stream must be dropped before the device is.
    stream: Option<Box<dyn Stream<Item = DynamicImageView<'a>>>>,
    device: Option<Box<dyn Device>>,
    image: Option<image::Handle>,

    devices: Vec<model::device::Node>,
    device_list: pick_list::State<model::device::Node>,
    device_selection: Option<model::device::Node>,

    formats: Vec<model::device::Format>,
    format_list: pick_list::State<model::device::Format>,
    format_selection: Option<model::device::Format>,

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
    LogLevelSelected(model::log::Level),
    Log(model::log::Level, String),
    NextFrame,
}

impl<'a> Application for Eyece<'a> {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Perform initial device enumeration.
        // TODO: Async?
        let devices: Vec<model::device::Node> = DeviceList::enumerate()
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

    fn subscription(&self) -> Subscription<Message> {
        if self.stream.is_some() {
            time::every(std::time::Duration::from_millis(33)).map(|_| Message::NextFrame)
        } else {
            Subscription::none()
        }
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
                    // enumerate formats
                    self.formats = Vec::new();
                    for info in device.query_formats().unwrap() {
                        if info.pixfmt == format.pixfmt {
                            for res in info.resolutions {
                                self.formats.push(model::device::Format {
                                    width: res.0,
                                    height: res.1,
                                });
                            }
                        }
                    }

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
            Message::FormatSelected(fmt) => {
                self.format_selection = Some(fmt);

                self.update(Message::Log(
                    model::log::Level::Info,
                    format!("Message::FormatSelected: {}x{}", fmt.width, fmt.height),
                ));

                // we need to destroy the stream to apply new parameters
                self.stream = None;

                // read the current format and set the resolution
                let device = self.device.as_mut().unwrap();
                let mut format = device.get_format().unwrap();
                format.width = fmt.width;
                format.height = fmt.height;
                device.set_format(&format).unwrap();

                // recreate the stream with the new format
                unsafe {
                    self.stream = Some(mem::transmute(device.stream().unwrap()));
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
            Message::NextFrame => {
                self.image = match &mut self.stream {
                    Some(stream) => {
                        let image = stream.next().unwrap();
                        let pixels = image.raw().as_slice().unwrap().to_vec();

                        Some(image::Handle::from_pixels(
                            image.width(),
                            image.height(),
                            pixels,
                        ))
                    }
                    None => None,
                };
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

        let mut image = Row::new();
        if let Some(handle) = self.image.as_ref().cloned() {
            image = image.push(Image::new(handle));
        }

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
            .push(image)
            .push(debug)
            .push(logs)
            .into()
    }
}
