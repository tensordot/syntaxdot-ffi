use std::io;

use ffi_support::{ErrorCode, ExternError};
use syntaxdot::error::SyntaxDotError;
use syntaxdot_transformers::models::bert::BertError;
use thiserror::Error;

pub mod error_codes {
    pub const BERT_ERROR: i32 = 1;
    pub const IO_ERROR: i32 = 2;
    pub const LOAD_ENCODERS_ERROR: i32 = 3;
    pub const LOAD_PARAMETERS_ERROR: i32 = 4;
    pub const SYNTAXDOT_ERROR: i32 = 5;
    pub const DECODE_PROTOBUF_ERROR: i32 = 6;
}

#[derive(Debug, Error)]
pub enum AnnotatorError {
    #[error("Cannot construct BERT model: {0}")]
    BertError(#[from] BertError),

    #[error("{0}: {1}")]
    IOError(String, io::Error),

    #[error("Cannot deserialize encoders from `{0}`: {1}")]
    LoadEncodersError(String, serde_yaml::Error),

    #[error("Cannot load model parameters: {0}")]
    LoadParametersError(#[from] tch::TchError),

    #[error("Cannot decode protobuf: {0}")]
    ProtobufDecodeError(#[from] prost::DecodeError),

    #[error(transparent)]
    SyntaxDotError(#[from] SyntaxDotError),
}

impl From<&AnnotatorError> for ErrorCode {
    fn from(err: &AnnotatorError) -> Self {
        use AnnotatorError::*;
        match err {
            BertError(_) => ErrorCode::new(error_codes::BERT_ERROR),
            IOError(_, _) => ErrorCode::new(error_codes::IO_ERROR),
            LoadEncodersError(_, _) => ErrorCode::new(error_codes::LOAD_ENCODERS_ERROR),
            LoadParametersError(_) => ErrorCode::new(error_codes::LOAD_PARAMETERS_ERROR),
            ProtobufDecodeError(_) => ErrorCode::new(error_codes::DECODE_PROTOBUF_ERROR),
            SyntaxDotError(_) => ErrorCode::new(error_codes::SYNTAXDOT_ERROR),
        }
    }
}

impl From<AnnotatorError> for ExternError {
    fn from(err: AnnotatorError) -> Self {
        ExternError::new_error((&err).into(), err.to_string())
    }
}
