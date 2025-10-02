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

use pyo3::{
    Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python,
    exceptions::{PyOSError},
    pyclass, pymethods,
    types::{PyAnyMethods, PyInt, PyString},
};
use tokio::{
    net::TcpListener,
    sync::watch::{self, Sender},
};
use uuid::Uuid;

use crate::{
    attp::shared::{session::Session},
    config::Limits,
};

#[pyclass]
pub struct AttpTransport {
    #[pyo3(get)]
    pub host: String,
    #[pyo3(get)]
    pub port: u16,
    #[pyo3(get)]
    pub limits: Limits,
    // #[pyo3(get)]
    pub on_connection: Py<PyAny>,

    shutdown_tx: Option<Sender<bool>>,
}

#[pymethods]
impl AttpTransport {
    #[new]
    pub fn new(
        host: Bound<PyString>,
        port: Bound<PyInt>,
        on_connection: Py<PyAny>,
        limits: Limits,
    ) -> PyResult<Self> {
        Ok(Self {
            host: host.extract()?,
            port: port.extract()?,
            shutdown_tx: None,
            on_connection,
            limits,
        })
    }

    pub fn start_server<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let host = self.host.clone();
        let port = self.port.clone();

        let connection_hook = self.on_connection.clone_ref(py);
        let limits = self.limits.clone();

        let (tx, mut rx) = watch::channel(false);

        self.shutdown_tx = Some(tx);

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let listener = TcpListener::bind((host.as_str(), port))
                .await
                .map_err(|e| {
                    PyOSError::new_err(format!("Failed to estabilish connection because of {}", e))
                })?;

            loop {
                tokio::select! {
                        _ = rx.changed() => break,
                        accept_res = listener.accept() => {
                            let (sock, _) = accept_res.map_err(|e| PyOSError::new_err(format!("Failed to accept connection {e}")))?;

                            let session_id = Uuid::new_v4().to_string();
                            let session = Session::new(sock, session_id, limits);

                            let fire = Python::attach(|pyt| {
                                let mut py_session = session.clone();
                                py_session.start_listening();
                                let object = py_session.into_bound_py_any(pyt)?;

                                let connect_hook = connection_hook.bind(pyt);
                                
                                pyo3_async_runtimes::tokio::into_future(connect_hook.call1((object,))?)
                            })?;
                            tokio::spawn(async move {
                                let _ = fire.await;
                            });
                        }
                }
            }

            return Ok(());
        })
    }

    pub fn stop_server(&self) -> PyResult<()> {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(true);
        }
        Ok(())
    }
}
