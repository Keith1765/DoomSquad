use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

const AUDIO_DISTANCE_SCALE_COEFFICIENT: f32 = 0.025;

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

    // Initialisiert Audio und lädt direkt alle Assets
    pub fn init() -> Option<Self> {
        let mut audio = Self::new().ok()?;

        let _ = audio.load_sfx("step", "assets/audio/step.wav");
        let _ = audio.load_sfx("jump", "assets/audio/jump.wav");
        let _ = audio.play_music_loop("assets/audio/music.wav", 0.6);

        Some(audio)
    }

    pub fn handle_input(&mut self, is_moving: bool, just_jumped: bool) {
        if is_moving {
            self.play_step();
        }

        if just_jumped {
            self.play_sfx("jump", 1.0);
        }
    }

    pub fn load_sfx<P: AsRef<Path>>(
        &mut self,
        name: &str,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.sfx_data
            .insert(name.to_string(), Arc::from(std::fs::read(path)?));
        Ok(())
    }

    pub fn play_music_loop<P: AsRef<Path>>(
        &mut self,
        path: P,
        volume: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn play_sfx(&self, name: &str, volume: f32) {
        let data = match self.sfx_data.get(name) {
            Some(d) => d,
            None => return,
        };
        let cursor = Cursor::new(Arc::clone(data));
        if let (Ok(decoder), Ok(sink)) = (
            Decoder::new(BufReader::new(cursor)),
            Sink::try_new(&self.handle),
        ) {
            sink.set_volume(volume);
            sink.append(decoder);
            sink.detach();
        }
    }

    pub fn play_sfx_distance_scaled(&self, name: &str, distance: f32) {
        let volume = (1.0 / (distance.max(0.01) * AUDIO_DISTANCE_SCALE_COEFFICIENT)).min(1.0);
        self.play_sfx(name, volume);
    }

    pub fn play_step(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_step_time) < self.step_interval {
            return;
        }
        self.last_step_time = now;
        if let Some(data) = self.sfx_data.get("step") {
            let cursor = Cursor::new(Arc::clone(data));
            if let (Ok(decoder), Ok(sink)) = (
                Decoder::new(BufReader::new(cursor)),
                Sink::try_new(&self.handle),
            ) {
                sink.set_volume(0.5); // half volume
                sink.append(decoder);
                sink.detach();
            }
        }
    }
}