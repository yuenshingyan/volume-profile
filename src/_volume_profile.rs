use pyo3::prelude::*;
use std::collections::HashMap;
use std::error::Error;

#[pyfunction]
pub fn compute_volume_profile(
    close: Vec<f64>,
    volume: Vec<i64>,
    bins: usize,
    window: usize,
) -> (Vec<Option<HashMap<String, Vec<f64>>>>, Vec<Option<f64>>) {
    if close.len() != volume.len() {
        panic!(
            "Lenth of argument `close` ({}) must share the same length with argument `volume` ({})",
            close.len(),
            volume.len()
        );
    }

    if window > close.len() {
        panic!(
            "Argument `window` ({}) must be less than the length of argument `close` ({})",
            window,
            close.len()
        );
    }

    let mut close_max = 0.0;
    for c in &close {
        close_max = f64::max(close_max, *c);
    }

    let offset = window - 1;
    let mut point_of_control = vec![None; offset];
    let mut volume_profile = vec![None; offset];
    for i in 0..close.len() {
        if i + window > close.len() {
            break;
        } else {
            let histogram = compute_histogram(
                &close[i..i + window],
                &volume[i..i + window],
                bins,
                close_max / bins as f64,
            )
            .expect("failed to compute histogram");
            point_of_control.push(Some(compute_point_of_control(&histogram)));
            volume_profile.push(Some(histogram));
        }
    }
    (volume_profile, point_of_control)
}

fn compute_point_of_control(histogram: &HashMap<String, Vec<f64>>) -> f64 {
    let mut max_middle = 0.0;
    for k in histogram.keys() {
        let freq = histogram.get(k).expect("failed to get value from key")[2];
        max_middle = f64::max(max_middle, freq);
    }
    max_middle
}

fn compute_histogram(
    close_slice: &[f64],
    volume_slice: &[i64],
    bins: usize,
    bin_width: f64,
) -> Result<HashMap<String, Vec<f64>>, Box<dyn Error>> {
    let mut volume_profile = HashMap::<String, Vec<f64>>::new();
    for i in 0..close_slice.len() {
        let frequency = volume_slice[i] as f64;

        for n in 0..bins {
            let lower = n as f64 * bin_width;
            let upper = (n + 1) as f64 * bin_width;
            let middle = (lower + upper) / 2.0;
            let interval = format!("({}, {})", lower, upper);

            if lower <= close_slice[i] && close_slice[i] <= upper {
                if volume_profile.contains_key(&interval) {
                    let existing_bin = volume_profile
                        .remove(&interval)
                        .expect("failed to remove 'interval' from 'volume_profile'");
                    let existing_freq = existing_bin[existing_bin.len() - 1];
                    volume_profile.insert(
                        interval.clone(),
                        vec![lower, upper, middle, existing_freq + frequency],
                    );
                    break;
                } else {
                    volume_profile.insert(interval.clone(), vec![lower, upper, middle, frequency]);
                    break;
                }
            }
        }
    }
    Ok(volume_profile)
}
