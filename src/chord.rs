use crate::note::{Key, Note};

static ROOT_KEY: Key = Key::new(Note::A, 0);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Chord {
    Major,
    Minor,
    Seventh,
    MajorSeventh,
    MinorSeventh,
    Diminished,
    All,
}

pub static CHORDS: [Chord; 7] = [
    Chord::Major,
    Chord::Minor,
    Chord::Seventh,
    Chord::MajorSeventh,
    Chord::MinorSeventh,
    Chord::Diminished,
    Chord::All,
];

impl Chord {
    pub fn get_text(&self) -> &'static str {
        return match self {
            Chord::Major => "",
            Chord::Minor => "m",
            Chord::Seventh => "7",
            Chord::MinorSeventh => "m7",
            Chord::MajorSeventh => "maj7",
            Chord::Diminished => "dim",
            Chord::All => "(All)",
        };
    }

    pub fn get_keys(&self, base: Key) -> Vec<Key> {
        return self
            .get_chord_notes()
            .into_iter()
            .map(|note| base.transpose(note - ROOT_KEY))
            .collect();
    }

    fn get_chord_notes(&self) -> Vec<Key> {
        return self
            .get_chord_intervals()
            .into_iter()
            .map(|interval| ROOT_KEY + interval)
            .collect();
    }

    fn get_chord_intervals(&self) -> Vec<i32> {
        return match self {
            Chord::Major => vec![0, 4, 7],
            Chord::Minor => vec![0, 3, 7],
            Chord::Seventh => vec![0, 4, 7, 10],
            Chord::MajorSeventh => vec![0, 4, 7, 11],
            Chord::MinorSeventh => vec![0, 3, 7, 10],
            Chord::Diminished => vec![0, 3, 6, 9],
            Chord::All => vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        };
    }
}
