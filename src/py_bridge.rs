use pyo3::prelude::*;
use std::thread;
use crossbeam_channel::unbounded;
use crate::sniff::{NetworkEvent, start_sniffing};

#[pyclass]
#[derive(Clone)]
pub struct PyNetworkEvent {
    #[pyo3(get)]
    pub protocol: String,
    #[pyo3(get)]
    pub source: String,
    #[pyo3(get)]
    pub destination: String,
    #[pyo3(get)]
    pub payload_size: usize,
    #[pyo3(get)]
    pub timestamp: f64,
}

impl From<NetworkEvent> for PyNetworkEvent {
    fn from(event: NetworkEvent) -> Self {
        PyNetworkEvent {
            protocol: event.protocol,
            source: event.source,
            destination: event.destination,
            payload_size: event.payload_size,
            timestamp: event.timestamp,
        }
    }
}

/// Start network sniffing and send events to a Python callback
#[pyfunction]
#[pyo3(signature = (callback, interface=None))]  // Fixed: required parameter before optional
fn start_sniffing_py(callback: PyObject, interface: Option<String>) -> PyResult<()> {
    let (sender, receiver) = unbounded::<NetworkEvent>();
    
    // Convert Option<String> to Option<&str> for the interface
    let iface_owned = interface.clone();
    
    // Spawn the sniffing thread
    thread::spawn(move || {
        if let Some(ref iface) = iface_owned {
            if let Err(e) = start_sniffing(Some(iface), sender) {
                eprintln!("Error in sniffing thread: {:?}", e);
            }
        } else {
            if let Err(e) = start_sniffing(None, sender) {
                eprintln!("Error in sniffing thread: {:?}", e);
            }
        }
    });

    // Spawn a thread to handle events and call Python callback
    thread::spawn(move || {
        for event in receiver {
            Python::with_gil(|py| {
                let py_event = PyNetworkEvent::from(event);
                if let Ok(py_event) = Py::new(py, py_event) {
                    if let Err(e) = callback.call1(py, (py_event,)) {
                        eprintln!("Error calling Python callback: {:?}", e);
                    }
                }
            });
        }
    });

    Ok(())
}

/// Python module
#[pymodule]
fn sniff(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_sniffing_py, m)?)?;
    m.add_class::<PyNetworkEvent>()?;
    Ok(())
}
