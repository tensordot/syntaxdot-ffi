use ffi_support::{
    define_bytebuffer_destructor, define_handle_map_deleter, define_string_destructor, ByteBuffer,
    ConcurrentHandleMap, ExternError, FfiStr,
};
use lazy_static::lazy_static;
use tch::Device;

mod annotator;
use annotator::Annotator;

mod error;
use error::AnnotatorError;
use std::ffi::CString;
use std::os::raw::c_char;

pub mod sentences;

mod util;

lazy_static! {
    static ref ANNOTATORS: ConcurrentHandleMap<Annotator> = ConcurrentHandleMap::new();
    static ref SYNTAXDOT_VERSION: CString = CString::new(syntaxdot::VERSION).unwrap();
}

define_bytebuffer_destructor!(syntaxdot_free_bytebuffer);
define_handle_map_deleter!(ANNOTATORS, syntaxdot_annotator_free);
define_string_destructor!(syntaxdot_free_string);

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
    batch_size: usize,
    err: &mut ExternError,
) -> ByteBuffer {
    ANNOTATORS.call_with_result(err, handle, |annotator| -> Result<_, ExternError> {
        let buffer = get_buffer(sentences_data, sentences_data_len);
        let sentences: sentences::proto::Sentences =
            prost::Message::decode(buffer).map_err(AnnotatorError::ProtobufDecode)?;
        let sentences: sentences::Sentences = sentences.into();
        let annotated_sentences = annotator
            .annotate_sentences(sentences.0, batch_size)?
            .into_iter()
            .map(|s| s.sentence)
            .collect::<Vec<_>>();
        Ok(sentences::Sentences(annotated_sentences))
    })
}

/// Load a syntaxdot annotator.
#[no_mangle]
pub extern "C" fn syntaxdot_annotator_load(config_path: FfiStr<'_>, err: &mut ExternError) -> u64 {
    ANNOTATORS.insert_with_result(err, || -> Result<Annotator, ExternError> {
        Annotator::load(Device::Cpu, config_path.as_str()).map_err(Into::into)
    })
}

/// Set the number of inter-op threads.
#[no_mangle]
pub extern "C" fn syntaxdot_set_num_interop_threads(n_threads: i32) {
    tch::set_num_interop_threads(n_threads);
}

/// Set the number of intra-op threads.
#[no_mangle]
pub extern "C" fn syntaxdot_set_num_intraop_threads(n_threads: i32) {
    tch::set_num_threads(n_threads);
}

/// Get the syntaxdot version.
///
/// The returned string must not be deallocated.
#[no_mangle]
pub extern "C" fn syntaxdot_version() -> *const c_char {
    SYNTAXDOT_VERSION.as_ptr()
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use ffi_support::{ErrorCode, ExternError, FfiStr};

    use crate::error::error_codes::IO_ERROR;
    use crate::syntaxdot_annotator_load;

    #[test]
    fn model_cannot_be_loaded() {
        let mut err = ExternError::default();
        let config_path = CString::new("/foo/bar/baz").unwrap();
        let _handle = syntaxdot_annotator_load(FfiStr::from_cstr(&config_path), &mut err);
        assert_eq!(err.get_code(), ErrorCode::new(IO_ERROR));
    }
}

#[cfg(feature = "model-tests")]
#[cfg(test)]
mod model_tests {
    use std::env;
    use std::ffi::CString;
    use std::iter::FromIterator;

    use conllu::graph::{DepTriple, Sentence};
    use conllu::token::{Features, Token, TokenBuilder};
    use ffi_support::{ErrorCode, ExternError, FfiStr};
    use pretty_assertions::assert_eq;
    use prost::Message;

    use crate::sentences::{proto, Sentences};
    use crate::{syntaxdot_annotator_annotate, syntaxdot_annotator_free, syntaxdot_annotator_load};

    fn test_sentence_protobuf() -> Vec<u8> {
        let tokens = vec![
            Token::new("Dit"),
            Token::new("is"),
            Token::new("een"),
            Token::new("test"),
            Token::new("."),
        ];
        let sentences = Sentences(vec![Sentence::from_iter(tokens.into_iter())]);
        let sentences = proto::Sentences::from(sentences);
        let mut sentences_proto = Vec::new();
        sentences.encode(&mut sentences_proto).unwrap();
        sentences_proto
    }

    fn test_sentence_check() -> Sentence {
        let mut sentence = Sentence::from_iter(vec![
            TokenBuilder::new("Dit")
                .lemma("dit")
                .upos("PRON")
                .xpos("PRON-aanw")
                .features(Features::from_iter(vec![
                    ("Person".to_string(), "3".to_string()),
                    ("PronType".to_string(), "Dem".to_string()),
                ]))
                .into(),
            TokenBuilder::new("is")
                .lemma("zijn")
                .upos("AUX")
                .xpos("AUX-pv")
                .features(Features::from_iter(vec![
                    ("Number".to_string(), "Sing".to_string()),
                    ("Tense".to_string(), "Pres".to_string()),
                    ("VerbForm".to_string(), "Fin".to_string()),
                ]))
                .into(),
            TokenBuilder::new("een")
                .lemma("een")
                .upos("DET")
                .xpos("DET-onbep")
                .features(Features::from_iter(vec![(
                    "Definite".to_string(),
                    "Ind".to_string(),
                )]))
                .into(),
            TokenBuilder::new("test")
                .lemma("test")
                .upos("NOUN")
                .xpos("NOUN")
                .features(Features::from_iter(vec![
                    ("Gender".to_string(), "Com".to_string()),
                    ("Number".to_string(), "Sing".to_string()),
                ]))
                .into(),
            TokenBuilder::new(".")
                .lemma(".")
                .upos("PUNCT")
                .xpos("PUNCT")
                .into(),
        ]);

        sentence
            .dep_graph_mut()
            .add_deprel(DepTriple::new(0, Some("root"), 4));
        sentence
            .dep_graph_mut()
            .add_deprel(DepTriple::new(4, Some("nsubj"), 1));
        sentence
            .dep_graph_mut()
            .add_deprel(DepTriple::new(4, Some("cop"), 2));
        sentence
            .dep_graph_mut()
            .add_deprel(DepTriple::new(4, Some("det"), 3));
        sentence
            .dep_graph_mut()
            .add_deprel(DepTriple::new(4, Some("punct"), 5));

        sentence
    }

    #[test]
    fn model_can_be_loaded() {
        let model_config_path = format!("{}/sticker.conf", env::var("DUTCH_UD_SMALL").unwrap());

        let mut err = ExternError::default();

        // Check that a model can be loaded.
        let config_path = CString::new(model_config_path.as_str()).unwrap();
        let handle = syntaxdot_annotator_load(FfiStr::from_cstr(&config_path), &mut err);
        assert_eq!(err.get_code(), ErrorCode::SUCCESS);

        // Check that a model can be freed.
        let mut err = ExternError::default();
        syntaxdot_annotator_free(handle, &mut err);
        assert_eq!(err.get_code(), ErrorCode::SUCCESS);

        // A double free should result in an invalid handle error.
        let mut err = ExternError::default();
        syntaxdot_annotator_free(handle, &mut err);
        assert_eq!(err.get_code(), ErrorCode::INVALID_HANDLE);
    }

    #[test]
    fn model_gives_correct_output() {
        let model_config_path = format!("{}/sticker.conf", env::var("DUTCH_UD_SMALL").unwrap());

        let mut err = ExternError::default();

        // Check that a model can be loaded.
        let config_path = CString::new(model_config_path.as_str()).unwrap();
        let handle = syntaxdot_annotator_load(FfiStr::from_cstr(&config_path), &mut err);
        assert_eq!(err.get_code(), ErrorCode::SUCCESS);

        let sentences_proto = test_sentence_protobuf();

        let buffer = unsafe {
            syntaxdot_annotator_annotate(
                handle,
                sentences_proto.as_ptr(),
                sentences_proto.len() as i32,
                32,
                &mut err,
            )
        };
        assert_eq!(err.get_code(), ErrorCode::SUCCESS);

        let annotated_sentences: Sentences =
            proto::Sentences::decode(buffer.as_slice()).unwrap().into();
        assert_eq!(annotated_sentences.0, vec![test_sentence_check()]);

        let mut err = ExternError::default();
        syntaxdot_annotator_free(handle, &mut err);
        assert_eq!(err.get_code(), ErrorCode::SUCCESS);
    }
}
