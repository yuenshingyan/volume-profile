pub mod _volume_profile;

use crate::_volume_profile::compute_volume_profile;
use pyo3::prelude::*;

#[pymodule]
fn volume_profile(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_volume_profile, m)?)?;
    Ok(())
}
