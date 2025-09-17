/// Ascender AgentHub (c) 2025
/// Proprietary software. All rights reserved.
/// See the project license for terms of use.

use thiserror::Error;

use crate::attp::shared::command::AttpCommand;


#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("incomplete frame")]
    Incomplete,
    #[error("bad magic header")]
    BadMagic,
    #[error("unsupported version {0:?}")]
    BadVersion([u8;2]),
    #[error("unknown command {0}")]
    BadCommand(u8),
    #[error("invalid varint length")]
    BadVarint,
    #[error("payload too large: {0} > {1}")]
    PayloadTooLarge(usize, usize),
    #[error("missing correlation id for command {0:?}")]
    MissingCorrelation(AttpCommand),
}

