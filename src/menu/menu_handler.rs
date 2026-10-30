//! The Menu and User Interface system.
//!
//! This module manages all the non-gameplay screens, such as the Main Menu,
//! the Pause Screen, and the Game Over screen. It handles drawing buttons and text,
//! processing mouse clicks, and taking care of saving and loading the player's progress.

use font8x8::{BASIC_FONTS, UnicodeFonts};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};
use std::fs;

use crate::audio::audio_handler::Audio;
use crate::game::gamestate::Game;
use crate::render::RendererData;

const KEYBIND_FILE: &str = "keybinds.txt";

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
    KeyBindings,
    Quit,
}

/// Simple layout container for button definitions.
struct ButtonDef {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    text: String,
}

/// Holds the minimal state required for the IMGUI.
/// Since `minifb` natively handles keyboard debouncing via `KeyRepeat::No`,
/// we only need to manually track the mouse state for edge detection (clicks).
pub struct Menu {
    was_mouse_down: bool,
    rebinding_key: Option<usize>,
}

pub struct KeyBinds {
    pub forewards: Key,
    pub backwards: Key,
    pub sprint: Key,
    pub crouch: Key,
    pub interact: Key,
    pub crosshair_down: Key,
    pub turn_right: Key,
    pub fire_bullet: Key,
    pub godmode: Key,
    pub left: Key,
    pub right: Key,
    pub jump: Key,
    pub rocket_jump: Key,
    pub crosshair_up: Key,
    pub turn_left: Key,
    pub fine_aiming: Key,
    pub fire_arrow: Key,
    pub lock_enemies: Key,
}

impl KeyBinds {
    pub fn new() -> Self {
        Self {
            forewards: Key::W,
            backwards: Key::S,
            sprint: Key::LeftShift,
            crouch: Key::C,
            interact: Key::F,
            crosshair_down: Key::Down,
            turn_right: Key::Right,
            fire_bullet: Key::RightCtrl,
            godmode: Key::G,
            left: Key::A,
            right: Key::D,
            jump: Key::Space,
            rocket_jump: Key::R,
            crosshair_up: Key::Up,
            turn_left: Key::Left,
            fine_aiming: Key::E,
            fire_arrow: Key::RightShift,
            lock_enemies: Key::L,
        }
    }

    // Start from the defaults, then override with whatever is in the file.
    pub fn load() -> Self {
        let mut binds = Self::new();

        if let Ok(content) = fs::read_to_string(KEYBIND_FILE) {
            for line in content.lines() {
                if let Some((name, key_name)) = line.split_once('=')
                    && let Some(key) = key_from_name(key_name.trim())
                {
                    match name.trim() {
                        "forewards" => binds.forewards = key,
                        "backwards" => binds.backwards = key,
                        "sprint" => binds.sprint = key,
                        "crouch" => binds.crouch = key,
                        "interact" => binds.interact = key,
                        "crosshair_down" => binds.crosshair_down = key,
                        "turn_right" => binds.turn_right = key,
                        "fire_bullet" => binds.fire_bullet = key,
                        "godmode" => binds.godmode = key,
                        "left" => binds.left = key,
                        "right" => binds.right = key,
                        "jump" => binds.jump = key,
                        "rocket_jump" => binds.rocket_jump = key,
                        "crosshair_up" => binds.crosshair_up = key,
                        "turn_left" => binds.turn_left = key,
                        "fine_aiming" => binds.fine_aiming = key,
                        "fire_arrow" => binds.fire_arrow = key,
                        "lock" => binds.lock_enemies = key,
                        _ => {}
                    }
                }
            }
        }
        binds
    }

    pub fn save(&self) {
        let text = format!(
            "forewards={:?}\nbackwards={:?}\nsprint={:?}\ncrouch={:?}\ninteract={:?}\ncrosshair_down={:?}\nturn_right={:?}\nfire_bullet={:?}\ngodmode={:?}\nleft={:?}\nright={:?}\njump={:?}\nrocket_jump={:?}\ncrosshair_up={:?}\nturn_left={:?}\nfine_aiming={:?}\nfire_arrow={:?}\nlock={:?}\n",
            self.forewards, self.backwards, self.sprint, self.crouch, self.interact, self.crosshair_down, self.turn_right, self.fire_bullet, self.godmode, self.left, self.right, self.jump, self.rocket_jump, self.crosshair_up, self.turn_left, self.fine_aiming, self.fire_arrow, self.lock_enemies
        );
        let _ = fs::write(KEYBIND_FILE, text);
    }
}

impl Menu {
    pub fn new() -> Self {
        Self {
            was_mouse_down: false,
            rebinding_key: None,
        }
    }

    /// Renders the main menu and handles user interactions.
    pub fn update_and_draw_start_menu(
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
            50,
            6,
            COL_TITLE_RED,
        );
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "MAIN MENU",
            105,
            2,
            COL_TEXT_WHITE,
        );

        // Define button layouts
        let btn_start = ButtonDef {
            x: 230,
            y: 160,
            w: 340,
            h: 40,
            text: "[1] NEW GAME".to_string(),
        };
        let btn_load = ButtonDef {
            x: 230,
            y: 220,
            w: 340,
            h: 40,
            text: "[2] LOAD MAP".to_string(),
        };

        let audio_text = if !audio.is_muted {
            "[3] SOUND: ON"
        } else {
            "[3] SOUND: OFF"
        };
        let btn_audio = ButtonDef {
            x: 230,
            y: 280,
            w: 340,
            h: 40,
            text: audio_text.to_string(),
        };

        let btn_kbind = ButtonDef {
            x: 230,
            y: 340,
            w: 340,
            h: 40,
            text: "[4] KEY BINDINGS".to_string(),
        };

        let btn_quit = ButtonDef {
            x: 230,
            y: 400,
            w: 340,
            h: 40,
            text: "[ESC] QUIT GAME".to_string(),
        };

        // Interaction processing
        if self.draw_flat_button(buffer, &btn_start, mx, my) && clicked || click1 {
            *game = Game::new_game(renderer_data).unwrap(); // ! this unwrap is intentionally accepted
            game.map_swap(renderer_data, 1); // Jump straight to map 1 (map 0 is reserved for testing)
            return AppState::Playing;
        }

        #[allow(clippy::collapsible_if)]
        if self.draw_flat_button(buffer, &btn_load, mx, my) && clicked || click2 {
            // Read saved map index from disk and safely parse it
            if let Ok(content) = fs::read_to_string("savegame.txt")
                && let Ok(index) = content.trim().parse::<usize>()
            {
                *game = Game::new_game(renderer_data).unwrap(); // ! this unwrap is intentionally accepted
                game.map_swap(renderer_data, index);
                return AppState::Playing;
            }
        }

        if self.draw_flat_button(buffer, &btn_audio, mx, my) && clicked || click3 {
            audio.set_muted(!audio.is_muted);
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_kbind, mx, my) && clicked || click4 {
            return AppState::KeyBindings;
        }

        if self.draw_flat_button(buffer, &btn_quit, mx, my) && clicked {
            return AppState::Quit;
        }

        // Keep looping in the menu until an action changes the state
        AppState::StartScreen
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
    pub fn update_and_draw_game_paused(
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

    /// Renders the key bindings menu and handles user interactions.
    pub fn update_and_draw_key_bindings(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        key_binds: &mut KeyBinds,
    ) -> AppState {
        buffer.fill(COL_BG_MAIN);

        // Fetch user inputs
        let (mx, my, clicked) = self.get_mouse_state(window);

        // Render titles
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "KEY BINDINGS",
            60,
            5,
            COL_TEXT_WHITE,
        );

        // Define button layouts
        let btn_forewards = ButtonDef {
            x: 15,
            y: 100,
            w: 370,
            h: 25,
            text: format!("FOREWARDS: {:?}", key_binds.forewards).to_string(),
        };
        
        let btn_backwards = ButtonDef {
            x: 15,
            y: 135,
            w: 370,
            h: 25,
            text: format!("BACKWARDS: {:?}", key_binds.backwards).to_string(),
        };

        let btn_sprint = ButtonDef {
            x: 15,
            y: 170,
            w: 370,
            h: 25,
            text: format!("SPRINT: {:?}", key_binds.sprint).to_string(),
        };

        let btn_crouch = ButtonDef {
            x: 15,
            y: 205,
            w: 370,
            h: 25,
            text: format!("CROUCH: {:?}", key_binds.crouch).to_string(),
        };

        let btn_interact = ButtonDef {
            x: 15,
            y: 240,
            w: 370,
            h: 25,
            text: format!("INTERACT: {:?}", key_binds.interact).to_string(),
        };

        let btn_move_crosshair_down = ButtonDef {
            x: 15,
            y: 275,
            w: 370,
            h: 25,
            text: format!("CROSSHAIR DOWN: {:?}", key_binds.crosshair_down).to_string(),
        };

        let btn_turn_right = ButtonDef {
            x: 15,
            y: 310,
            w: 370,
            h: 25,
            text: format!("TURN RIGHT; {:?}", key_binds.turn_right).to_string(),
        };

        let btn_fire_bullet = ButtonDef {
            x: 15,
            y: 345,
            w: 370,
            h: 25,
            text: format!("FIRE BULLET: {:?}", key_binds.fire_bullet).to_string(),
        };

        let btn_godmode = ButtonDef {
            x: 15,
            y: 380,
            w: 370,
            h: 25,
            text: format!("ENTER GODMODE: {:?}", key_binds.godmode).to_string(),
        };

        let btn_left = ButtonDef {
            x: 415,
            y: 100,
            w: 370,
            h: 25,
            text: format!("LEFT: {:?}", key_binds.left).to_string(),
        };

        let btn_right = ButtonDef {
            x: 415,
            y: 135,
            w: 370,
            h: 25,
            text: format!("RIGHT: {:?}", key_binds.right).to_string(),
        };

        let btn_jump = ButtonDef {
            x: 415,
            y: 170,
            w: 370,
            h: 25,
            text: format!("JUMP: {:?}", key_binds.jump).to_string(),
        };

        let btn_rocket_jump = ButtonDef {
            x: 415,
            y: 205,
            w: 370,
            h: 25,
            text: format!("ROCKET JUMP: {:?}", key_binds.rocket_jump).to_string(),
        };

        let btn_move_crosshair_up = ButtonDef {
            x: 415,
            y: 240,
            w: 370,
            h: 25,
            text: format!("CROSSHAIR UP: {:?}", key_binds.crosshair_up).to_string(),
        };

        let btn_turn_left = ButtonDef {
            x: 415,
            y: 275,
            w: 370,
            h: 25,
            text: format!("TURN LEFT: {:?}", key_binds.turn_left).to_string(),
        };

        let btn_toggle_fine_aiming = ButtonDef {
            x: 415,
            y: 310,
            w: 370,
            h: 25,
            text: format!("TOGGLE FINE AIMING: {:?}", key_binds.fine_aiming).to_string(),
        };

        let btn_fire_arrow = ButtonDef {
            x: 415,
            y: 345,
            w: 370,
            h: 25,
            text: format!("FIRE ARROW: {:?}", key_binds.fire_arrow).to_string(),
        };

        let btn_lock_enemies = ButtonDef {
            x: 415,
            y: 380,
            w: 370,
            h: 25,
            text: format!("LOCK ENEMIES: {:?}", key_binds.lock_enemies).to_string(),
        };

        let btn_menu = ButtonDef {
            x: 230,
            y: 415,
            w: 370,
            h: 25,
            text: "[ESC] MAIN MENU".to_string(),
        };

        // Interaction processing
        // Start listening when the button is clicked
        if self.draw_flat_button(buffer, &btn_forewards, mx, my) && clicked {
            self.rebinding_key = Some(0);
        }

        if self.draw_flat_button(buffer, &btn_backwards, mx, my) && clicked {
            self.rebinding_key = Some(1);
        }

        if self.draw_flat_button(buffer, &btn_sprint, mx, my) && clicked {
            self.rebinding_key = Some(2);
        }

        if self.draw_flat_button(buffer, &btn_crouch, mx, my) && clicked {
            self.rebinding_key = Some(3);
        }
        
        if self.draw_flat_button(buffer, &btn_interact, mx, my) && clicked {
            self.rebinding_key = Some(4);
        }
        
        if self.draw_flat_button(buffer, &btn_move_crosshair_down, mx, my) && clicked {
            self.rebinding_key = Some(5);
        }
        
        if self.draw_flat_button(buffer, &btn_turn_right, mx, my) && clicked {
            self.rebinding_key = Some(6);
        }
        
        if self.draw_flat_button(buffer, &btn_fire_bullet, mx, my) && clicked {
            self.rebinding_key = Some(7);
        }
        
        if self.draw_flat_button(buffer, &btn_godmode, mx, my) && clicked {
            self.rebinding_key = Some(8);
        }
        
        if self.draw_flat_button(buffer, &btn_left, mx, my) && clicked {
            self.rebinding_key = Some(9);
        }

        if self.draw_flat_button(buffer, &btn_right, mx, my) && clicked {
            self.rebinding_key = Some(10);
        }

        if self.draw_flat_button(buffer, &btn_jump, mx, my) && clicked {
            self.rebinding_key = Some(11);
        }

        if self.draw_flat_button(buffer, &btn_rocket_jump, mx, my) && clicked {
            self.rebinding_key = Some(12);
        }

        if self.draw_flat_button(buffer, &btn_move_crosshair_up, mx, my) && clicked {
            self.rebinding_key = Some(13);
        }

        if self.draw_flat_button(buffer, &btn_turn_left, mx, my) && clicked {
            self.rebinding_key = Some(14);
        }

        if self.draw_flat_button(buffer, &btn_toggle_fine_aiming, mx, my) && clicked {
            self.rebinding_key = Some(15);
        }

        if self.draw_flat_button(buffer, &btn_fire_arrow, mx, my) && clicked {
            self.rebinding_key = Some(16);
        }

        if self.draw_flat_button(buffer, &btn_lock_enemies, mx, my) && clicked {
            self.rebinding_key = Some(17);
        }

        if self.draw_flat_button(buffer, &btn_menu, mx, my) && clicked {
            return AppState::StartScreen;
        }

        // Capture first, so the text below already shows the new key
        if let Some(id) = self.rebinding_key {
            let keys = window.get_keys_pressed(KeyRepeat::No);
            if let Some(&key) = keys.first() {
                match id {
                    0 => key_binds.forewards = key,
                    1 => key_binds.backwards = key,
                    2 => key_binds.sprint = key,
                    3 => key_binds.crouch = key, 
                    4 => key_binds.interact = key,
                    5 => key_binds.crosshair_down = key,
                    6 => key_binds.turn_right = key,
                    7 => key_binds.fire_bullet = key,
                    8 => key_binds.godmode = key,
                    9 => key_binds.left = key,
                    10 => key_binds.right = key, 
                    11 => key_binds.jump = key,
                    12 => key_binds.rocket_jump = key,
                    13 => key_binds.crosshair_up = key,
                    14 => key_binds.turn_left = key,
                    15 => key_binds.fine_aiming = key,
                    16 => key_binds.fire_arrow = key,
                    17 => key_binds.lock_enemies = key,
                    _ => {}
                }
                key_binds.save();
                self.rebinding_key = None;
                return AppState::KeyBindings;
            }
        }

        // Keep looping in the menu until an action changes the state
        AppState::KeyBindings
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


// Helper since Key can’t be turned back into a key from text automatically, so this looks the name up in a list.
fn key_from_name(name: &str) -> Option<Key> {
    const KEYS: &[Key] = &[
        Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I,
        Key::J, Key::K, Key::L, Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R,
        Key::S, Key::T, Key::U, Key::V, Key::W, Key::X, Key::Y, Key::Z,
        Key::Key0, Key::Key1, Key::Key2, Key::Key3, Key::Key4,
        Key::Key5, Key::Key6, Key::Key7, Key::Key8, Key::Key9,
        Key::Up, Key::Down, Key::Left, Key::Right,
        Key::Space, Key::Enter, Key::Tab, Key::Backspace,
        Key::LeftShift, Key::RightShift, Key::LeftCtrl, Key::RightCtrl,
        Key::LeftAlt, Key::RightAlt,
        Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6,
        Key::F7, Key::F8, Key::F9, Key::F10, Key::F11, Key::F12,
    ];

    KEYS.iter().copied().find(|k| format!("{:?}", k) == name)
}