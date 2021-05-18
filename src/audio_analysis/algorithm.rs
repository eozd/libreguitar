use crate::audio_analysis::target_notes::TargetNotes;
use crate::note::Note;
use statrs::statistics::Median;
use std::collections::HashMap;
use std::hash::Hash;

pub fn find_note(freq_spectrum: &[f64], delta_f: f64, target_notes: &TargetNotes) -> Option<Note> {
    // TODO: make the algorithm adaptive instead of hardcoding these constants
    let median = freq_spectrum.median();
    let mut peaks = find_peaks(freq_spectrum, Some(500. * median), Some(10));
    peaks.sort_unstable_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    let top_notes: Vec<&Note> = peaks
        .into_iter()
        .rev()
        .take(5)
        .map(|p| {
            let freq = (p.idx as f64) * delta_f;
            target_notes.get_closest(freq)
        })
        .collect();
    let top_notenames = top_notes.iter().map(|note| &note.name);
    if let Some(notename) = most_common(top_notenames) {
        let top_notes = top_notes.into_iter().filter(|x| x.name == *notename);
        let min_note = top_notes.min_by(|a, b| a.frequency.partial_cmp(&b.frequency).unwrap());
        if let Some(note) = min_note {
            return Some(note.clone());
        }
    }
    None
}

fn most_common<'a, T>(notes: impl Iterator<Item = &'a T>) -> Option<&'a T>
where
    T: Eq + Hash,
{
    let mut counts = HashMap::new();
    for val in notes {
        if let Some(count) = counts.get_mut(val) {
            *count += 1;
        } else {
            counts.insert(val, 1);
        }
    }
    if let Some((value, _)) = counts.into_iter().max_by_key(|(_, count)| *count) {
        Some(value)
    } else {
        None
    }
}

#[derive(Debug, PartialEq)]
struct Peak<T> {
    idx: usize,
    value: T,
}

impl<T> Peak<T> {
    fn new(idx: usize, value: T) -> Peak<T> {
        Peak { idx, value }
    }
}

fn find_peaks(
    signal: &[f64],
    min_height: Option<f64>,
    min_peak_dist: Option<usize>,
) -> Vec<Peak<f64>> {
    let n_samples = signal.len();
    if n_samples == 0 {
        return Vec::new();
    } else if n_samples == 1 {
        return vec![Peak::new(0, signal[0])];
    }
    let min_height = min_height.unwrap_or(0.0);
    let min_peak_dist = min_peak_dist.unwrap_or(0);
    let mut out: Vec<Peak<f64>> = Vec::new();
    for i in 0..n_samples {
        let greater_than_left = i == 0 || signal[i] > signal[i - 1];
        let greater_than_right = i == n_samples - 1 || signal[i] > signal[i + 1];
        let is_peak = greater_than_left && greater_than_right && signal[i] >= min_height;
        let is_far_apart = out.is_empty() || i - out[out.len() - 1].idx >= min_peak_dist;
        if is_peak && is_far_apart {
            out.push(Peak::new(i, signal[i]));
        }
    }
    out
}

pub fn moving_avg(signal: &mut [f64], window_size: usize) {
    assert!(
        window_size > 0,
        "Moving avg for zero window size is undefined."
    );
    if signal.is_empty() || window_size == 1 {
        return;
    }
    let mut cumsum = vec![0.0f64; signal.len()];
    cumsum[0] = signal[0];
    for i in 1..signal.len() {
        cumsum[i] = cumsum[i - 1] + signal[i];
    }
    let left_offset = window_size / 2;
    let right_offset = window_size - 1 - left_offset;
    for i in (0..cumsum.len()).rev() {
        let right = (cumsum.len() - 1).min(i + right_offset);
        let avg = if i > left_offset {
            let left = i - left_offset - 1;
            (cumsum[right] - cumsum[left]) / (window_size as f64)
        } else {
            cumsum[right] / ((right + 1) as f64)
        };
        signal[i] = avg;
    }
}

#[cfg(test)]
mod tests_moving_avg {
    use super::moving_avg;

    #[test]
    fn moving_avg_empty_arr() {
        let mut signal = Vec::new();
        moving_avg(&mut signal, 1);
        assert_eq!(signal.len(), 0);

        moving_avg(&mut signal, 2);
        assert_eq!(signal.len(), 0);
    }

    #[test]
    #[should_panic]
    fn moving_avg_bad_window_size() {
        let mut signal = Vec::new();
        moving_avg(&mut signal, 0);
    }

    #[test]
    fn moving_avg_identity_case() {
        let mut signal = vec![1.0, 2.0, 3.0, 4.0];
        let expected = vec![1.0, 2.0, 3.0, 4.0];
        moving_avg(&mut signal, 1);
        assert_eq!(expected, signal);
    }

    #[test]
    fn moving_avg_general_case_0() {
        let mut signal = vec![1.0, 2.0, 3.0, 4.0];
        let expected = vec![1.0, 1.5, 2.5, 3.5];
        moving_avg(&mut signal, 2);
        assert_eq!(expected, signal);
    }

    #[test]
    fn moving_avg_general_case_1() {
        let mut signal = vec![1.0, 5.0, 10.0, 15.0, 7.5, -5.0];
        let expected = vec![
            3.0,
            16.0 / 3.0,
            30.0 / 3.0,
            32.5 / 3.0,
            17.5 / 3.0,
            2.5 / 3.0,
        ];
        moving_avg(&mut signal, 3);
        assert_eq!(expected, signal);
    }

    #[test]
    fn moving_avg_window_larger_than_arr_0() {
        let mut signal = vec![1.0];
        let expected = vec![1.0];
        moving_avg(&mut signal, 52);
        assert_eq!(expected, signal);
    }

    #[test]
    fn moving_avg_window_larger_than_arr_1() {
        let mut signal = vec![1.0, 5.0];
        let expected = vec![3.0, 3.0];
        moving_avg(&mut signal, 4);
        assert_eq!(expected, signal);
    }
}

#[cfg(test)]
mod tests_find_peaks {
    use super::{find_peaks, Peak};

    #[test]
    fn find_peaks_empty_arr() {
        let signal = Vec::new();
        let out = find_peaks(&signal, None, None);
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn find_peaks_single_elem() {
        let signal = vec![1.0];
        let expected = vec![Peak::new(0, 1.0)];
        let actual = find_peaks(&signal, None, None);
        assert_eq!(expected, actual);
    }

    #[test]
    fn find_peaks_decreasing() {
        let signal = vec![1.0, 0.5, 0.25];
        let expected = vec![Peak::new(0, 1.0)];
        let actual = find_peaks(&signal, None, None);
        assert_eq!(expected, actual);
    }

    #[test]
    fn find_peaks_parabola() {
        let signal = vec![1.0, 0.5, 0.25, 0.5, 1.0];
        let expected = vec![Peak::new(0, 1.0), Peak::new(4, 1.0)];
        let actual = find_peaks(&signal, None, None);
        assert_eq!(expected, actual);
    }

    #[test]
    fn find_peaks_two_peaks() {
        let signal = vec![0.5, 1.0, 2.0, 1.0, 0.0, 5.0, 2.5];
        let expected = vec![Peak::new(2, 2.0), Peak::new(5, 5.0)];
        let actual = find_peaks(&signal, None, None);
        assert_eq!(expected, actual);
    }
}

#[cfg(test)]
mod tests_most_common {
    use super::most_common;

    #[test]
    fn most_common_empty_iter() {
        let arr: Vec<i32> = Vec::new();
        let actual = most_common(arr.iter());
        assert_eq!(actual, None);
    }

    #[test]
    fn most_common_same_elem_0() {
        let arr = vec![1];
        let expected = 1;
        let actual = *most_common(arr.iter()).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn most_common_same_elem_1() {
        let arr = vec![2, 2, 2, 2];
        let expected = 2;
        let actual = *most_common(arr.iter()).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn most_common_equal_counts() {
        let arr = vec!["a", "b", "c"];
        let actual = *most_common(arr.iter()).unwrap();
        assert!(actual == "a" || actual == "b" || actual == "c");
    }

    #[test]
    fn most_common_general_case() {
        let arr = vec![("a", 1), ("b", -5), ("a", 1)];
        let expected = ("a", 1);
        let actual = *most_common(arr.iter()).unwrap();
        assert_eq!(expected, actual);
    }
}
