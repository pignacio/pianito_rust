use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer::{Chunk, AUDIO_S16LSB, DEFAULT_CHANNELS};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
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
        1 => "2m".to_owned(),
        2 => "2".to_owned(),
        3 => "3m".to_owned(),
        4 => "3".to_owned(),
        5 => "4".to_owned(),
        6 => "4a".to_owned(),
        7 => "5".to_owned(),
        8 => "5a".to_owned(),
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
    let window = video.window("Testing window", 800, 600).build().unwrap();
    let timer = sdl2.timer()?;

    sdl2::mixer::allocate_channels(16);
    let font = Rc::new(ttf.load_font("Inconsolata.otf", 30)?);

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let keyboard = Keyboard::new(font.clone(), canvas.texture_creator());
    let mut frame_count = 0;

    let root_key = Key::new(Note::C, 3);
    let mut current_key = root_key;
    let mut current_chord_index = 0i32;

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

    'running: loop {
        canvas.set_draw_color(Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });
        canvas.clear();
        for event in pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape) | Some(Keycode::Q),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => current_key = current_key.transpose(1),
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => current_key = current_key.transpose(-1),
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    current_chord_index -= 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    current_chord_index += 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    let current_chord =
                        CHORDS[positive_remainder(current_chord_index, CHORDS.len())];
                    sdl2::mixer::Channel::all().halt();
                    for key in current_chord.get_keys(current_key) {
                        play(key, &mut sounds, &mut sounding_until, &timer);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    let current_chord =
                        CHORDS[positive_remainder(current_chord_index, CHORDS.len())];
                    sdl2::mixer::Channel::all().halt();
                    let mut time = timer.ticks();

                    for key in current_chord.get_keys(current_key) {
                        pending_sounds.push_back(PendingSound { key, time });
                        time += 400;
                    }
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
        let current_chord = CHORDS[positive_remainder(current_chord_index, CHORDS.len())];
        for key in current_chord.get_keys(current_key) {
            states.insert(
                key,
                KeyState::new(
                    if current_ticks < sounding_until.get(&key).map(|x| *x).unwrap_or(0u32) {
                        State::SOUNDING
                    } else {
                        State::PRESSED
                    },
                    Some(get_interval_text(key - current_key)),
                ),
            );
        }

        let chord_text = format!("{}{}", current_key.note().text(), current_chord.get_text());
        let rendered = font.render(&chord_text).blended(Color::WHITE).unwrap();
        let text_rect = rendered.rect();
        let rendered_texture = texture_creator
            .create_texture_from_surface(rendered)
            .unwrap();
        canvas
            .copy(&rendered_texture, text_rect, text_rect)
            .unwrap();

        keyboard.draw_keyboard(&mut canvas, Rect::new(50, 150, 700, 300), &states);

        canvas.present();
        frame_count += 1;
        sleep(Duration::from_millis(10));
    }

    println!("Hello, world!");
    Ok(())
}
