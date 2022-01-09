use chord::Chord;
use note::NOTES;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer::{Chunk, AUDIO_S16LSB, DEFAULT_CHANNELS};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use sdl2::TimerSubsystem;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;
use std::{collections::HashMap, thread::sleep};

use crate::chord::CHORDS;
use crate::keyboard::{KeyState, Keyboard, State};
use crate::note::{Key, Note};

mod chord;
mod keyboard;
mod note;

fn get_interval_text(interval: i32) -> String {
    return match interval {
        0 => "R".to_owned(),
        1 => "1a\n2m".to_owned(),
        2 => "2".to_owned(),
        3 => "2a\n3m".to_owned(),
        4 => "3".to_owned(),
        5 => "4".to_owned(),
        6 => "4a\n5m".to_owned(),
        7 => "5".to_owned(),
        8 => "5a\n6m".to_owned(),
        9 => "6".to_owned(),
        10 => "7".to_owned(),
        11 => "7M".to_owned(),
        12 => "8".to_owned(),
        _ => format!("({})", interval),
    };
}

fn positive_remainder(dividend: i32, divisor: usize) -> usize {
    let remainder = dividend % divisor as i32;
    let result = if remainder >= 0 {
        remainder
    } else {
        remainder + divisor as i32
    };
    return result as usize;
}

fn main() {
    run().unwrap()
}

fn play(
    key: Key,
    sounds: &mut HashMap<Key, Chunk>,
    sounding_until: &mut HashMap<Key, u32>,
    timer: &TimerSubsystem,
) {
    sounds
        .get(&key)
        .map(|sound| sdl2::mixer::Channel::all().play(&sound, 0).unwrap());
    sounding_until.insert(key, timer.ticks() + 3_000);
}

fn play_chord(
    keys: Vec<Key>,
    delay: u32,
    pending_sounds: &mut VecDeque<PendingSound>,
    timer: &TimerSubsystem,
) {
    sdl2::mixer::Channel::all().halt();
    pending_sounds.clear();
    let mut time = timer.ticks();

    for key in keys {
        pending_sounds.push_back(PendingSound { key, time });
        time += delay;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingSound {
    key: Key,
    time: u32,
}

fn run() -> Result<(), String> {
    let sdl2 = sdl2::init()?;
    let ttf = sdl2::ttf::init().unwrap();
    let _audio = sdl2.audio()?;
    sdl2::mixer::open_audio(44_100, AUDIO_S16LSB, DEFAULT_CHANNELS, 1024)?;
    let _mixer = sdl2::mixer::init(sdl2::mixer::InitFlag::OGG)?;
    let mut pump = sdl2.event_pump()?;
    let video = sdl2.video()?;
    let window = video.window("Testing window", 1200, 900).build().unwrap();
    let timer = sdl2.timer()?;

    sdl2::mixer::allocate_channels(16);
    let font = Rc::new(ttf.load_font("Inconsolata.otf", 30)?);

    let mut canvas = window.into_canvas().build().unwrap();
    let keyboard = Keyboard::new(font.clone(), canvas.texture_creator());
    let mut frame_count = 0;

    let root_key = Key::new(Note::C, 3);
    let mut current_key = root_key;

    let mut sounding_until: HashMap<Key, u32> = HashMap::new();

    let mut sounds: HashMap<Key, Chunk> = HashMap::new();
    for interval in 0..24 {
        let key = root_key + interval;
        let filename = format!("notes/{}{}.ogg", key.note().text(), key.octave());
        println!("Loading #{}: {}", interval, filename);
        let chunk = Chunk::from_file(filename)?;
        sounds.insert(key, chunk);
    }

    let mut pending_sounds: VecDeque<PendingSound> = VecDeque::with_capacity(5);

    let mut chord_table = ChordTable::new(font.clone(), canvas.texture_creator());

    'running: loop {
        canvas.set_draw_color(Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });
        canvas.clear();
        for event in pump.poll_iter() {
            let (chord_key, chord) = chord_table.current_chord();
            let chord_keys = chord.get_keys(chord_key);
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape) | Some(Keycode::Q),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => chord_table.right(),
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => chord_table.left(),
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => chord_table.down(),
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => chord_table.up(),
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    play_chord(chord_keys, 0, &mut pending_sounds, &timer);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    play_chord(chord_keys, 70, &mut pending_sounds, &timer);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    play_chord(chord_keys, 400, &mut pending_sounds, &timer);
                }
                _ => {}
            }
        }

        let current_ticks = timer.ticks();

        while let Some(pending) = pending_sounds.pop_front() {
            if current_ticks > pending.time {
                println!("Playing {:?}", pending);
                play(pending.key, &mut sounds, &mut sounding_until, &timer)
            } else {
                pending_sounds.push_front(pending);
                break;
            }
        }

        let _rendered = font
            .render(&format!("{}", frame_count))
            .blended(Color::WHITE)
            .unwrap();

        // canvas.copy(&rendered.as_texture(&texture_creator).unwrap(), None, None).unwrap();

        let mut states = HashMap::new();

        states.insert(
            root_key,
            KeyState::new(State::HIGHLIGHTED, Some(root_key.text())),
        );

        // for key in get_chord(root_c, Chord::Minor) {
        //     states.insert(key, KeyState::new(State::PRESSED, None));
        // }
        let (chord_key, chord) = chord_table.current_chord();
        for key in chord.get_keys(chord_key) {
            states.insert(
                key,
                KeyState::new(
                    if current_ticks < sounding_until.get(&key).map(|x| *x).unwrap_or(0u32) {
                        State::SOUNDING
                    } else {
                        State::PRESSED
                    },
                    Some(get_interval_text(key - chord_key)),
                ),
            );
        }

        // let chord_text = format!("{}{}", current_key.note().text(), current_chord.get_text());
        // let rendered = font.render(&chord_text).blended(Color::WHITE).unwrap();
        // let text_rect = rendered.rect();
        // let rendered_texture = texture_creator
        //     .create_texture_from_surface(rendered)
        //     .unwrap();
        // canvas
        //     .copy(&rendered_texture, text_rect, text_rect)
        //     .unwrap();

        chord_table.draw(&mut canvas, Rect::new(0, 0, 1200, 600))?;

        keyboard.draw_keyboard(&mut canvas, Rect::new(150, 600, 900, 300), &states);

        canvas.present();
        frame_count += 1;
        sleep(Duration::from_millis(10));
    }

    println!("Hello, world!");
    Ok(())
}

struct ChordTable<'ttf> {
    font: Rc<Font<'ttf, 'static>>,
    texture_creator: TextureCreator<WindowContext>,
    note_index: i32,
    chord_index: i32,
}

impl<'ttf> ChordTable<'ttf> {
    pub fn right(&mut self) {
        self.note_index += 1;
    }

    pub fn left(&mut self) {
        self.note_index -= 1;
    }

    pub fn up(&mut self) {
        self.chord_index -= 1;
    }

    pub fn down(&mut self) {
        self.chord_index += 1;
    }

    pub fn current_chord(&self) -> (Key, Chord) {
        let note = NOTES[positive_remainder(self.note_index, NOTES.len())];
        (
            Key::new(note, 3),
            CHORDS[positive_remainder(self.chord_index, CHORDS.len())],
        )
    }

    pub fn draw<T: RenderTarget>(&self, canvas: &mut Canvas<T>, rect: Rect) -> Result<(), String> {
        let cell_height = rect.height() / CHORDS.len() as u32;
        let cell_width = rect.width() / NOTES.len() as u32;

        let current_x = positive_remainder(self.note_index, NOTES.len());
        let current_y = positive_remainder(self.chord_index, CHORDS.len());

        for (x, note) in NOTES.iter().enumerate() {
            for (y, chord) in CHORDS.iter().enumerate() {
                let chord_text = format!("{}{}", note.text(), chord.get_text());
                let color = if x == current_x && y == current_y {
                    Color::RED
                } else {
                    Color::WHITE
                };
                let rendered = self
                    .font
                    .render(&chord_text)
                    .blended(color)
                    .map_err(|err| err.to_string())?;
                let text_rect = rendered.rect();
                let rendered_texture = self
                    .texture_creator
                    .create_texture_from_surface(rendered)
                    .map_err(|e| e.to_string())?;
                canvas.copy(
                    &rendered_texture,
                    text_rect,
                    Rect::new(
                        rect.x + x as i32 * cell_width as i32,
                        rect.y + y as i32 * cell_height as i32,
                        text_rect.width(),
                        text_rect.height(),
                    ),
                )?;
            }
        }
        Ok(())
    }

    pub fn new(
        font: Rc<Font<'ttf, 'static>>,
        texture_creator: TextureCreator<WindowContext>,
    ) -> ChordTable<'ttf> {
        ChordTable {
            font,
            texture_creator,
            note_index: 0,
            chord_index: 0,
        }
    }
}
