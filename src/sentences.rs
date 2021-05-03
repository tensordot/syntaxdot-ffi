use std::ops::Deref;

use ffi_support::{implement_into_ffi_by_delegation, implement_into_ffi_by_protobuf};
use udgraph::graph::{DepTriple, Sentence};
use udgraph::token::{Token, Tokens};

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
    fn from(proto_token: proto::Token) -> Self {
        let mut token = Token::new(proto_token.form);

        if !proto_token.lemma.is_empty() {
            token.set_lemma(Some(proto_token.lemma));
        }

        if !proto_token.upos.is_empty() {
            token.set_upos(Some(proto_token.upos));
        }

        if !proto_token.xpos.is_empty() {
            token.set_xpos(Some(proto_token.xpos));
        }

        if !proto_token.features.is_empty() {
            token.set_features(proto_token.features.into_iter().collect());
        }

        if !proto_token.misc.is_empty() {
            token.set_misc(
                proto_token
                    .misc
                    .into_iter()
                    .map(|(k, v)| {
                        if v.is_empty() {
                            (k, None)
                        } else {
                            (k, Some(v))
                        }
                    })
                    .collect(),
            );
        }

        token
    }
}

impl From<proto::Sentence> for Sentence {
    fn from(sentence: proto::Sentence) -> Self {
        let dep_rels: Vec<_> = sentence
            .tokens
            .iter()
            .map(|t| (t.head, t.relation.clone()))
            .collect();

        // Convert tokens.
        let mut sentence: Sentence = sentence.tokens.into_iter().map(Into::into).collect();

        // Add dependency relations.
        for (idx, (head, rel)) in dep_rels.into_iter().enumerate() {
            // CoNLL-U requires HEAD and DEPREL for UD treebanks. So, let's assume
            // that we only have a dependency relation when DEPREL is present.
            if !rel.is_empty() {
                sentence.dep_graph_mut().add_deprel(DepTriple::new(
                    head as usize,
                    Some(rel),
                    idx + 1,
                ));
            }
        }

        sentence
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
