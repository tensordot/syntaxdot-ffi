use std::ops::Deref;

use conllu::graph::Sentence;
use conllu::token::{Token, Tokens};
use ffi_support::{implement_into_ffi_by_delegation, implement_into_ffi_by_protobuf};

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/syntaxdot.sentence.rs"));
}

pub struct Sentences(pub Vec<Sentence>);

impl Deref for Sentences {
    type Target = [Sentence];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&Token> for proto::Token {
    fn from(token: &Token) -> Self {
        proto::Token {
            form: token.form().to_string(),
            lemma: token
                .lemma()
                .map(ToString::to_string)
                .unwrap_or_else(String::new),
            upos: token
                .upos()
                .map(ToString::to_string)
                .unwrap_or_else(String::new),
            xpos: token
                .xpos()
                .map(ToString::to_string)
                .unwrap_or_else(String::new),
            features: token
                .features()
                .iter()
                .map(|(attr, val)| (attr.clone(), val.clone()))
                .collect(),
            head: Default::default(),
            relation: Default::default(),
            misc: token
                .misc()
                .iter()
                .map(|(attr, val)| (attr.clone(), val.to_owned().unwrap_or_else(String::new)))
                .collect(),
        }
    }
}

impl From<proto::Token> for Token {
    fn from(token: proto::Token) -> Self {
        // Ignore every other layer for now...
        Token::new(token.form)
    }
}

impl From<proto::Sentence> for Sentence {
    fn from(sentence: proto::Sentence) -> Self {
        sentence.tokens.into_iter().map(Into::into).collect()
    }
}

impl From<Sentence> for proto::Sentence {
    fn from(sentence: Sentence) -> Self {
        let mut tokens = sentence
            .tokens()
            .map(proto::Token::from)
            .collect::<Vec<_>>();

        // Add dependency edges to the tokens.
        let dep_graph = sentence.dep_graph();
        for dependent in 1..sentence.len() {
            if let Some(dep_triple) = dep_graph.head(dependent) {
                tokens[dependent - 1].head = dep_triple.head() as i32;
                tokens[dependent - 1].relation = dep_triple
                    .relation()
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(String::new);
            }
        }

        proto::Sentence { tokens }
    }
}

impl From<proto::Sentences> for Sentences {
    fn from(sentences: proto::Sentences) -> Self {
        Sentences(
            sentences
                .sentences
                .into_iter()
                .map(|sentence| sentence.into())
                .collect(),
        )
    }
}

impl From<Sentences> for proto::Sentences {
    fn from(sentences: Sentences) -> Self {
        proto::Sentences {
            sentences: sentences.0.into_iter().map(Into::into).collect(),
        }
    }
}

implement_into_ffi_by_protobuf!(proto::Sentences);
implement_into_ffi_by_delegation!(Sentences, proto::Sentences);
