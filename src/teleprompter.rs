use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontFamily {
    SansSerif,
    Serif,
    Monospace,
}

impl FontFamily {
    pub fn as_str(&self) -> &str {
        match self {
            FontFamily::SansSerif => "Sans Serif",
            FontFamily::Serif => "Serif",
            FontFamily::Monospace => "Monospace",
        }
    }
}

pub struct TeleprompterState {
    pub text: String,
    pub scroll_position: f32,
    pub speed: f32,
    pub paused: bool,
    pub transparency: f32,
    pub font_family: FontFamily,
    last_update: Instant,
    frame_count: u32,
}

impl TeleprompterState {
    pub fn new() -> Self {
        Self {
            text: include_str!("../sample_text.txt").to_string(),
            scroll_position: 0.0,
            speed: 50.0,
            paused: false,
            transparency: 0.95,
            font_family: FontFamily::SansSerif,
            last_update: Instant::now(),
            frame_count: 0,
        }
    }

    pub fn update(&mut self) {
        if !self.paused {
            let now = Instant::now();
            let delta = now.duration_since(self.last_update).as_secs_f32();
            
            // Cap delta to prevent large jumps (e.g., when window is moved)
            let capped_delta = delta.min(0.1);
            
            self.scroll_position += self.speed * capped_delta;
            self.last_update = now;
            self.frame_count += 1;
        } else {
            self.last_update = Instant::now();
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        self.last_update = Instant::now();
    }

    pub fn increase_speed(&mut self) {
        self.speed = (self.speed + 10.0).min(200.0);
    }

    pub fn decrease_speed(&mut self) {
        self.speed = (self.speed - 10.0).max(10.0);
    }

    pub fn toggle_transparency(&mut self) {
        self.transparency = if self.transparency > 0.5 { 0.3 } else { 0.95 };
    }

    pub fn reset(&mut self) {
        self.scroll_position = 0.0;
        self.paused = false;
    }
}
