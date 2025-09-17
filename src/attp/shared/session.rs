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

use std::{sync::Arc};

use pyo3::{
    Bound, Py, PyAny, PyErr, PyResult, Python,
    exceptions::PyOSError,
    pyclass, pymethods,
    types::{PyAnyMethods, PyList},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::{Mutex, mpsc, watch},
};

use bytes::{Buf, BytesMut};

use crate::{
    attp::shared::{command::{AttpCommand, AttpMessage}, errors::DecodeError, pyattp_message::PyAttpMessage},
    config::Limits,
};

#[derive(Debug, Clone)]
#[pyclass]
pub struct Session {
    #[pyo3(get)]
    pub session_id: String,
    #[pyo3(get)]
    pub peername: String,
    #[pyo3(get)]
    pub limits: Limits,
    pub shutdown_tx: Option<watch::Sender<bool>>,

    pub event_receivers: Arc<std::sync::Mutex<Vec<Py<PyAny>>>>,

    pub reader: Arc<Mutex<ReadHalf<TcpStream>>>,
    pub message_reader: Option<Arc<Mutex<mpsc::Receiver<Vec<AttpMessage>>>>>,
    // writer_lock: Mutex<()>,
    message_writer: Option<mpsc::Sender<Vec<AttpMessage>>>,
    buf: BytesMut,
    tx: mpsc::Sender<Vec<u8>>,

    shutdown_rx: Option<watch::Receiver<bool>>,
}

impl Session {
    pub fn new(socket: TcpStream, session_id: String, limits: Limits) -> Self {
        let peername = socket
            .peer_addr()
            .map(|p| p.to_string())
            .unwrap_or_default();

        let (reader, writer) = tokio::io::split(socket);

        let tx = Session::run_writer_task(writer);

        return Self {
            tx,
            reader: Arc::new(Mutex::new(reader)),
            session_id,
            peername,
            limits,
            event_receivers: Arc::new(std::sync::Mutex::new(Vec::new())),
            buf: BytesMut::new(),
            message_writer: None,
            message_reader: None,
            shutdown_rx: None,
            shutdown_tx: None,
        };
    }

    pub fn start_listening(&mut self) {
        let (message_writer, message_listener) = mpsc::channel::<Vec<AttpMessage>>(1040);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        self.message_writer = Some(message_writer);
        self.shutdown_rx = Some(shutdown_rx);
        self.shutdown_tx = Some(shutdown_tx);
        self.message_reader = Some(Arc::new(Mutex::new(message_listener)));
    }

    fn run_writer_task(mut writer: WriteHalf<TcpStream>) -> mpsc::Sender<Vec<u8>> {
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(3040);

        tokio::spawn(async move {
            while let Some(buf) = rx.recv().await {
                if let Err(e) = writer.write_all(&buf).await {
                    eprintln!("write error: {e}");
                    break;
                }
            }
        });

        return tx;
    }

    /// Listens for incoming buff bytes and processses them automatically
    /// Handles out messages before and forward passes them to python via FFI boundry
    pub async fn listen(
        &mut self,
        max_output: usize,
        max_bytes: usize
    ) -> Result<(), String> {
        let mut shutdown_rx = self.shutdown_rx.clone().ok_or_else(|| {
            return "Cannot start listener, make sure you initialized shutdown writer by executing Session.start_listening(...)";
        })?;

        loop {
            // Lock the reader only for the duration of the read, so the borrow ends before calling self.read_batch
            let read_result = {
                let mut reader_guard = self.reader.lock().await;
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        self.message_writer.take();
                        self.shutdown_rx.take();
                        break;
                    },
                    read_buf = reader_guard.read_buf(&mut self.buf) => {
                        match read_buf {
                            Ok(n) => n,
                            Err(e) => {
                                eprintln!("Error reading from socket: {}", e);
                                return Err(e.to_string()); // Выходим с ошибкой
                            }
                        }
                    },
                }
            };

            if read_result == 0 {
                // Connection closed
                println!("Connection closed by peer.");
                self.message_writer.take();
                self.shutdown_rx.take();
                break;
            }

            match self.read_batch(max_output, max_bytes) {
                Ok(r) => {
                    println!("Received a messages from ATTP client: {}", r.len());

                    if !self.message_writer.is_none() {
                        let _ = self.message_writer.as_ref().unwrap().send(r).await;
                    }
                }
                Err(e) => {
                    eprintln!("Errors with decoding: {}", e);

                    let _ = self
                        ._send(AttpMessage::new(1u16, 3, None, None, [0, 1]))
                        .await
                        .ok();
                }
            }
        }
        Ok(())
    }

    fn read_batch(
        &mut self,
        max_output: usize,
        max_bytes: usize
    ) -> Result<Vec<AttpMessage>, DecodeError> {
        let mut out: Vec<AttpMessage> = Vec::with_capacity(max_output);
        let mut consumed = 0usize;

        self.parse_batch(&mut out, max_output, max_bytes, &mut consumed)?;

        Ok(out)
    }

    fn parse_batch(
        &mut self,
        out: &mut Vec<AttpMessage>,
        max_count: usize,
        max_bytes: usize,
        consumed: &mut usize,
    ) -> Result<(), DecodeError> {
        loop {
            if out.len() >= max_count || *consumed >= max_bytes {
                break Ok(());
            }
            match AttpMessage::from_bytes(&self.buf, &self.limits) {
                Ok((msg, used)) => {
                    // advance buffer by used bytes
                    self.buf.advance(used);
                    *consumed += used;
                    out.push(msg);
                }
                Err(DecodeError::Incomplete) => break Ok(()), // need more bytes
                Err(e) => {
                    // hard parse error; you probably want to close the connection
                    return Err(e);
                }
            }
        }
    }

    pub async fn _send(&self, frame: AttpMessage) -> tokio::io::Result<()> {
        // let _guard = self.writer_lock.lock().await; || We don't need this anymore lol >_0

        let bytes = frame.to_bytes();

        if let Err(e) = self.tx.send(bytes).await {
            eprintln!("Failed to send message {}", e);
        }

        Ok(())
    }

    /// Borrow checker hits hard
    pub async fn _send_batch(&self, frames: Vec<AttpMessage>) -> tokio::io::Result<()> {
        let mut all_bytes = Vec::new();
        for frame in frames {
            let bytes = frame.to_bytes();
            all_bytes.extend_from_slice(&bytes);
        }

        if let Err(e) = self.tx.send(all_bytes).await {
            eprintln!("Failed to send message {}", e);
        }

        Ok(())
    }
}

#[pymethods]
impl Session {
    // #[new]
    // pub fn new() -> Self {
    //     Self {
    //         event_receivers: Vec::new(),
    //         session_id: None,
    //         peername: None,
    //         inner: None,
    //         message_listener: None,
    //         shutdown_tx: None
    //     }
    // }

    /// Adds any callback into event handler
    /// `pysession.add_event_handler(...)``
    /// NOTE: This method isn't awaitable, callback also should be asynchronous!
    pub fn add_event_handler(&mut self, callback: Py<PyAny>) {
        self.event_receivers.lock().unwrap().push(callback);
    }

    pub fn start_listener<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let mut inner = self.clone();

        self.start_listening(); // Damn, almost forgot to call this fn, I would get cracked if wouldn't call this lol >_0;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner
            .listen(3000, 3000)
            .await
            .map_err(|e| PyOSError::new_err(e))
        })
    }

    pub fn start_handler<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = Arc::new(Mutex::new(self.clone()));

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = inner.lock().await;
            guard.handle_messages().await;
            Ok(())
        })
    }

    /// Signals session manager to K.O. the listener...
    /// Btw raises error if session isn't listening at all... >=)
    pub fn stop_listener(&self) -> PyResult<()> {
        if self.shutdown_tx.is_none() {
            return Err(PyOSError::new_err("Session isn't running at all!"));
        }
        self.shutdown_tx
            .as_ref()
            .unwrap()
            .send(true)
            .map_err(|e| PyOSError::new_err(e.to_string()))?;
        Ok(())
    }

    /// Python: `await pysession.send(frame)`
    /// Do NOT mark async. Return an awaitable instead.
    pub fn send<'a>(&self, py: Python<'a>, frame: Bound<PyAny>) -> PyResult<Bound<'a, PyAny>> {
        // extract BEFORE creating the future, so we don’t hold the GIL inside await
        let msg: PyAttpMessage = frame.extract()?;
        let inner = self.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner._send(msg.to_attp()).await.map_err(to_py_io_err)?;
            Ok(())
        })
    }

    /// Python: `await pysession.send_batch([f1, f2, ...])`
    pub fn send_batch<'a>(
        &self,
        py: Python<'a>,
        frames: Bound<PyAny>,
    ) -> PyResult<Bound<'a, PyAny>> {
        // Accept list/tuple iterable; convert to Vec<PyAttpMessage>
        let frames_vec: Vec<PyAttpMessage> = if let Ok(list) = frames.downcast::<PyList>() {
            list.extract()?
        } else {
            frames.extract()? // fallback: any iterable of PyAttpMessage
        };
        // SHUT UP BORROW CHEKCER!!!
        let inner = self.clone();

        let attpified_frames = Vec::from_iter(frames_vec.iter().map(|i| i.to_attp()));
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner
                ._send_batch(attpified_frames)
                .await
                .map_err(to_py_io_err)?;
            Ok(())
        })
    }

    pub fn disconnect<'a>(&self, py: Python<'a>) -> PyResult<()> {
        if self.shutdown_tx.is_none() {
            return Err(PyOSError::new_err("Session isn't running at all!"));
        }

        let inner = self.clone();
        if let Err(err) = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let frame = PyAttpMessage::new(0, AttpCommand::DISCONNECT, None, None, [0, 1]);

            inner._send(frame.to_attp()).await.map_err(to_py_io_err)?;
            Ok(())
        }) {
            eprintln!("Failed to send disconnect frame: {}", err);
        }

        self.shutdown_tx
            .as_ref()
            .unwrap()
            .send(true)
            .map_err(|e| PyOSError::new_err(e.to_string()))?;
        Ok(())
    }
}

impl Session {
    pub async fn handle_messages(&mut self) {
        if self.message_reader.is_none() {
            eprint!("Cannot start message handler, make sure you initialized message reader by executing Session.start_listening(...)");
            return;
        }

        if let Some(receiver) = &mut self.message_reader {
            let mut receiver_guard = receiver.lock().await;
            loop {
                if let Some(received) = receiver_guard.recv().await {
                    Python::attach(|py| {
                        println!("Received {} messages to forward to Python", received.len());
                        println!("Message frame example: {:?}", received.get(0));
                        let py_attp_messages =
                            PyList::new(py, received.iter().map(|i| PyAttpMessage::from_attp(i)))
                                .unwrap_or_else(|_| PyList::empty(py));
                        for event_receiver in self.event_receivers.lock().unwrap().iter() {
                            let bound_receiver = event_receiver.bind(py).to_owned();
                            if let Ok(awaitable) = bound_receiver.call1((py_attp_messages.clone(),))
                            {
                                pyo3_async_runtimes::tokio::into_future(awaitable).ok();
                            }
                        }
                    });
                }
            }
        }
    }
}

fn to_py_io_err(e: std::io::Error) -> PyErr {
    PyOSError::new_err(e.to_string())
}
