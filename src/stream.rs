use std::ops::{Deref, DerefMut};

use eye::hal::traits::Stream;
use ffimage::packed::dynamic::ImageView;

use iced_futures::futures;

pub struct ImageStream {
    stream: Wrapper,
}

impl ImageStream {
    pub fn new(stream: Box<dyn Stream<Item = ImageView<'static>> + 'static>) -> Self {
        ImageStream {
            stream: Wrapper { stream },
        }
    }
}

struct Wrapper {
    stream: Box<dyn Stream<Item = ImageView<'static>>>,
}

impl Deref for Wrapper {
    type Target = Box<dyn Stream<Item = ImageView<'static>>>;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl DerefMut for Wrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

unsafe impl Send for Wrapper {}

impl<H, I> iced_futures::subscription::Recipe<H, I> for ImageStream
where
    H: std::hash::Hasher,
{
    type Output = Event;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(
            State::Ready(self.stream),
            |state| async move {
                match state {
                    State::Ready(stream) => Some((Event::Started, State::Streaming(stream))),
                    State::Streaming(mut stream) => match stream.next() {
                        Ok(frame) => {
                            let pixels = frame.raw().as_slice().unwrap().to_vec();
                            let handle = iced::image::Handle::from_pixels(
                                frame.width(),
                                frame.height(),
                                pixels,
                            );
                            Some((Event::Advanced(handle), State::Streaming(stream)))
                        }
                        Err(_) => Some((Event::Errored, State::Broken)),
                    },
                    State::Broken => None, // TODO
                }
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Started,
    Advanced(iced::image::Handle),
    Errored,
}

enum State {
    Ready(Wrapper),
    Streaming(Wrapper),
    Broken,
}
