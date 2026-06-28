use self::PathSegmentTokensResult::*;
use self::PathTemplateParseResult::*;
use self::UrlPathToken::*;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum UrlPathToken {
    LiteralToken { text: String },
    ParamToken { name: String },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PathTemplate {
    pub tokens: Rc<Vec<Rc<UrlPathToken>>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum PathSegmentTokensResult {
    ParsedSegmentTokens { tokens: Rc<Vec<Rc<UrlPathToken>>> },
    MalformedPathSegment { segment: String, reason: String },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum PathTemplateParseResult {
    ParsedPathTemplate {
        template: Rc<PathTemplate>,
    },
    MalformedPathTemplate {
        raw: String,
        segment: String,
        reason: String,
    },
}

pub fn parse_path_template(raw: String) -> Rc<PathTemplateParseResult> {
    {
        let path_only = match Rc::new(
            raw.clone()
                .split(&"?".to_string())
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        )
        .first()
        .cloned()
        {
            Some(p) => p.clone(),
            None => raw.clone(),
        };
        let segments = Rc::new({
            let mut __result = Vec::new();
            for s in Rc::new(
                path_only
                    .split(&"/".to_string())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            )
            .iter()
            .cloned()
            {
                if (s.clone().as_str() != "".to_string().as_str()) {
                    __result.push(s);
                }
            }
            __result
        });
        match segments.clone().first().cloned() {
            None => Rc::new(PathTemplateParseResult::ParsedPathTemplate {
                template: Rc::new(PathTemplate {
                    tokens: Rc::new(vec![]),
                }),
            }),
            Some(first_seg) => match (*parse_segment_tokens(first_seg.clone())).clone() {
                PathSegmentTokensResult::MalformedPathSegment {
                    segment: s,
                    reason: r,
                    ..
                } => Rc::new(PathTemplateParseResult::MalformedPathTemplate {
                    raw: raw.clone(),
                    segment: s.clone(),
                    reason: r.clone(),
                }),
                PathSegmentTokensResult::ParsedSegmentTokens {
                    tokens: first_tokens,
                    ..
                } => {
                    let parsed = Rc::new(
                        segments
                            .clone()
                            .iter()
                            .cloned()
                            .skip(1 as usize)
                            .collect::<Vec<_>>(),
                    )
                    .iter()
                    .cloned()
                    .fold(
                        Rc::new(PathTemplateParseResult::ParsedPathTemplate {
                            template: Rc::new(PathTemplate {
                                tokens: first_tokens.clone(),
                            }),
                        }),
                        |acc: Rc<PathTemplateParseResult>, seg: String| match (*acc.clone()).clone()
                        {
                            PathTemplateParseResult::MalformedPathTemplate { .. } => acc.clone(),
                            PathTemplateParseResult::ParsedPathTemplate {
                                template: path, ..
                            } => match (*parse_segment_tokens(seg.clone())).clone() {
                                PathSegmentTokensResult::MalformedPathSegment {
                                    segment: s,
                                    reason: r,
                                    ..
                                } => Rc::new(PathTemplateParseResult::MalformedPathTemplate {
                                    raw: raw.clone(),
                                    segment: s.clone(),
                                    reason: r.clone(),
                                }),
                                PathSegmentTokensResult::ParsedSegmentTokens {
                                    tokens: seg_tokens,
                                    ..
                                } => Rc::new(PathTemplateParseResult::ParsedPathTemplate {
                                    template: Rc::new(PathTemplate {
                                        tokens: v1_rt::concat(
                                            path.tokens.clone(),
                                            seg_tokens.clone(),
                                        ),
                                    }),
                                }),
                            },
                        },
                    );
                    parsed
                }
            },
        }
    }
}

pub fn parse_segment_tokens(seg: String) -> Rc<PathSegmentTokensResult> {
    if !v1_rt::contains(seg.clone(), "{".to_string()) {
        if v1_rt::contains(seg.clone(), "}".to_string()) {
            Rc::new(PathSegmentTokensResult::MalformedPathSegment {
                segment: seg.clone(),
                reason: "stray closing brace".to_string(),
            })
        } else {
            Rc::new(PathSegmentTokensResult::ParsedSegmentTokens {
                tokens: Rc::new(vec![Rc::new(UrlPathToken::LiteralToken {
                    text: seg.clone(),
                })]),
            })
        }
    } else {
        {
            let before_and_rest = Rc::new(
                seg.clone()
                    .split(&"{".to_string())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            );
            if ((before_and_rest.clone().len() as i64) != 2) {
                return Rc::new(PathSegmentTokensResult::MalformedPathSegment {
                    segment: seg.clone(),
                    reason: "multiple opening braces in one segment".to_string(),
                });
            }
            let prefix = match before_and_rest.clone().first().cloned() {
                Some(p) => p.clone(),
                None => {
                    return Rc::new(PathSegmentTokensResult::MalformedPathSegment {
                        segment: seg.clone(),
                        reason: "internal: missing prefix after opening-brace split".to_string(),
                    })
                }
            };
            let after_open = match before_and_rest.clone().get(1 as usize).cloned() {
                Some(r) => r.clone(),
                None => {
                    return Rc::new(PathSegmentTokensResult::MalformedPathSegment {
                        segment: seg.clone(),
                        reason: "internal: missing tail after opening-brace split".to_string(),
                    })
                }
            };
            let name_and_suffix = Rc::new(
                after_open
                    .split(&"}".to_string())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            );
            if (v1_rt::contains(prefix.clone(), "}".to_string())
                || ((name_and_suffix.clone().len() as i64) != 2))
            {
                return Rc::new(PathSegmentTokensResult::MalformedPathSegment {
                    segment: seg.clone(),
                    reason: "missing closing brace or extra closing brace".to_string(),
                });
            }
            let param_name = match name_and_suffix.clone().first().cloned() {
                Some(p) => p.clone(),
                None => {
                    return Rc::new(PathSegmentTokensResult::MalformedPathSegment {
                        segment: seg.clone(),
                        reason: "internal: missing parameter name after closing-brace split"
                            .to_string(),
                    })
                }
            };
            let suffix = match name_and_suffix.clone().get(1 as usize).cloned() {
                Some(s) => s.clone(),
                None => {
                    return Rc::new(PathSegmentTokensResult::MalformedPathSegment {
                        segment: seg.clone(),
                        reason: "internal: missing suffix after closing-brace split".to_string(),
                    })
                }
            };
            let prefix_tokens = if (prefix.clone().as_str() != "".to_string().as_str()) {
                Rc::new(vec![Rc::new(UrlPathToken::LiteralToken {
                    text: prefix.clone(),
                })])
            } else {
                Rc::new(vec![])
            };
            let param_tokens = if (param_name.clone().as_str() != "".to_string().as_str()) {
                Rc::new(vec![Rc::new(UrlPathToken::ParamToken {
                    name: param_name.clone(),
                })])
            } else {
                Rc::new(vec![])
            };
            let suffix_tokens = if (suffix.clone().as_str() != "".to_string().as_str()) {
                Rc::new(vec![Rc::new(UrlPathToken::LiteralToken {
                    text: suffix.clone(),
                })])
            } else {
                Rc::new(vec![])
            };
            if (((param_name.clone().as_str() == "".to_string().as_str())
                || v1_rt::contains(suffix.clone(), "{".to_string()))
                || v1_rt::contains(suffix.clone(), "}".to_string()))
            {
                Rc::new(PathSegmentTokensResult::MalformedPathSegment {
                    segment: seg.clone(),
                    reason: "invalid parameter segment structure".to_string(),
                })
            } else {
                Rc::new(PathSegmentTokensResult::ParsedSegmentTokens {
                    tokens: v1_rt::concat(
                        v1_rt::concat(prefix_tokens, param_tokens),
                        suffix_tokens,
                    ),
                })
            }
        }
    }
}

pub fn has_path_params(template: Rc<PathTemplate>) -> bool {
    {
        let mut __found = false;
        for t in template.tokens.clone().iter().cloned() {
            if match (*t.clone()).clone() {
                UrlPathToken::ParamToken { .. } => true,
                UrlPathToken::LiteralToken { .. } => false,
            } {
                __found = true;
                break;
            }
        }
        __found
    }
}

pub fn last_path_param(template: Rc<PathTemplate>) -> Option<String> {
    {
        let params = Rc::new({
            let mut __result = Vec::new();
            for t in template.tokens.clone().iter().cloned() {
                if match (*t.clone()).clone() {
                    UrlPathToken::ParamToken { .. } => true,
                    UrlPathToken::LiteralToken { .. } => false,
                } {
                    __result.push(t);
                }
            }
            __result
        });
        match params.last().cloned() {
            Some(tok) => match (*tok.clone()).clone() {
                UrlPathToken::ParamToken { name: n, .. } => Some(n.clone()),
                UrlPathToken::LiteralToken { .. } => None,
            },
            None => None,
        }
    }
}
