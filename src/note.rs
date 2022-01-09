use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Note {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl Sub for Note {
    type Output = i32;

    fn sub(self, rhs: Self) -> Self::Output {
        return self as i32 - rhs as i32;
    }
}

impl Note {
    pub fn text(&self) -> &'static str {
        return match self {
            Note::A => "A",
            Note::ASharp => "Bb",
            Note::B => "B",
            Note::C => "C",
            Note::CSharp => "Db",
            Note::D => "D",
            Note::DSharp => "Eb",
            Note::E => "E",
            Note::F => "F",
            Note::FSharp => "Gb",
            Note::G => "G",
            Note::GSharp => "Ab",
        };
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Key {
    note: Note,
    octave: i32,
}

impl Key {
    pub const fn new(note: Note, octave: i32) -> Self {
        Key { note, octave }
    }

    pub fn transpose(&self, amount: i32) -> Key {
        let current_index = self.note as i32 + NOTES_LEN * self.octave as i32;
        let new_index = current_index + amount;
        let mut new_octave = new_index / NOTES_LEN;
        let mut new_note = new_index % NOTES_LEN;
        while new_note < 0 {
            new_octave -= 1;
            new_note += NOTES_LEN;
        }
        Key {
            note: NOTES[new_note as usize],
            octave: new_octave,
        }
    }

    pub fn note(&self) -> Note {
        return self.note;
    }

    pub fn octave(&self) -> i32 {
        return self.octave;
    }

    pub fn text(&self) -> String {
        return format!("{}{}", self.note.text(), self.octave);
    }
}

impl Add<i32> for Key {
    type Output = Key;

    fn add(self, rhs: i32) -> Self::Output {
        return self.transpose(rhs);
    }
}

impl Sub for Key {
    type Output = i32;

    fn sub(self, rhs: Self) -> Self::Output {
        return (self.octave - rhs.octave) * NOTES_LEN + (self.note - rhs.note);
    }
}

pub static NOTES: [Note; 12] = [
    Note::C,
    Note::CSharp,
    Note::D,
    Note::DSharp,
    Note::E,
    Note::F,
    Note::FSharp,
    Note::G,
    Note::GSharp,
    Note::A,
    Note::ASharp,
    Note::B,
];

static NOTES_LEN: i32 = NOTES.len() as i32;
