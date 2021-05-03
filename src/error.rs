use std::io;

use ffi_support::{ErrorCode, ExternError};
use syntaxdot::error::SyntaxDotError;
use syntaxdot_transformers::TransformerError;
use thiserror::Error;

pub mod error_codes {
    pub const TRANSFORMER_ERROR: i32 = 1;
    pub const IO_ERROR: i32 = 2;
    pub const LOAD_ENCODERS_ERROR: i32 = 3;
    pub const LOAD_PARAMETERS_ERROR: i32 = 4;
    pub const SYNTAXDOT_ERROR: i32 = 5;
    pub const DECODE_PROTOBUF_ERROR: i32 = 6;
}

#[derive(Debug, Error)]
pub enum AnnotatorError {
    #[error("Cannot construct BERT model: {0}")]
    Transformer(#[from] TransformerError),

    #[error("{0}: {1}")]
    Io(String, io::Error),

    #[error("Cannot deserialize encoders from `{0}`: {1}")]
    LoadEncoders(String, serde_yaml::Error),

    #[error("Cannot load model parameters: {0}")]
    LoadParameters(#[from] tch::TchError),

    #[error("Cannot decode protobuf: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),

    #[error(transparent)]
    SyntaxDot(#[from] SyntaxDotError),
}

impl From<&AnnotatorError> for ErrorCode {
    fn from(err: &AnnotatorError) -> Self {
        use AnnotatorError::*;
        match err {
            Transformer(_) => ErrorCode::new(error_codes::TRANSFORMER_ERROR),
            Io(_, _) => ErrorCode::new(error_codes::IO_ERROR),
            LoadEncoders(_, _) => ErrorCode::new(error_codes::LOAD_ENCODERS_ERROR),
            LoadParameters(_) => ErrorCode::new(error_codes::LOAD_PARAMETERS_ERROR),
            ProtobufDecode(_) => ErrorCode::new(error_codes::DECODE_PROTOBUF_ERROR),
            SyntaxDot(_) => ErrorCode::new(error_codes::SYNTAXDOT_ERROR),
        }
    }
}

impl From<AnnotatorError> for ExternError {
    fn from(err: AnnotatorError) -> Self {
        ExternError::new_error((&err).into(), err.to_string())
    }
}
