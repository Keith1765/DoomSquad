use font8x8::{BASIC_FONTS, UnicodeFonts};
use minifb::{Key, MouseButton, MouseMode, Window};
use std::fs;

use crate::game::gamestate::Game;
use crate::render::RendererData;

const COL_BG_MAIN: u32 = 0x111111;
const COL_BG_GAMEOVER: u32 = 0x330000;
const COL_TITLE_RED: u32 = 0xCC0000;
const COL_BTN_NORMAL: u32 = 0x444444;
const COL_BTN_HOVER: u32 = 0xAA0000;
const COL_TEXT_WHITE: u32 = 0xFFFFFF;

pub enum AppState {
    StartScreen,
    Playing,
    GameOver,
    Quit,
}

struct ButtonDef {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    text: String,
}

pub struct Menu {
    was_mouse_down: bool,
    was_key1_down: bool,
    was_key2_down: bool,
    was_key3_down: bool,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            was_mouse_down: false,
            was_key1_down: false,
            was_key2_down: false,
            was_key3_down: false,
        }
    }

    pub fn update_and_draw(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        renderer_data: &RendererData,
        is_audio_enabled: &mut bool,
    ) -> AppState {
        buffer.fill(COL_BG_MAIN);

        let (mx, my, clicked) = self.get_mouse_state(window);
        let k1 = window.is_key_down(Key::Key1);
        let k2 = window.is_key_down(Key::Key2);
        let k3 = window.is_key_down(Key::Key3);

        let click1 = k1 && !self.was_key1_down;
        let click2 = k2 && !self.was_key2_down;
        let click3 = k3 && !self.was_key3_down;

        self.was_key1_down = k1;
        self.was_key2_down = k2;
        self.was_key3_down = k3;

        Self::draw_text_centered(buffer, 800, 450, "DOOMSQUAD", 50, 6, COL_TITLE_RED);
        Self::draw_text_centered(buffer, 800, 450, "MAIN MENU", 105, 2, COL_TEXT_WHITE);

        let btn_start = ButtonDef {
            x: 230,
            y: 160,
            w: 340,
            h: 50,
            text: "[1] NEW GAME".to_string(),
        };
        let btn_load = ButtonDef {
            x: 230,
            y: 230,
            w: 340,
            h: 50,
            text: "[2] LOAD MAP".to_string(),
        };
        let audio_text = if *is_audio_enabled {
            "[3] SOUND: ON"
        } else {
            "[3] SOUND: OFF"
        };
        let btn_audio = ButtonDef {
            x: 230,
            y: 300,
            w: 340,
            h: 50,
            text: audio_text.to_string(),
        };
        let btn_quit = ButtonDef {
            x: 230,
            y: 370,
            w: 340,
            h: 50,
            text: "[ESC] QUIT TO DESKTOP".to_string(),
        };

        // BUTTON new game
        if self.draw_flat_button(buffer, 800, 450, &btn_start, mx, my) && clicked || click1 {
            *game = Game::new_game(renderer_data).unwrap();
            return AppState::Playing;
        }

        // BUTTON load game
        if self.draw_flat_button(buffer, 800, 450, &btn_load, mx, my) && clicked || click2 {
            if let Ok(content) = fs::read_to_string("savegame.txt") {
                if let Ok(index) = content.trim().parse::<usize>() {
                    *game = Game::new_game(renderer_data).unwrap();
                    game.map_swap(renderer_data, index);
                    println!("Map {} erfolgreich geladen!", index);
                    return AppState::Playing;
                }
            } else {
                println!("Kein Spielstand gefunden!");
            }
        }

        // BUTTON audio
        if self.draw_flat_button(buffer, 800, 450, &btn_audio, mx, my) && clicked || click3 {
            *is_audio_enabled = !*is_audio_enabled;
            return AppState::StartScreen;
        }

        // BUTTON quit
        if self.draw_flat_button(buffer, 800, 450, &btn_quit, mx, my) && clicked {
            return AppState::Quit;
        }

        AppState::StartScreen
    }

    pub fn update_and_draw_game_over(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
    ) -> AppState {
        buffer.fill(COL_BG_GAMEOVER);

        let (mx, my, clicked) = self.get_mouse_state(window);
        let k1 = window.is_key_down(Key::Key1);
        let click1 = k1 && !self.was_key1_down;
        self.was_key1_down = k1;

        Self::draw_text_centered(buffer, 800, 450, "YOU DIED", 80, 6, COL_TITLE_RED);

        let btn_save = ButtonDef {
            x: 230,
            y: 220,
            w: 340,
            h: 60,
            text: "[1] SAVE MAP INDEX".to_string(),
        };
        let btn_menu = ButtonDef {
            x: 230,
            y: 300,
            w: 340,
            h: 60,
            text: "[ESC] MAIN MENU".to_string(),
        };

        // BUTTON save map index
        if self.draw_flat_button(buffer, 800, 450, &btn_save, mx, my) && clicked || click1 {
            let _ = fs::write("savegame.txt", game.map_index.to_string());
            println!("Map-Index {} gespeichert!", game.map_index);
            return AppState::StartScreen;
        }

        // BUTTON back to main menu
        if self.draw_flat_button(buffer, 800, 450, &btn_menu, mx, my) && clicked {
            return AppState::StartScreen;
        }

        AppState::GameOver
    }

    fn get_mouse_state(&mut self, window: &Window) -> (usize, usize, bool) {
        let mouse_pos = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let is_mouse_down = window.get_mouse_down(MouseButton::Left);
        let clicked = is_mouse_down && !self.was_mouse_down;
        self.was_mouse_down = is_mouse_down;
        (mouse_pos.0 as usize, mouse_pos.1 as usize, clicked)
    }

    fn draw_flat_button(
        &self,
        buffer: &mut [u32],
        screen_width: usize,
        screen_height: usize,
        btn: &ButtonDef,
        mx: usize,
        my: usize,
    ) -> bool {
        let hover = mx >= btn.x && mx <= (btn.x + btn.w) && my >= btn.y && my <= (btn.y + btn.h);
        let bg_color = if hover { COL_BTN_HOVER } else { COL_BTN_NORMAL };

        for row in btn.y..(btn.y + btn.h) {
            for col in btn.x..(btn.x + btn.w) {
                if row < screen_height && col < screen_width {
                    buffer[row * screen_width + col] = bg_color;
                }
            }
        }

        let text_width = btn.text.len() * 8 * 2;
        let text_x = btn.x + btn.w.saturating_sub(text_width) / 2;
        let text_y = btn.y + btn.h.saturating_sub(16) / 2;
        Self::draw_text(
            buffer,
            screen_width,
            screen_height,
            &btn.text,
            text_x,
            text_y,
            2,
            COL_TEXT_WHITE,
        );

        hover
    }

    fn draw_text_centered(
        buffer: &mut [u32],
        screen_width: usize,
        screen_height: usize,
        text: &str,
        y: usize,
        scale: usize,
        color: u32,
    ) {
        let text_width = text.len() * 8 * scale;
        let x = screen_width.saturating_sub(text_width) / 2;
        Self::draw_text(
            buffer,
            screen_width,
            screen_height,
            text,
            x,
            y,
            scale,
            color,
        );
    }

    fn draw_text(
        buffer: &mut [u32],
        screen_width: usize,
        screen_height: usize,
        text: &str,
        start_x: usize,
        start_y: usize,
        scale: usize,
        color: u32,
    ) {
        let mut cursor_x = start_x;
        for c in text.chars() {
            if let Some(glyph) = BASIC_FONTS.get(c) {
                for y in 0..8 {
                    for x in 0..8 {
                        if (glyph[y] & (1 << x)) != 0 {
                            for dy in 0..scale {
                                for dx in 0..scale {
                                    let px = cursor_x + x * scale + dx;
                                    let py = start_y + y * scale + dy;
                                    if px < screen_width && py < screen_height {
                                        buffer[py * screen_width + px] = color;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            cursor_x += 8 * scale;
        }
    }
}
