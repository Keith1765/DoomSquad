//! The Menu and User Interface system.
//!
//! This module manages all the non-gameplay screens, such as the Main Menu,
//! the Pause Screen, and the Game Over screen. It handles drawing buttons and text,
//! processing mouse clicks, and taking care of saving and loading the player's progress.

use font8x8::{BASIC_FONTS, UnicodeFonts};
use minifb::Key::Escape;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};
use std::collections::{BinaryHeap, HashMap};
use std::fs;

use crate::audio::audio_handler::Audio;
use crate::game::gamestate::Game;
use crate::render::RendererData;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 450;

const COL_BG_MAIN: u32 = 0x111111;
const COL_BG_GAMEOVER: u32 = 0x330000;
const COL_BG_PAUSED: u32 = 0x002244;
const COL_TITLE_RED: u32 = 0xCC0000;
const COL_TITLE_BLUE: u32 = 0x44AAFF;
const COL_BTN_NORMAL: u32 = 0x444444;
const COL_BTN_HOVER: u32 = 0xAA0000;
const COL_TEXT_WHITE: u32 = 0xFFFFFF;

/// Dictates the high-level routing in the `main.rs` loop.
pub enum AppState {
    StartScreen,
    Playing,
    GameOver,
    GamePaused,
    Quit,
    ConfigKeys,
}

/// Simple layout container for button definitions.
struct ButtonDef {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    text: String,
}

#[derive(Clone,Copy,Debug,Eq, PartialEq, Hash)]
pub enum Actions {
	MvForward, 
	MvBackward,
	MvLeft,
	MvRight,
	Jump,
	RocketJump,
	Sprint,
	Crouch,
	Interact,
	MvCrosshairDown,
	MvCrosshairUp,
	TurnRight,
	TurnLeft,
	FireBullet,
	FireArrow,
	ToggleGodmode,
	ToggleFineAim,
	LockActiveEnemies,
	Exit
}

impl Actions {
    pub fn actions_display_name(&self) ->&'static str {
        match self {
            Actions::MvForward => "MOVE FORWARD",
			Actions::MvBackward => "MOVE BACKWARD",
			Actions::MvLeft => "MOVE LEFT",
			Actions::MvRight => "MOVE RIGHT",
			Actions::Jump => "JUMP",
			Actions::RocketJump => "ROCKET JUMP",
			Actions::Sprint => "SPRINT",
			Actions::Crouch => "CROUCH",
			Actions::Interact => "INTERACT",
			Actions::MvCrosshairDown => "AIM DOWN",
			Actions::MvCrosshairUp => "AIM UP",
			Actions::TurnRight => "TURN RIGHT",
			Actions::TurnLeft => "TURN LEFT",
			Actions::FireBullet => "SHOOT BULLET",
			Actions::FireArrow => "SHOOT ARROW",
			Actions::ToggleGodmode => "GODMODE",
			Actions::ToggleFineAim => "FINE AIM",
			Actions::LockActiveEnemies => "LOCK ENEMIES",
			Actions::Exit => "EXIT",
        }
    }
}



/// Holds the minimal state required for the IMGUI.
/// Since `minifb` natively handles keyboard debouncing via `KeyRepeat::No`,
/// we only need to manually track the mouse state for edge detection (clicks).
pub struct Menu {
    was_mouse_down: bool,
    pub active_binding: Option<Actions>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            was_mouse_down: false,
            active_binding: None,
        }
    }

    /// Renders the main menu and handles user interactions.
    pub fn update_and_draw_start_menu(  //called from main when AppState::StartScreen
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        renderer_data: &RendererData,
        audio: &mut Audio,
    ) -> AppState {
        // Clear the buffer from the previous frame
        buffer.fill(COL_BG_MAIN);

        // Fetch user inputs
        let (mx, my, clicked) = self.get_mouse_state(window);
        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);
        let click2 = window.is_key_pressed(Key::Key2, KeyRepeat::No);
        let click3 = window.is_key_pressed(Key::Key3, KeyRepeat::No);
        let click4 = window.is_key_pressed(Key::Key4, KeyRepeat::No);

        // Render titles
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "DOOMSQUAD",
            20,
            6,
            COL_TITLE_RED,
        );
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "MAIN MENU",
            75,
            2,
            COL_TEXT_WHITE,
        );

        //
        //
        // Define button layouts
        //
        //
        let btn_start = ButtonDef {
            x: 210,
            y: 105, //change!!!
            w: 370, 
            h: 50,
            text: "[1] NEW GAME".to_string(),
        };

        let btn_load = ButtonDef {
            x: 210,
            y: 175, //change!!!
            w: 370, 
            h: 50,
            text: "[2] LOAD MAP".to_string(),
        };

        let audio_text = if !audio.is_muted {
            "[3] SOUND: ON"
        } else {
            "[3] SOUND: OFF"
        };

        let btn_audio = ButtonDef {
            x: 210,
            y: 245, //change!!!
            w: 370,
            h: 50,
            text: audio_text.to_string(),
        };

        let btn_quit = ButtonDef {
            x: 210,
            y: 385,  //change!!!
            w: 370,  
            h: 50,
            text: "[ESC] QUIT GAME".to_string(),
        };

        //TO DO: change//assign correct values for 
        let btn_keys = ButtonDef {
            x : 210,
            y : 315,  //change!!!
            w : 370,  
            h : 50 ,
            text: "[4] CHANGE KEYBINDS".to_string(),
        };

        // Interaction processing --> ggf. change AppState to Playing
        if self.draw_flat_button(buffer, &btn_start, mx, my) && clicked || click1 {
            let saved_keybinds = game.keybinds.clone();
            *game = Game::new_game(renderer_data).unwrap(); // ! this unwrap is intentionally accepted
            game.keybinds = saved_keybinds;
            game.map_swap(renderer_data, 1); // Jump straight to map 1 (map 0 is reserved for testing)
            return AppState::Playing;
        }

        #[allow(clippy::collapsible_if)]
        if self.draw_flat_button(buffer, &btn_load, mx, my) && clicked || click2 {
            // Read saved map index from disk and safely parse it
            if let Ok(content) = fs::read_to_string("savegame.txt")
                && let Ok(index) = content.trim().parse::<usize>()
            {
                let saved_keybinds = game.keybinds.clone();
                *game = Game::new_game(renderer_data).unwrap(); // ! this unwrap is intentionally accepted
                game.keybinds= saved_keybinds;
                game.map_swap(renderer_data, index);
                return AppState::Playing;
            }
        }

        if self.draw_flat_button(buffer, &btn_audio, mx, my) && clicked || click3 {
            audio.set_muted(!audio.is_muted);
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_quit, mx, my) && clicked {
            return AppState::Quit;
        }

        //TO DO: Add interaction logic when btn_keys is pressed
        if self.draw_flat_button(buffer, &btn_keys, mx, my) &&clicked || click4 {
            //TO DO: Add interaction logic when btn_keys is pressed


            return AppState::ConfigKeys;   // Maybe also add main() logic to handle new menu
        }


        // Keep looping in the menu until an action changes the state
        AppState::StartScreen
    }

     
    pub fn update_and_draw_key_config_menu(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
		
    ) -> AppState { 
        
        buffer.fill(COL_BG_MAIN);

        let (mx, my, clicked) = self.get_mouse_state(window);
        let click_esc = window.is_key_pressed(Key::Escape, KeyRepeat::No);

        let was_binding = self.active_binding.is_some();
		

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "CONFIGURE KEY BINDINGS",
            15,
            3,
            COL_TEXT_WHITE,
        );

        // TO DO: Add btns 4 new key binds
		let actions: [Actions;19] = [
			Actions::MvForward,
			Actions::MvBackward,
			Actions::MvLeft,
			Actions::MvRight,
			Actions::Jump,
			Actions::RocketJump,
			Actions::Sprint,
			Actions::Crouch,
			Actions::Interact,
			Actions::MvCrosshairDown,
			Actions::MvCrosshairUp,
			Actions::TurnRight,
			Actions::TurnLeft,
			Actions::FireBullet,
			Actions::FireArrow,
			Actions::ToggleGodmode,
			Actions::ToggleFineAim,
			Actions::LockActiveEnemies,
			Actions::Exit
		];

		if let Some(target_action) = self.active_binding {
            Self::draw_text_centered(
                buffer, 
                SCREEN_WIDTH,
                SCREEN_HEIGHT, 
                &format!("PRESS A KEY FOR \"{}\"", target_action.actions_display_name()), 
                48, 
                2, 
                COL_TITLE_BLUE
            );
        
            //check if user pressed key
            if let Some(new_key) = window.get_keys_pressed(KeyRepeat::No).first() {
                game.keybinds.insert(target_action, *new_key);//overwrite old key in HashMap

                //Save binding in config txt
                let mut content = String::new();

                for (action, key) in &game.keybinds{
                    content.push_str(&format!("{:?},{:?}\n", action, key));
                }

                let _ =fs::write("keybinds.txt", content);

                self.active_binding = None;
    
            
            }
        }

        let w = 230;
        let h = 36;
	
		for (i,action) in actions.iter().enumerate() {
            //starting coordinates & actions list --> used in for loop
            let col= i % 3;
            let row = i/3;
            
		    let y = 75 + row *42;
		    let x= 25 + col * 260;
		
			let current_key = game.keybinds.get(action).copied().unwrap_or(Key::Unknown);

			let text = if self.active_binding== Some(*action) {
                format!("{}: [???]", action.actions_display_name())
            } else {
                format!("{}: [{:?}]", action.actions_display_name(), current_key)

            };

            let btn_current_key = ButtonDef {x, y, w, h, text};
			if self.draw_flat_button_scaled(buffer, &btn_current_key, mx, my, 1) && clicked {
                self.active_binding = Some(*action);
	
			}
   
        }
	

        //back btn interaction
        let btn_back = ButtonDef {
            x : 285,
            y : 375,  //change!!!
            w : 230,  
            h : 40 ,
            text: "<- CLICK TO GO BACK".to_string(),
        };

        //back btn interaction
        if self.draw_flat_button_scaled(buffer, &btn_back, mx, my, 1) && clicked 
        || (click_esc && !was_binding) 
        { //check if esc returns
            self.active_binding = None;
            return AppState::StartScreen;
        }

        // Keep looping in the key binding menu until an action changes the state
        AppState::ConfigKeys

 	}       

        





    /// Renders the death screen and allows the user to save their current level progress.
    pub fn update_and_draw_game_over(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
    ) -> AppState {
        buffer.fill(COL_BG_GAMEOVER);

        let (mx, my, clicked) = self.get_mouse_state(window);
        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "YOU DIED",
            80,
            6,
            COL_TITLE_RED,
        );

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

        if self.draw_flat_button(buffer, &btn_save, mx, my) && clicked || click1 {
            // Persist the current level index to disk so it can be loaded later
            let _ = fs::write("savegame.txt", game.map_index.to_string());
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_menu, mx, my) && clicked {
            return AppState::StartScreen;
        }

        AppState::GameOver
    }

    /// Renders the pause menu (triggered by pressing ESC during gameplay). 
    pub fn update_and_draw_game_paused( //--->>>Change to new key bind
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        audio: &mut Audio,
    ) -> AppState {
        buffer.fill(COL_BG_PAUSED);

        let (mx, my, clicked) = self.get_mouse_state(window);

        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);
        let click2 = window.is_key_pressed(Key::Key2, KeyRepeat::No);
        let click3 = window.is_key_pressed(Key::Key3, KeyRepeat::No);

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "GAME PAUSED",
            60,
            6,
            COL_TITLE_BLUE,
        );

        let audio_text = if !audio.is_muted {
            "[3] SOUND: ON"
        } else {
            "[3] SOUND: OFF"
        };

        let btn_resume = ButtonDef {
            x: 230,
            y: 150,
            w: 340,
            h: 50,
            text: "[1] RESUME GAME".to_string(),
        };
        let btn_save = ButtonDef {
            x: 230,
            y: 210,
            w: 340,
            h: 50,
            text: "[2] SAVE & EXIT".to_string(),
        };
        let btn_audio = ButtonDef {
            x: 230,
            y: 270,
            w: 340,
            h: 50,
            text: audio_text.to_string(),
        };
        let btn_menu = ButtonDef {
            x: 230,
            y: 330,
            w: 340,
            h: 50,
            text: "[ESC] MAIN MENU".to_string(),
        };

        if self.draw_flat_button(buffer, &btn_resume, mx, my) && clicked || click1 {
            // Simply return to the Playing state without touching the game object to resume seamlessly
            return AppState::Playing;
        }

        if self.draw_flat_button(buffer, &btn_save, mx, my) && clicked || click2 {
            let _ = std::fs::write("savegame.txt", game.map_index.to_string());
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_audio, mx, my) && clicked || click3 {
            audio.set_muted(!audio.is_muted);
            return AppState::GamePaused;
        }

        if self.draw_flat_button(buffer, &btn_menu, mx, my) && clicked {
            return AppState::StartScreen;
        }

        AppState::GamePaused
    }

    /// Fetches mouse coordinates and performs edge detection (debouncing) for left-clicks.
    fn get_mouse_state(&mut self, window: &Window) -> (usize, usize, bool) {
        // Clamp prevents out-of-bounds coordinates if the mouse leaves the window frame
        let mouse_pos = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let is_mouse_down = window.get_mouse_down(MouseButton::Left);

        let clicked = is_mouse_down && !self.was_mouse_down;
        self.was_mouse_down = is_mouse_down;

        (mouse_pos.0 as usize, mouse_pos.1 as usize, clicked)
    }

    /// Draws a rectangular button and its text onto the buffer.
    /// Returns `true` if the mouse cursor is currently hovering over it.
    fn draw_flat_button(&self, buffer: &mut [u32], btn: &ButtonDef, mx: usize, my: usize) -> bool {
        // Simple check for hover detection
        let hover = mx >= btn.x && mx <= (btn.x + btn.w) && my >= btn.y && my <= (btn.y + btn.h);
        let bg_color = if hover { COL_BTN_HOVER } else { COL_BTN_NORMAL };

        // Rasterize the rectangle into the 1D pixel buffer
        for row in btn.y..(btn.y + btn.h) {
            for col in btn.x..(btn.x + btn.w) {
                // Safety check to prevent panic from out-of-bounds memory writes
                if row < SCREEN_HEIGHT && col < SCREEN_WIDTH {
                    buffer[row * SCREEN_WIDTH + col] = bg_color;
                }
            }
        }

        // Center the text horizontally and vertically inside the button bounds
        let text_width = btn.text.len() * 8 * 2;
        let text_x = btn.x + btn.w.saturating_sub(text_width) / 2;
        let text_y = btn.y + btn.h.saturating_sub(16) / 2;

        Self::draw_text(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &btn.text,
            text_x,
            text_y,
            2,
            COL_TEXT_WHITE,
        );

        hover
    }

    fn draw_flat_button_scaled(
        &self,
        buffer: &mut [u32], 
        btn : &ButtonDef, 
        mx: usize, 
        my: usize, 
        scale: usize
    ) -> bool {
        let hover = mx >= btn.x && mx <= (btn.x + btn.w) && my >= btn.y && my <= (btn.y + btn.h);
        let bg_color = if hover { COL_BTN_HOVER } else { COL_BTN_NORMAL };

        for row in btn.y..(btn.y + btn.h) {
            for col in btn.x..(btn.x + btn.w) {
                if row < SCREEN_HEIGHT && col < SCREEN_WIDTH {
                    buffer[row * SCREEN_WIDTH + col] = bg_color;
                }
            }
        }

        let text_width = btn.text.len() * 8 * scale;
        let text_height = 8 * scale;
        let text_x = btn.x + btn.w.saturating_sub(text_width) / 2;
        let text_y = btn.y + btn.h.saturating_sub(text_height) / 2;

        Self::draw_text(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &btn.text,
            text_x,
            text_y,
            scale,
            COL_TEXT_WHITE,
        );

        hover

    }

    /// Helper to align a given text string exactly in the horizontal center of the screen.
    fn draw_text_centered(
        buffer: &mut [u32],
        screen_width: usize,
        screen_height: usize,
        text: &str,
        y: usize,
        scale: usize,
        color: u32,
    ) {
        // Each character in the font is 8 pixels wide, multiplied by the scaling factor
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

    /// Software text rasterizer using the `font8x8` library.
    /// It translates strings into raw pixel data and pushes them to the 1D buffer.
    #[allow(clippy::too_many_arguments)]
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
                #[allow(clippy::needless_range_loop)]
                for y in 0..8 {
                    for x in 0..8 {
                        // Bitwise check: if the specific bit in the glyph byte is a 1, a pixel needs to be drawn
                        if (glyph[y] & (1 << x)) != 0 {
                            // Nearest-neighbor scaling (draws larger 2D blocks for each active pixel based on the `scale` factor)
                            for dy in 0..scale {
                                for dx in 0..scale {
                                    let px = cursor_x + x * scale + dx;
                                    let py = start_y + y * scale + dy;

                                    if px < screen_width && py < screen_height {
                                        // Flatten 2D coordinate into a 1D array index
                                        buffer[py * screen_width + px] = color;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Advance the cursor by the width of the character to prepare for the next letter
            cursor_x += 8 * scale;
        }
    }

    

}

pub fn parse_action(s: &str) -> Option<Actions> {
    match s {
        "MvForward" => Some(Actions::MvForward),
        "MvBackward" => Some(Actions::MvBackward),
        "MvLeft" => Some(Actions::MvLeft),
        "MvRight" => Some(Actions::MvRight),
        "Jump" => Some(Actions::Jump),
        "RocketJump" => Some(Actions::RocketJump),
        "Sprint" => Some(Actions::Sprint),
        "Crouch" => Some(Actions::Crouch),
        "Interact" => Some(Actions::Interact),
        "Sprint" => Some(Actions::Sprint),
        "MvCrosshairDown" => Some(Actions::MvCrosshairDown),
        "MvCrosshairUp" => Some(Actions::MvCrosshairUp),
        "TurnRight" => Some(Actions::TurnRight),
        "TurnLeft" => Some(Actions::TurnLeft),
        "FireBullet" => Some(Actions::FireBullet),
        "FireArrow" => Some(Actions::FireArrow),
        "ToggleGodmode" => Some(Actions::ToggleGodmode),
        "ToggleFineAim" => Some(Actions::ToggleFineAim),
        "LockActiveEnemies" => Some(Actions::LockActiveEnemies),
        "Exit" => Some(Actions::Exit),
        _ => None,
    }
}

pub fn parse_key(s: &str) -> Option<Key> {
    match s {
        // Alphabet
    "A" => Some(Key::A), "B" => Some(Key::B), "C" => Some(Key::C),
    "D" => Some(Key::D), "E" => Some(Key::E), "F" => Some(Key::F),
    "G" => Some(Key::G), "H" => Some(Key::H), "I" => Some(Key::I),
    "J" => Some(Key::J), "K" => Some(Key::K), "L" => Some(Key::L),
    "M" => Some(Key::M), "N" => Some(Key::N), "O" => Some(Key::O),
    "P" => Some(Key::P), "Q" => Some(Key::Q), "R" => Some(Key::R),
    "S" => Some(Key::S), "T" => Some(Key::T), "U" => Some(Key::U),
    "V" => Some(Key::V), "W" => Some(Key::W), "X" => Some(Key::X),
    "Y" => Some(Key::Y), "Z" => Some(Key::Z),
        
    // Numbers
    "Key0" => Some(Key::Key0), "Key1" => Some(Key::Key1), 
    "Key2" => Some(Key::Key2), "Key3" => Some(Key::Key3), 
    "Key4" => Some(Key::Key4), "Key5" => Some(Key::Key5), 
    "Key6" => Some(Key::Key6), "Key7" => Some(Key::Key7), 
    "Key8" => Some(Key::Key8), "Key9" => Some(Key::Key9),
        
    // Modifiers & Specials
    "Space" => Some(Key::Space), "Escape" => Some(Key::Escape),
    "Up" => Some(Key::Up), "Down" => Some(Key::Down),
    "Left" => Some(Key::Left), "Right" => Some(Key::Right),
    "RightCtrl" => Some(Key::RightCtrl), "LeftCtrl" => Some(Key::LeftCtrl),
    "RightShift" => Some(Key::RightShift), "LeftShift" => Some(Key::LeftShift),
    "Enter" => Some(Key::Enter), "Tab" => Some(Key::Tab),
    _ => None,
    }
}