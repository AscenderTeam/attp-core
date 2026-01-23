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

use pyo3::{pyclass, pymethods};
use log::{debug, trace};

use crate::attp::shared::command::{AttpCommand, AttpMessage};

#[derive(Clone)]
#[pyclass]
pub struct PyAttpMessage {
    #[pyo3(get)]
    route_id: u16,

    #[pyo3(get)]
    command_type: AttpCommand,
    
    #[pyo3(get)]
    correlation_id: Option<[u8; 16]>,
    
    #[pyo3(get)]
    payload: Option<Vec<u8>>,
    
    #[pyo3(get)]
    version: [u8;2],
}

#[pymethods]
impl PyAttpMessage {
    #[new]
    pub fn new(route_id: u16, command_type: AttpCommand, correlation_id: Option<[u8; 16]>, payload: Option<Vec<u8>>, version: [u8;2]) -> Self {
        debug!(
            "[PyAttpMessage] New message route_id={}, command={:?}, has_payload={}",
            route_id,
            command_type,
            payload.is_some()
        );
        Self {
            route_id,
            command_type,
            correlation_id,
            payload,
            version
        }
    }

    #[getter]
    pub fn bytes(&self) -> Vec<u8> {
        trace!(
            "[PyAttpMessage] Serializing to bytes route_id={}, command={:?}",
            self.route_id,
            self.command_type
        );
        let attp_message = self.to_attp();
        attp_message.to_bytes()
    }
}


impl PyAttpMessage {
    pub fn from_attp(attp_message: &AttpMessage) -> Self {
        let object = attp_message.clone();
        Self {
            route_id: object.route_id,
            command_type: object.command_type,
            correlation_id: object.correlation_id,
            payload: object.payload,
            version: object.version
        }
    }

    pub fn to_attp(&self) -> AttpMessage {
        let py_instance = self.clone();
        AttpMessage {
            route_id: py_instance.route_id,
            command_type: py_instance.command_type,
            correlation_id: py_instance.correlation_id,
            payload: py_instance.payload,
            version: py_instance.version
        }
    }
}
