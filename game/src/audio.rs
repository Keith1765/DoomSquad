use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

pub struct Audio {
    _stream: OutputStream, // must stay alive
    handle: OutputStreamHandle,
    music_sink: Option<Sink>,
    sfx_data: HashMap<String, Arc<[u8]>>,
    last_step_time: Instant,
    step_interval: Duration,
}

impl Audio {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, handle) = OutputStream::try_default()?;
        Ok(Self {
            _stream: stream,
            handle,
            music_sink: None,
            sfx_data: HashMap::new(),
            last_step_time: Instant::now() - Duration::from_millis(250),
            step_interval: Duration::from_millis(250),
        })
    }

    pub fn load_sfx<P: AsRef<Path>>(&mut self, name: &str, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.sfx_data.insert(name.to_string(), Arc::from(std::fs::read(path)?));
        Ok(())
    }

    pub fn play_music_loop<P: AsRef<Path>>(&mut self, path: P, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
        self.stop_music();
        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))?.repeat_infinite();

        let sink = Sink::try_new(&self.handle)?;
        sink.set_volume(volume);
        sink.append(source);
        sink.play();

        self.music_sink = Some(sink);
        Ok(())
    }

    pub fn stop_music(&mut self) {
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }
    }

    pub fn play_sfx(&self, name: &str) {
        let data = match self.sfx_data.get(name) {
            Some(d) => d,
            None => return,
        };
        let cursor = Cursor::new(Arc::clone(data));
        if let (Ok(decoder), Ok(sink)) = (Decoder::new(BufReader::new(cursor)), Sink::try_new(&self.handle)) {
            sink.append(decoder);
            sink.detach();
        }
    }

    pub fn play_step(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_step_time) < self.step_interval {
            return;
        }
        self.last_step_time = now;
        if let Some(data) = self.sfx_data.get("step") {
            let cursor = Cursor::new(Arc::clone(data));
            if let (Ok(decoder), Ok(sink)) = (Decoder::new(BufReader::new(cursor)), Sink::try_new(&self.handle)) {
                sink.set_volume(0.5); // half volume
                sink.append(decoder);
                sink.detach();
            }
        }
    }
}