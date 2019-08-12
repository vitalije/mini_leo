mod model;
mod utils;
mod parsing;

pub use crate::parsing::{ldf_parse,from_derived_file_content};
pub use crate::utils::{b64int, b64str, rpartition};
pub use crate::model::{VData, Outline, OutlineOps, LevGnx, LevGnxOps};
use pyo3::prelude::*;
use pyo3::{wrap_pyfunction};
use pyo3::type_object::PyTypeObject;
#[pyfunction]
pub fn a_function_from_rust() -> PyResult<i32> {
    Ok(42)
}

#[pymodule]
fn _minileo(py: Python, m:&PyModule) -> PyResult<()> {
    /// creates outline from str
    #[pyfn(m, "outline_from_str")]
    fn outline_from_string(py: Python, txt:&str) -> PyResult<(Outline, Vec<VData>)> {
        let (o, nodes) = from_derived_file_content(txt);
        Ok((o, nodes))
    }
    m.add_wrapped(wrap_pyfunction!(a_function_from_rust))?;
    m.add("VData", <VData as PyTypeObject>::type_object())?;
    Ok(())
}
