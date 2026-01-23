/// =============================================
///  Ascender AgentHub - ATTP Core
///
///  Copyright (c) 2025 Ascender
///  All rights reserved.
///
///  This file is part of the Ascender AgentHub project.
///  Unauthorized copying, distribution, or modification
///  of this file, via any medium, is strictly prohibited.
///
///  This software is proprietary and confidential.
///  Use of this source code is governed by the license
///  agreement provided with the project.
///
///  Developed by the Ascender AgentHub team.
/// =============================================


use pyo3::{prelude::*, exceptions::PyRuntimeError};
use env_logger::Env;

use crate::{attp::{client::client_session::AttpClientSession, shared::{command::{AttpCommand, AttpMessage}, pyattp_message::PyAttpMessage, session::Session}, server::transport::AttpTransport}, config::Limits};

mod config;
mod attp;


#[pymodule]
#[pyo3(name="rs_api")]
fn attp_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Ensure Python is initialized for callbacks from tokio worker threads.
    pyo3::prepare_freethreaded_python();

    m.add_class::<AttpMessage>()?;
    m.add_class::<PyAttpMessage>()?;
    m.add_class::<Session>()?;
    m.add_class::<AttpTransport>()?;
    m.add_class::<Limits>()?;
    m.add_class::<AttpCommand>()?;
    m.add_class::<AttpClientSession>()?;
    m.add_function(wrap_pyfunction!(init_logging, m)?)?;

    Ok(())
}

#[pyfunction]
fn init_logging() -> PyResult<()> {
    // Initialize logger once; ignore error if already initialized.
    let mut builder = env_logger::Builder::from_env(Env::default().default_filter_or("info"));
    if let Err(e) = builder.try_init() {
        if e.to_string().contains("initialized") {
            return Ok(());
        }
        return Err(PyRuntimeError::new_err(e.to_string()));
    }
    Ok(())
}
