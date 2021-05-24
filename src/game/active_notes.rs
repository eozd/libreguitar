use crate::core::{FretLoc, FretRange, Note, NoteRegistry, StringRange, Tuning};
use std::collections::HashMap;

pub struct ActiveNotes {
    pub string_range: StringRange,
    pub fret_range: FretRange,
    notes: HashMap<FretLoc, Note>,
}

impl ActiveNotes {
    pub fn new(
        registry: &NoteRegistry,
        tuning: &Tuning,
        string_range: StringRange,
        fret_range: FretRange,
    ) -> ActiveNotes {
        let active_locs = active_locations(&string_range, &fret_range);
        let locs_and_notes = locs2notes(active_locs.into_iter(), tuning, registry);
        let mut notes = HashMap::new();
        for (loc, maybe_note) in locs_and_notes {
            if let Some(note) = maybe_note {
                notes.insert(loc, note.clone());
            } else {
                println!(
                    "Note on string {} fret {} does not exist in frequency list. Skipping...",
                    loc.string_idx, loc.fret_idx
                );
            }
        }

        ActiveNotes {
            string_range,
            fret_range,
            notes,
        }
    }

    pub fn get<'a>(&'a self, loc: &FretLoc) -> Option<&'a Note> {
        self.notes.get(loc)
    }
}

fn locs2notes<'a>(
    locs: impl Iterator<Item = FretLoc>,
    tuning: &'a Tuning,
    registry: &'a NoteRegistry,
) -> impl Iterator<Item = (FretLoc, Option<&'a Note>)> {
    locs.map(move |loc| {
        let open_string_note = tuning.note(loc.string_idx).unwrap();
        let fret_idx = loc.fret_idx as i32;
        (loc, registry.add_semitones(open_string_note, fret_idx))
    })
}

fn active_locations(string_range: &StringRange, fret_range: &FretRange) -> Vec<FretLoc> {
    let mut out = Vec::new();
    for string_idx in string_range.r() {
        for fret_idx in fret_range.r() {
            out.push(FretLoc {
                string_idx,
                fret_idx,
            })
        }
    }
    out
}

