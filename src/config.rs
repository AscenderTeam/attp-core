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


pub const MAGIC: &[u8;2] = b"AT";

#[derive (Clone, Copy, Debug)]
#[pyclass]
pub struct Limits {
    pub max_payload_size: usize
}

#[pymethods]
impl Limits {
    #[new]
    fn new(max_payload_size: Option<usize>) -> Self {
        Self {
            max_payload_size: if let Some(o) = max_payload_size { o } else { 64 * 1024 },
        }
    }
}