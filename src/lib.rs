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


use pyo3::prelude::*;

use crate::{attp::{client::client_session::AttpClientSession, shared::{command::{AttpCommand, AttpMessage}, pyattp_message::PyAttpMessage, session::Session}, server::transport::AttpTransport}, config::Limits};

mod config;
mod attp;


#[pymodule]
#[pyo3(name="rs_api")]
fn attp_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<AttpMessage>()?;
    m.add_class::<PyAttpMessage>()?;
    m.add_class::<Session>()?;
    m.add_class::<AttpTransport>()?;
    m.add_class::<Limits>()?;
    m.add_class::<AttpCommand>()?;
    m.add_class::<AttpClientSession>()?;

    Ok(())
}