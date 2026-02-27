use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::game::map::Point;

const AUDIO_DISTANCE_SCALE_COEFFICIENT: f32 = 0.025;
const AUDIO_ENABLED: bool = true;
const BACKGROUND_MUSIC_VOLUME: f32 = 0.2;

pub struct Audio {
    _stream: OutputStream, // must stay alive
    handle: OutputStreamHandle,
    music_sink: Option<Sink>,
    sfx_data: HashMap<String, Arc<[u8]>>,
    last_step_time: Instant,
    step_interval: Duration,
    pub is_muted: bool,
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
            is_muted: false, // Standardmäßig ist der Ton an
        })
    }

    // Initialisiert Audio und lädt direkt alle Assets
    pub fn init() -> Self {
        // ! accept unwrap; if audio totally broken, crashing is okay; should never fail anyway
        let mut audio = Self::new().ok().unwrap();
        if !AUDIO_ENABLED {
            return audio;
        } // if audio not enabled, return empty

        let _ = audio.load_sfx("arrow", "assets/soundeffects/arrow.wav");
        let _ = audio.load_sfx("button_press", "assets/soundeffects/button_press.wav");
        let _ = audio.load_sfx("enemy_shoot", "assets/soundeffects/enemy_shoot.wav");
        let _ = audio.load_sfx("explosion", "assets/soundeffects/explosion.wav");
        let _ = audio.load_sfx("heal", "assets/soundeffects/heal.wav");
        let _ = audio.load_sfx("jump_pad", "assets/soundeffects/jump_pad.wav");
        let _ = audio.load_sfx("jump", "assets/soundeffects/jump.wav");
        let _ = audio.load_sfx("hit", "assets/soundeffects/hit.wav");
        let _ = audio.load_sfx("monster_bite", "assets/soundeffects/monster_bite.wav");
        let _ = audio.load_sfx("rocketlauncher", "assets/soundeffects/rocketlauncher.wav");
        let _ = audio.load_sfx("shoot", "assets/soundeffects/shoot.wav");
        let _ = audio.load_sfx("slide", "assets/soundeffects/slide.wav");
        let _ = audio.load_sfx("slotmachine", "assets/soundeffects/slotmachine.wav");
        let _ = audio.load_sfx("spider_attack", "assets/soundeffects/spider_attack.wav");
        let _ = audio.load_sfx("step", "assets/soundeffects/step.wav");
        let _ = audio.load_sfx("summoner", "assets/soundeffects/summoner.wav");
        let _ = audio.play_music_loop("assets/music/doom_theme.wav", BACKGROUND_MUSIC_VOLUME);

        audio
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.is_muted = muted;
        if let Some(sink) = &self.music_sink {
            if muted {
                sink.pause(); // Pausiert die Hintergrundmusik wenn Audio muted
            } else {
                sink.play(); // Setzt Hintergrundmusik fort
            }
        }
    }

    pub fn handle_input(&mut self, is_moving: bool, just_jumped: bool) {
        // if is_moving {
        //     self.play_step(1.0);
        // }

        // if just_jumped {
        //     self.play_sfx("jump", 1.0);
        // }
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

        // Direkt pausieren, falls das Spiel beim Laden schon stummgeschaltet ist
        if self.is_muted {
            sink.pause();
        } else {
            sink.play();
        }

        self.music_sink = Some(sink);
        Ok(())
    }

    pub fn stop_music(&mut self) {
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }
    }

    pub fn play_sfx(&mut self, name: &str, volume: f32) {
        if self.is_muted {
            return;
        } // keine Audio wenn im Main Menu stummgeschaltet

        if let Some(data) = self.sfx_data.get(name) {
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
    }

    pub fn play_sfx_distance_scaled(
        &mut self,
        name: &str,
        orignial_volume: f32,
        player_position: Point,
        other_position: Point,
    ) {
        if self.is_muted {
            return;
        } // keine Audio wenn im Main Menu stummgeschaltet

        let distance = player_position.distance_to(&other_position) as f32;
        let volume =
            (orignial_volume / (distance.max(0.01) * AUDIO_DISTANCE_SCALE_COEFFICIENT)).min(1.0);
        self.play_sfx(name, volume);
    }

    pub fn play_step(&mut self, volume: f32) {
        if self.is_muted {
            return;
        } // keine Audio wenn im Main Menu stummgeschaltet

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
                sink.set_volume(volume);
                sink.append(decoder);
                sink.detach();
            }
        }
    }

    pub fn play_step_distance_scaled(&mut self, player_position: Point, other_position: Point) {
        if self.is_muted {
            return;
        } // keine Audio wenn im Main Menu stummgeschaltet

        let distance = player_position.distance_to(&other_position) as f32;
        let volume = (1.0 / (distance.max(0.01) * AUDIO_DISTANCE_SCALE_COEFFICIENT)).min(1.0);
        self.play_step(volume);
    }
}
