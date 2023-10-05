use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

#[pyfunction]
fn watch(path: &str) -> PyResult<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

    watcher.watch(Path::new(path), RecursiveMode::Recursive).unwrap();

    for res in rx {
        match res {
            Ok(event) => println!("Change: {:?}", event),
            Err(error) => println!("Error: {:?}", error),
        }
    }

    Ok(())
}

#[pymodule]
fn xpectate(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(watch, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::IntoPyDict;

    #[test]
    fn test_watch() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let rust_watcher = PyModule::new(py, "rust_watcher").unwrap();
        let locals = [("rust_watcher", rust_watcher)].into_py_dict(py);
        let res: PyResult<()> = py.eval("rust_watcher.watch('/path/to/watch')", Some(locals), None);
        assert!(res.is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::IntoPyDict;

    #[test]
    fn test_watch() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let rust_watcher = PyModule::new(py, "rust_watcher").unwrap();
        let locals = [("rust_watcher", rust_watcher)].into_py_dict(py);
        let res: PyResult<()> = py.eval("rust_watcher.watch('/path/to/watch')", Some(locals), None);
        assert!(res.is_ok());
    }
}
