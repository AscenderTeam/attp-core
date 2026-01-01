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
    Bound, PyAny, PyResult, Python,
    exceptions::{PyConnectionError, PyValueError},
    pyclass, pymethods,
};
use log::{error, info, warn};
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
    sync::mpsc,
};
use uuid::Uuid;

use crate::{
    attp::{
        shared::{command::AttpMessage, session::Session},
        utils::parse_connection_string,
    },
    config::Limits,
};

#[pyclass]
#[derive(Clone)]
pub struct AttpClientSession {
    #[pyo3(get)]
    pub host: String,
    #[pyo3(get)]
    pub port: u16,
    #[pyo3(get)]
    pub session: Option<Session>,
    #[pyo3(get)]
    pub limits: Limits,
}

impl AttpClientSession {
    pub fn from_session(host: String, port: u16, mut session: Session, limits: Limits) -> Self {
        session.start_listening();

        let instance = Self {
            host,
            port,
            session: Some(session),
            limits,
        };

        instance.pinger();
        return instance;
    }

    pub fn run_writer_task(mut writer: WriteHalf<TcpStream>) -> mpsc::Sender<Vec<u8>> {
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(3040);

        tokio::spawn(async move {
            while let Some(buf) = rx.recv().await {
                if writer.write_all(&buf).await.is_err() {
                    break;
                }
            }
        });

        return tx;
    }

    pub fn pinger(&self) {
        let inner = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                let Some(session) = inner.session.clone() else {
                    break;
                };

                if let Err(err) = session
                    ._send(AttpMessage::new(0, 6, None, None, b"01".clone()))
                    .await
                {
                    warn!("[AttpClientSession] Lost connection: {:?}", err);
                    // Gracefully shut down the session
                    if let Some(s) = inner.session.clone() {
                        let _ = s.stop_listener();
                    }
                    break;
                }
            }
        });
    }

    pub async fn try_reconnect(&self) -> Option<Session> {
        for attempt in 1..=3 {
            warn!("[AttpClientSession] Reconnect attempt {attempt}...");
            match self._connect().await {
                Some(session) => return Some(session),
                None => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
        None
    }

    pub async fn _connect(&self) -> Option<Session> {
        let inner = self.clone();
        let socket = TcpStream::connect(format!("{}:{}", inner.host, inner.port)).await;
        let session_id = Uuid::new_v4().to_string();

        if socket.is_err() {
            warn!(
                "[AttpClientSession] Failed to connect to {}:{}: {:?}",
                inner.host,
                inner.port,
                socket.as_ref().err()
            );
            return None;
        }

        let session = Session::new(socket.unwrap(), session_id, self.limits);

        return Some(session);
    }
}

#[pymethods]
impl AttpClientSession {
    #[new]
    fn new(connection_string: String, limits: Limits) -> PyResult<Self> {
        let (host, port) =
            parse_connection_string(connection_string).map_err(PyValueError::new_err)?;
        Ok(Self {
            host,
            port,
            session: None,
            limits,
        })
    }

    /// Getter to check if the client is connected
    /// Returns True if connected, False otherwise
    #[getter]
    pub fn is_connected(&self) -> bool {
        self.session.is_some()
    }

    pub fn disconnect(&mut self) -> PyResult<()> {
        if self.session.is_none() {
            return Err(PyConnectionError::new_err(
                "Cannot close non-existing connection!",
            ));
        }
        self.session.as_ref().unwrap().stop_listener()?;
        self.session = None;
        Ok(())
    }

    /// Initiates client connection to the ATTP server
    /// If error happened, returns as result
    pub fn connect<'p>(
        &self,
        py: Python<'p>,
        max_retries: usize
    ) -> PyResult<Bound<'p, PyAny>> {
        let mut attempts: usize = 0;

        let host = self.host.clone();
        let port = self.port;
        let limits = self.limits;

        let inner = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            while attempts < max_retries {
                info!("[AttpClientSession] Connect attempt {}/{}", attempts + 1, max_retries);
                let session = inner._connect().await;
                if let Some(session) = session {
                    info!("[AttpClientSession] Connected to {}:{}", host, port);
                    return Ok(AttpClientSession::from_session(
                        host.clone(),
                        port,
                        session,
                        limits
                    ));
                } else {
                    attempts += 1;
                    warn!("[AttpClientSession] Connect attempt {} failed, retrying...", attempts);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
            error!(
                "[AttpClientSession] Failed to connect to the ATTP server at {}:{} after {} attempts!",
                host,
                port,
                max_retries
            );
            Err(PyConnectionError::new_err(format!(
                "Failed to connect to the ATTP server at {}:{} after {} attempts!",
                host, port, max_retries
            )))
        })
    }
}
