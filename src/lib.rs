use ffi_support::{
    define_handle_map_deleter, ByteBuffer, ConcurrentHandleMap, ExternError, FfiStr,
};
use lazy_static::lazy_static;
use tch::Device;

mod annotator;
use annotator::Annotator;

mod error;
use error::AnnotatorError;

pub mod sentences;

lazy_static! {
    static ref ANNOTATORS: ConcurrentHandleMap<Annotator> = ConcurrentHandleMap::new();
}

define_handle_map_deleter!(ANNOTATORS, syntaxdot_annotator_free);

unsafe fn get_buffer<'a>(data: *const u8, len: i32) -> &'a [u8] {
    assert!(len >= 0, "Bad buffer len: {}", len);
    if len == 0 {
        &[]
    } else {
        assert!(!data.is_null(), "Unexpected null data pointer");
        std::slice::from_raw_parts(data, len as usize)
    }
}

/// Annotate the given sentences.
///
/// # Safety
///
/// Safe use of this function requires a valid pointer `sentences_data` and
/// a correct length `sentences_data_len`.
#[no_mangle]
pub unsafe extern "C" fn syntaxdot_annotator_annotate(
    handle: u64,
    sentences_data: *const u8,
    sentences_data_len: i32,
    err: &mut ExternError,
) -> ByteBuffer {
    ANNOTATORS.call_with_result(err, handle, |annotator| -> Result<_, ExternError> {
        let buffer = get_buffer(sentences_data, sentences_data_len);
        let sentences: sentences::proto::Sentences =
            prost::Message::decode(buffer).map_err(AnnotatorError::ProtobufDecodeError)?;
        let sentences: sentences::Sentences = sentences.into();
        annotator.annotate_sentences(&sentences, 32)?;
        Ok(sentences)
    })
}

/// Load a syntaxdot annotator.
#[no_mangle]
pub extern "C" fn syntaxdot_annotator_load(config_path: FfiStr<'_>, err: &mut ExternError) -> u64 {
    ANNOTATORS.insert_with_result(err, || -> Result<Annotator, ExternError> {
        Annotator::load(Device::Cpu, config_path.as_str()).map_err(Into::into)
    })
}

#[cfg(feature = "model-tests")]
#[cfg(test)]
mod tests {
    use std::env;
    use std::ffi::CString;

    use ffi_support::{ErrorCode, ExternError, FfiStr};

    use crate::{syntaxdot_annotator_free, syntaxdot_annotator_load};

    #[test]
    fn model_can_be_loaded() {
        let model_config_path = env::var("MODEL_CONFIG").unwrap();
        let mut err = ExternError::default();
        let config_path = CString::new(model_config_path.as_str()).unwrap();
        let handle = syntaxdot_annotator_load(FfiStr::from_cstr(&config_path), &mut err);
        assert_eq!(err.get_code(), ErrorCode::SUCCESS);

        let mut err = ExternError::default();
        syntaxdot_annotator_free(handle, &mut err);
        assert_eq!(err.get_code(), ErrorCode::SUCCESS);

        let mut err = ExternError::default();
        syntaxdot_annotator_free(handle, &mut err);
        assert_eq!(err.get_code(), ErrorCode::INVALID_HANDLE);
    }
}
