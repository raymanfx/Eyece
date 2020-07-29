mod model;

use std::mem;

use eye::hal::traits::{Device, Stream};
use eye::prelude::*;

use ffimage::packed::dynamic::ImageView as DynamicImageView;

use iced::{
    executor, futures, pick_list, Application, Column, Command, Element, PickList, Row, Settings,
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
}

#[derive(Debug, Clone)]
enum Message {
    EnumerateDevices(Vec<model::device::Info>),
    DeviceSelected(model::device::Info),
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
                }
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

        Column::new().padding(PADDING).push(config).into()
    }
}
