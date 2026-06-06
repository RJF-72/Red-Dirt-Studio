use std::cell::RefCell;
use std::rc::Rc;
use rodio::{OutputStream, OutputStreamHandle, Sink, buffer::SamplesBuffer};

pub struct AudioPlayer {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Option<Sink>,
}

impl PartialEq for AudioPlayer {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl AudioPlayer {
    pub fn new() -> anyhow::Result<Self> {
        let (stream, handle) = OutputStream::try_default()?;
        Ok(Self { _stream: stream, handle, sink: None })
    }

    pub fn play(&mut self, data: &[f32], sample_rate: u32, channels: u16) {
        if let Some(s) = &self.sink {
            s.stop();
        }
        if let Ok(sink) = Sink::try_new(&self.handle) {
            let source = SamplesBuffer::new(channels, sample_rate, data.to_vec());
            sink.append(source);
            self.sink = Some(sink);
        }
    }

    pub fn stop(&mut self) {
        if let Some(s) = self.sink.take() {
            s.stop();
        }
    }

    #[allow(dead_code)]
    pub fn is_playing(&self) -> bool {
        self.sink.as_ref().map(|s| !s.empty()).unwrap_or(false)
    }
}

pub type SharedPlayer = Rc<RefCell<AudioPlayer>>;

pub fn create_player() -> anyhow::Result<SharedPlayer> {
    Ok(Rc::new(RefCell::new(AudioPlayer::new()?)))
}
