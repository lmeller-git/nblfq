use pyo3::prelude::*;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[pymodule]
#[pyo3(name = "_nblf_queue_py")]
fn nblf_queue_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add your classes...
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
