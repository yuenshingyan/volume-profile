use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;

#[pyfunction]
#[pyo3(signature = (close, volume, bins=100.0, window=255.0))]
pub fn compute_volume_profile(
    close: Vec<f64>,
    volume: Vec<f64>,
    bins: f64,
    window: f64,
) -> PyResult<(Vec<Option<f64>>, Vec<Option<HashMap<String, Vec<f64>>>>)> {
    if bins.fract() != 0.0 {
        let message = format!("Argument `bins` must be integer");
        return Err(PyValueError::new_err(message));
    } 

    if bins <= 0.0 {
        let message = format!("Argument `bins` must be greater than 0");
        return Err(PyValueError::new_err(message));
    }

    if window.fract() != 0.0 {
        let message = format!("Argument `window` must be integer");
        return Err(PyValueError::new_err(message));
    } 

    if window <= 0.0 {
        let message = format!("Argument `window` must be greater than 0");
        return Err(PyValueError::new_err(message));
    }

    let bins = bins as usize;
    let window = window as usize;

    if close.len() != volume.len() {
        let message = format!("Lenth of argument `close` ({}) must share the same length with argument `volume` ({})", close.len(), volume.len());
        return Err(PyValueError::new_err(message));
    }

    if window > close.len() {
        let message = format!("Argument `window` ({}) must be less than the length of argument `close` ({})", window, close.len());
        return Err(PyValueError::new_err(message));
    }

    if window > volume.len() {
        let message = format!("Argument `window` ({}) must be less than the length of argument `volume` ({})", window, close.len());
        return Err(PyValueError::new_err(message));
    }

    // Compute minimum and maximum close price and bin width of histogram.
    let mut close_min = 0.0;
    let mut close_max = 0.0;
    for c in &close {
        close_min = f64::min(close_min, *c);
        close_max = f64::max(close_max, *c);
    }
    let bin_width = (close_max - close_min) / bins as f64;

    // Compute histogram over a sliding window, and use computed histogram to find the point of control.
    let (point_of_control, volume_profile) = (0..close.len())
        .into_par_iter()
        .map(|i| {
            if i >= window {
                let histogram = compute_histogram(
                    &close[i - window..i],
                    &volume[i - window..i],
                    bins,
                    bin_width,
                )
                .expect("failed to compute histogram");
                (Some(compute_point_of_control(&histogram)), Some(histogram))
            } else {
                (None, None)
            }
        })
        .collect::<(Vec<Option<f64>>, Vec<Option<HashMap<String, Vec<f64>>>>)>();
    Ok((point_of_control, volume_profile))
}

fn compute_point_of_control(histogram: &HashMap<String, Vec<f64>>) -> f64 {
    // Find middle value of the maximum frequency as point of control.
    let mut max_middle = 0.0;
    for k in histogram.keys() {
        let freq = histogram.get(k).expect("failed to get value from key")[2];
        max_middle = f64::max(max_middle, freq);
    }
    max_middle
}

fn compute_histogram(
    close_slice: &[f64],
    volume_slice: &[f64],
    bins: usize,
    bin_width: f64,
) -> Result<HashMap<String, Vec<f64>>, Box<dyn Error>> {
    let mut volume_profile = HashMap::<String, Vec<f64>>::new();
    for i in 0..close_slice.len() {
        for n in 1..=bins {
            let lower = n as f64 * bin_width;
            let upper = (n + 1) as f64 * bin_width;
            let middle = (lower + upper) / 2.0;

            let interval;
            if n == 1 {
                interval = format!("[{}, {})", lower, upper);
            } else if n == bins {
                interval = format!("({}, {}]", lower, upper);
            } else {
                interval = format!("({}, {})", lower, upper);
            }

            if lower <= close_slice[i] && close_slice[i] < upper {
                if volume_profile.contains_key(&interval) {
                    let existing_bin = volume_profile
                        .remove(&interval)
                        .expect("failed to remove 'interval' from 'volume_profile'");
                    let existing_freq = existing_bin
                        .last()
                        .expect("failed to obtain existing frequency");
                    volume_profile.insert(
                        interval.clone(),
                        vec![lower, upper, middle, existing_freq + volume_slice[i]],
                    );
                    break;
                } else {
                    volume_profile.insert(
                        interval.clone(),
                        vec![lower, upper, middle, volume_slice[i]],
                    );
                    break;
                }
            }
        }
    }
    Ok(volume_profile)
}
