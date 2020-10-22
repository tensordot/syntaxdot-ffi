use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::path::Path;

use conllu::graph::Sentence;
use syntaxdot::config::{Config, PretrainConfig, TomlRead};
use syntaxdot::encoders::Encoders;
use syntaxdot::input::{SentenceWithPieces, Tokenize};
use syntaxdot::model::bert::BertModel;
use syntaxdot::tagger::Tagger;
use tch::nn::VarStore;
use tch::Device;

use crate::AnnotatorError;

/// A wrapper of `Tagger` that is `Send + Sync`.
///
/// Tensors are not thread-safe in the general case, but
/// multi-threaded use is safe if no (in-place) modifications are
/// made:
///
/// https://discuss.pytorch.org/t/is-evaluating-the-network-thread-safe/37802
struct TaggerWrap(Tagger);

unsafe impl Send for TaggerWrap {}

unsafe impl Sync for TaggerWrap {}

impl Deref for TaggerWrap {
    type Target = Tagger;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Annotator {
    tagger: TaggerWrap,
    tokenizer: Box<dyn Tokenize>,
}

impl Annotator {
    pub fn load<P>(device: Device, config_path: P) -> Result<Self, AnnotatorError>
    where
        P: AsRef<Path>,
    {
        let r = BufReader::new(File::open(&config_path).map_err(|err| {
            AnnotatorError::IOError(
                format!(
                    "Cannot open syntaxdot config file `{}`",
                    config_path.as_ref().to_string_lossy()
                ),
                err,
            )
        })?);
        let mut config = Config::from_toml_read(r)?;
        config.relativize_paths(config_path)?;

        let encoders = load_encoders(&config)?;
        let tokenizer = load_tokenizer(&config)?;
        let pretrain_config = load_pretrain_config(&config)?;

        let mut vs = VarStore::new(device);

        let model = BertModel::new(
            vs.root(),
            &pretrain_config,
            &encoders,
            0.0,
            config.model.position_embeddings.clone(),
        )?;

        vs.load(&config.model.parameters)?;

        vs.freeze();

        let tagger = Tagger::new(device, model, encoders);

        Ok(Annotator {
            tagger: TaggerWrap(tagger),
            tokenizer,
        })
    }

    pub fn annotate_sentences(
        &self,
        sentences: impl IntoIterator<Item = Sentence>,
        batch_size: usize,
    ) -> Result<Vec<SentenceWithPieces>, AnnotatorError> where {
        let mut sentences_with_pieces = sentences
            .into_iter()
            .map(|s| self.tokenizer.tokenize(s))
            .collect::<Vec<_>>();

        // Sort sentences by length.
        let mut sent_refs: Vec<_> = sentences_with_pieces.iter_mut().collect();
        sent_refs.sort_unstable_by_key(|s| s.pieces.len());

        // Split in batches, tag, and merge results.
        for batch in sent_refs.chunks_mut(batch_size) {
            self.tagger.tag_sentences(batch)?;
        }

        Ok(sentences_with_pieces)
    }
}

pub fn load_pretrain_config(config: &Config) -> Result<PretrainConfig, AnnotatorError> {
    Ok(config.model.pretrain_config()?)
}

fn load_encoders(config: &Config) -> Result<Encoders, AnnotatorError> {
    let f = File::open(&config.labeler.labels).map_err(|err| {
        AnnotatorError::IOError(
            format!("Cannot open label file: {}", config.labeler.labels),
            err,
        )
    })?;

    Ok(serde_yaml::from_reader(&f)
        .map_err(|err| AnnotatorError::LoadEncodersError(config.labeler.labels.clone(), err))?)
}

pub fn load_tokenizer(config: &Config) -> Result<Box<dyn Tokenize>, AnnotatorError> {
    Ok(config.tokenizer()?)
}
