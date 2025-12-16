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

use num_enum::{IntoPrimitive, TryFromPrimitive};
use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyResult};
use std::{convert::TryFrom, hash::Hash};

use crate::{attp::{shared::errors::DecodeError, utils::{read_var_u32, take_exact, write_var_u32}}, config::{Limits, MAGIC}};


#[pyclass]
#[derive(Clone, Copy, Hash, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum AttpCommand {
    CALL = 0,
    ACK = 1,
    EMIT = 2,
    ERR = 3,

    AUTH = 4,
    READY = 5,
    PING = 6,
    PONG = 7,
    DISCONNECT = 8,
    DEFER = 9,
    STREAMBOS = 10,
    CHUNK = 11,
    STREAMEOS = 12
}


#[pymethods]
impl AttpCommand {
    fn __eq__(&self, other: &AttpCommand) -> bool {
        self.eq(other)
    }

    fn __ne__(&self, other: &AttpCommand) -> bool {
        self.ne(other)
    }

    fn __str__(&self) -> String {
        format!("{:?}", self)
    }
    
    fn __hash__(&self) -> isize {
        // hash by discriminant
        *self as isize
    }

    // fn __richcmp__(&self, other: PyRef<Self>, op: CompareOp) -> bool {
    //     match op {
    //         CompareOp::Eq => self == &*other,
    //         CompareOp::Ne => self != &*other,
    //         _ => false,
    //     }
    // }
}


#[derive(Clone, Debug)]
#[pyclass]
pub struct AttpMessage {
    pub route_id: u16,
    pub command_type: AttpCommand,
    pub correlation_id: Option<[u8; 16]>,
    pub payload: Option<Vec<u8>>,
    pub version: [u8;2]
}


impl PartialEq for AttpMessage {
    fn eq(&self, other: &Self) -> bool {
        if self.correlation_id.is_none() {
            return self.command_type == other.command_type && self.route_id == other.route_id;
        }
        
        self.correlation_id == other.correlation_id
    }
}


impl AttpMessage {
    pub fn new(
        route_id: u16, 
        command_type: u8, 
        correlation_id: Option<[u8; 16]>, 
        payload: Option<Vec<u8>>,
        version: [u8;2]
    ) -> Self {
        let command_type = AttpCommand::try_from(command_type).unwrap_or(AttpCommand::ERR);
        
        Self {
            route_id: route_id,
            command_type: command_type,
            correlation_id: correlation_id,
            payload: payload,
            version: version
        }
    }

    pub fn from_bytes(buf: &[u8], limits: &Limits) -> Result<(Self, usize), DecodeError> {
        let mut off: usize = 0;

        // Handling MAGIC here...
        let magic = take_exact(buf, &mut off, 2)?;
        if magic != MAGIC {
            return Err(DecodeError::BadMagic);
        }

        
        let ver = take_exact(buf, &mut off, 2)?;
        let version = [ver[0], ver[1]];

        // if version[0] == 0 { return Err(DecodeError::BadVersion(version)) }

        
        let cmd_raw = *take_exact(buf, &mut off, 1)?.first().unwrap();
        let command_type = AttpCommand::try_from(cmd_raw).map_err(|_| DecodeError::BadCommand(cmd_raw))?;

        // Route ID (for agenthub route mapping)
        let route_id_bytes = take_exact(buf, &mut off, 2)?;
        let route_id = u16::from_le_bytes([route_id_bytes[0], route_id_bytes[1]]);

        
        let has_correlation = *take_exact(buf, &mut off, 1)?.first().unwrap();

        let correlation_id: Option<[u8; 16]> = if has_correlation != 0 {
            let corr = take_exact(buf, &mut off, 16)?;
            let mut cid = [0u8; 16];
            cid.copy_from_slice(corr);
            
            Some(cid)
        }
        else {
            None
        };

        // Handle payload and its length... Ofc if it's there =) (Yeah lol, there are frames without payloads tbh...)
        let payload_len = read_var_u32(buf, &mut off)? as usize;
        
        if payload_len > limits.max_payload_size {
            return Err(DecodeError::PayloadTooLarge(payload_len, limits.max_payload_size));
        }

        let payload = if payload_len > 0 {
            let raw = take_exact(buf, &mut off, payload_len)?;
            Some(raw.to_vec())
        }
        else {
            None
        };

        // For fail safety, there are a few AttpCommand types that are REQUIRE that correlation ID is specified.
        // We do check on them, for ACK, CALL and ERR
        
        match command_type {
            AttpCommand::ACK | 
            AttpCommand::CALL | 
            AttpCommand::STREAMBOS | 
            AttpCommand::STREAMEOS | 
            AttpCommand::CHUNK => {
                if correlation_id.is_none() {
                    return Err(DecodeError::MissingCorrelation(command_type));
                }
            }
            _ => {}
        }

        Ok((
            AttpMessage {
                route_id,
                command_type,
                correlation_id,
                payload,
                version
            },
            off
        ))

    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 + self.payload.as_ref().map(|p| p.len()).unwrap_or(0));
        
        // magic + version's sizew
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&self.version);

        // binarized command
        out.push(self.command_type.clone() as u8);

        // Form route id
        out.extend_from_slice(&self.route_id.to_le_bytes());

        // Correlation ID, in this part we pass byte 1 to the binary if there's a correlation ID in the AttpMessage
        // Otherwise we pass 0 and skip right to the payload leaving correlation ID alone.
        match self.correlation_id {
            Some(cid) => {
                out.push(1);
                out.extend_from_slice(&cid);
            }
            None => out.push(0)
        }

        let plen = self.payload.as_ref().map(|p| p.len()).unwrap_or(0);

        write_var_u32(&mut out, plen as u32);

        if let Some(p) = &self.payload {
            out.extend_from_slice(p);
        }
        
        out
    }
}


#[pymethods]
impl AttpMessage {
    #[new]
    pub fn python_new(
        route_id: u16, 
        command_type: u8, 
        correlation_id: Option<[u8; 16]>, 
        payload: Option<Vec<u8>>,
        version: [u8;2]
    ) -> Self {
        AttpMessage::new(route_id, command_type, correlation_id, payload, version)
    }
    
    #[staticmethod]
    pub fn get_from_bytes(buf: &[u8], limits: &Limits) -> PyResult<(AttpMessage, usize)> {
        return AttpMessage::from_bytes(buf, limits).map_err(|e| PyValueError::new_err(e.to_string()));
    }

    pub fn convert_to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}