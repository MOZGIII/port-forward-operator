//! Validate operation.

use pcp_proto as protocol;

pub type ValidateError = ValidateOpcodeDataError;

#[derive(Debug, thiserror::Error)]
pub enum ValidateOpcodeDataError {
    #[error("opcode mismatch")]
    OpcodeMismatch,
    #[error("map data: {0}")]
    MapData(ValidateMapDataError),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateMapDataError {
    #[error("nonce mismatch: expected {expected:?} got {got:?}")]
    NonceMismatch { expected: [u8; 12], got: [u8; 12] },
    #[error("protocol mismatch: expected {expected:?} got {got:?}")]
    ProtocolMismatch { expected: u8, got: u8 },
    #[error("local port mismatch: expected {expected:?} got {got:?}")]
    LocalPortMismatch { expected: u16, got: u16 },
}

pub fn validate(
    request: &protocol::Request,
    response: &protocol::Response,
) -> Result<(), ValidateOpcodeDataError> {
    validate_opcode_data(&request.opcode_data, &response.data)
}

pub fn validate_opcode_data(
    request: &protocol::OpcodeData,
    response: &protocol::OpcodeData,
) -> Result<(), ValidateOpcodeDataError> {
    match (request, response) {
        (protocol::OpcodeData::Announce, protocol::OpcodeData::Announce) => Ok(()),
        (protocol::OpcodeData::MapData(request), protocol::OpcodeData::MapData(response)) => {
            validate_map_data(request, response).map_err(ValidateOpcodeDataError::MapData)
        }
        _ => Err(ValidateOpcodeDataError::OpcodeMismatch),
    }
}

pub fn validate_map_data(
    request: &protocol::MapData,
    response: &protocol::MapData,
) -> Result<(), ValidateMapDataError> {
    if request.nonce != response.nonce {
        return Err(ValidateMapDataError::NonceMismatch {
            expected: request.nonce,
            got: response.nonce,
        });
    }

    if request.protocol != response.protocol {
        return Err(ValidateMapDataError::ProtocolMismatch {
            expected: request.protocol,
            got: response.protocol,
        });
    }

    if request.local_port != response.local_port {
        return Err(ValidateMapDataError::LocalPortMismatch {
            expected: request.local_port,
            got: response.local_port,
        });
    }

    Ok(())
}
