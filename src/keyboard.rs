use std::collections::HashMap;
use std::rc::Rc;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;

use crate::note::{Key, Note};

fn draw_bordered_rect<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    dest: Rect,
    color: Color,
    border: u32,
) -> Result<(), String> {
    assert!(dest.height() >= 2 * border);
    assert!(dest.width() >= 2 * border);
    canvas.set_draw_color(Color {
        r: 50,
        g: 50,
        b: 50,
        a: 255,
    });
    canvas.fill_rect(dest).unwrap();
    let copy = Rect::new(
        dest.x() + border as i32,
        dest.y() + border as i32,
        dest.width() - 2 * border,
        dest.height() - 2 * border,
    );
    canvas.set_draw_color(color);
    return canvas.fill_rect(copy);
}

static DARK_RED: Color = Color {
    r: 128,
    g: 0,
    b: 0,
    a: 255,
};
static DARK_CYAN: Color = Color {
    r: 0,
    g: 128,
    b: 128,
    a: 255,
};
static DARK_GREEN: Color = Color {
    r: 0,
    g: 128,
    b: 0,
    a: 255,
};

pub static WHITE_KEYS: [Key; 7] = [
    Key::new(Note::C, 3),
    Key::new(Note::D, 3),
    Key::new(Note::E, 3),
    Key::new(Note::F, 3),
    Key::new(Note::G, 3),
    Key::new(Note::A, 3),
    Key::new(Note::B, 3),
];

pub static BLACK_KEYS: [Option<Key>; 7] = [
    Some(Key::new(Note::CSharp, 3)),
    Some(Key::new(Note::DSharp, 3)),
    None,
    Some(Key::new(Note::FSharp, 3)),
    Some(Key::new(Note::GSharp, 3)),
    Some(Key::new(Note::ASharp, 3)),
    None,
];

#[derive(Debug, Clone, Copy)]
pub enum State {
    NORMAL,
    PRESSED,
    HIGHLIGHTED,
    SOUNDING,
}

impl Default for State {
    fn default() -> Self {
        return Self::NORMAL;
    }
}
pub struct KeyState {
    state: State,
    text: Option<String>,
}

impl KeyState {
    pub const fn new(state: State, text: Option<String>) -> Self {
        KeyState { state, text }
    }
}

impl Default for KeyState {
    fn default() -> Self {
        Self {
            state: Default::default(),
            text: Default::default(),
        }
    }
}

static DEFAULT_KEYSTATE: KeyState = KeyState::new(State::NORMAL, None);

pub struct Keyboard<'ttf> {
    font: Rc<Font<'ttf, 'static>>,
    texture_creator: TextureCreator<WindowContext>,
}

impl<'ttf> Keyboard<'ttf> {
    pub fn new(
        font: Rc<Font<'ttf, 'static>>,
        texture_creator: TextureCreator<WindowContext>,
    ) -> Self {
        Keyboard {
            font,
            texture_creator,
        }
    }

    fn build_white_key(index: usize) -> Key {
        WHITE_KEYS[index % 7].transpose(12 * (index as i32 / 7))
    }

    fn build_black_key(index: usize) -> Option<Key> {
        BLACK_KEYS[index % 7].map(|key| key.transpose(12 * (index as i32 / 7)))
    }

    fn draw_key_text<T: RenderTarget, U: AsRef<str>>(
        &self,
        canvas: &mut Canvas<T>,
        key_rect: Rect,
        text: U,
        color: Color,
    ) {
        let mut bottom = key_rect.y() + key_rect.height() as i32;
        let texts: Vec<&str> = text.as_ref().rsplit("\n").collect();
        for t in texts {
            let surface = self.font.render(t).blended(color).unwrap();
            let surface_rect = surface.rect();
            let texture = self
                .texture_creator
                .create_texture_from_surface(surface)
                .unwrap();
            let center = key_rect.x() + key_rect.width() as i32 / 2;
            let dest_rect = Rect::new(
                center - surface_rect.width() as i32 / 2,
                bottom - surface_rect.height() as i32,
                surface_rect.width(),
                surface_rect.height(),
            );
            canvas.copy(&texture, surface_rect, dest_rect).unwrap();
            bottom -= surface_rect.height() as i32;
        }
    }

    pub fn draw_keyboard<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        dest: Rect,
        states: &HashMap<Key, KeyState>,
    ) {
        let base_width = dest.width() / 24;
        let base_height = dest.height() / 3;
        let white_width = (3 * base_width) / 2;
        let white_height = dest.height();
        let black_width = base_width;
        let black_height = 2 * base_height;
        // let mut pos = 0;
        for white_position in 0..15 {
            // note = WHITE_KEYS[pos] + octave * 12
            // color = RED if state[note] else WHITE
            let key = Keyboard::build_white_key(white_position);
            let state = states.get(&key).unwrap_or(&DEFAULT_KEYSTATE);
            let color = match state.state {
                State::NORMAL => Color::WHITE,
                State::PRESSED => Color::RED,
                State::HIGHLIGHTED => Color::CYAN,
                State::SOUNDING => Color::GREEN,
            };
            let rect = Rect::new(
                dest.x() + (white_width * white_position as u32) as i32,
                dest.y(),
                white_width,
                white_height,
            );
            draw_bordered_rect(canvas, rect, color, 1).unwrap();
            state
                .text
                .as_ref()
                .map(|text| self.draw_key_text(canvas, rect, text, Color::BLACK));
        }
        for black_position in 0..14 {
            Keyboard::build_black_key(black_position).map(|key| {
                let state = states.get(&key).unwrap_or(&DEFAULT_KEYSTATE);
                let color = match state.state {
                    State::NORMAL => Color::BLACK,
                    State::PRESSED => DARK_RED,
                    State::HIGHLIGHTED => DARK_CYAN,
                    State::SOUNDING => DARK_GREEN,
                };
                let rect = Rect::new(
                    dest.x() + (white_width * black_position as u32 + base_width) as i32,
                    dest.y(),
                    black_width,
                    black_height,
                );
                draw_bordered_rect(canvas, rect, color, 1).unwrap();
                state
                    .text
                    .as_ref()
                    .map(|text| self.draw_key_text(canvas, rect, text, Color::WHITE));
            });

            // note = WHITE_KEYS[pos] + octave * 12
            // color = RED if state[note] else WHITE
        }
    }
}

// draw_bordered_rect(renderer, key, WHITE)
