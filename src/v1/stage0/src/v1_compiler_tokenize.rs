use self::StringScanResult::*;
pub use crate::extdeps_languages_dag_syntax::dag_keyword_set;
pub use crate::std_types::SourceSpan;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
pub use crate::v1_std_core::make_file_span;
use crate::v1_std_core::TokenShape::{
    ShAnd, ShArrow, ShBang, ShCaret, ShColon, ShComma, ShDot, ShDotDot, ShEof, ShEq, ShEqEq,
    ShFatArrow, ShGe, ShGt, ShIdent, ShKeyword, ShLBrace, ShLBracket, ShLParen, ShLe, ShLitFloat,
    ShLitInt, ShLitStr, ShLt, ShMinus, ShNe, ShNewline, ShNullCoalesce, ShOr, ShPercent, ShPipe,
    ShPipeArrow, ShPlus, ShQuestion, ShRBrace, ShRBracket, ShRParen, ShSlash, ShStar, ShStrBegin,
    ShStrEnd, ShStrMid, ShUnknown,
};
pub use crate::v1_std_core::{Token, TokenShape};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

pub fn is_keyword_text(text: String) -> bool {
    match v1_rt::lookup(&dag_keyword_set(), text) {
        v1_rt::Witness::Holds { value: _, .. } => true,
        v1_rt::Witness::Violates { diagnostic: _, .. } => false,
    }
}

pub fn single_punct() -> Rc<HashMap<String, TokenShape>> {
    thread_local! {
        static CACHED: Rc<HashMap<String, TokenShape>> = {
            let mut __m = HashMap::new();
            __m.insert("(".to_string(), TokenShape::ShLParen);
            __m.insert(")".to_string(), TokenShape::ShRParen);
            __m.insert("[".to_string(), TokenShape::ShLBracket);
            __m.insert("]".to_string(), TokenShape::ShRBracket);
            __m.insert(":".to_string(), TokenShape::ShColon);
            __m.insert(",".to_string(), TokenShape::ShComma);
            __m.insert(".".to_string(), TokenShape::ShDot);
            __m.insert("+".to_string(), TokenShape::ShPlus);
            __m.insert("*".to_string(), TokenShape::ShStar);
            __m.insert("%".to_string(), TokenShape::ShPercent);
            __m.insert("/".to_string(), TokenShape::ShSlash);
            __m.insert("^".to_string(), TokenShape::ShCaret);
            Rc::new(__m)
        };
    }
    CACHED.with(|c: &Rc<HashMap<String, TokenShape>>| c.clone())
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TokenizerState {
    pub pos: i64,
    pub tokens: Rc<Vec<Rc<Token>>>,
    pub interp_depth: Rc<Vec<i64>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TokPos {
    pub pos: i64,
    pub interp_depth: Rc<Vec<i64>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScanResult {
    pub pos: i64,
    pub token: Rc<Token>,
    pub interp_depth: Rc<Vec<i64>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SourceRef {
    pub file: String,
    pub text: String,
    pub source_chars: Rc<Vec<i64>>,
}

pub fn make_token(text: String, span: Rc<SourceSpan>, shape: TokenShape) -> Rc<Token> {
    Rc::new(Token {
        text: text,
        span: span,
        shape: shape,
    })
}

pub fn source_char(source: Rc<SourceRef>, pos: i64) -> String {
    v1_rt::from_code_point(source.source_chars.clone()[(pos) as usize].clone())
}

pub fn source_code_point(source: Rc<SourceRef>, pos: i64) -> i64 {
    {
        let _ = v1_rt::record_source_chars_index_lookup();
        source.source_chars.clone()[(pos) as usize].clone()
    }
}

pub fn source_len(source: Rc<SourceRef>) -> i64 {
    (source.source_chars.clone().len() as i64)
}

pub fn source_substring(source: Rc<SourceRef>, start: i64, end: i64) -> String {
    v1_rt::chars_to_string(&source.source_chars.clone(), start, end)
}

pub fn source_scan_while(
    mut source: Rc<SourceRef>,
    mut start: i64,
    mut pred: impl Fn(i64) -> bool + Clone,
) -> i64 {
    loop {
        if (start.clone() >= source_len(source.clone())) {
            break source_len(source.clone());
        } else {
            if pred(source.source_chars.clone()[(start.clone()) as usize].clone()) {
                {
                    let __tco_0 = (start + 1);
                    start = __tco_0;
                    continue;
                }
            } else {
                break start.clone();
            }
        }
    }
}

pub fn source_skip_ws(mut source: Rc<SourceRef>, mut start: i64) -> i64 {
    loop {
        if (start.clone() >= source_len(source.clone())) {
            break start.clone();
        } else {
            let ch = source.source_chars.clone()[(start.clone()) as usize].clone();
            if ((ch.clone() == 32) || (ch.clone() == 9)) {
                {
                    let __tco_0 = (start + 1);
                    start = __tco_0;
                    continue;
                }
            } else {
                break start.clone();
            }
        }
    }
}

pub fn source_scan_to_eol(mut source: Rc<SourceRef>, mut start: i64) -> i64 {
    loop {
        if (start.clone() >= source_len(source.clone())) {
            break source_len(source.clone());
        } else {
            if (source.source_chars.clone()[(start.clone()) as usize].clone() == 10) {
                break start.clone();
            } else {
                {
                    let __tco_0 = (start + 1);
                    start = __tco_0;
                    continue;
                }
            }
        }
    }
}

pub fn tokenize(source: String, file: String) -> Rc<Vec<Rc<Token>>> {
    {
        let c = Rc::new(source.clone().chars().map(|c| c as i64).collect::<Vec<_>>());
        let src = Rc::new(SourceRef {
            file: file,
            text: source.clone(),
            source_chars: c,
        });
        let initial = Rc::new(TokPos {
            pos: 0,
            interp_depth: Rc::new(vec![]),
        });
        let final_state = tokenize_loop(
            src.clone(),
            Rc::new(vec![]),
            initial,
            (source_len(src.clone()) + 1),
        );
        let eof_span = make_file_span(
            src.file.clone(),
            final_state.pos.clone(),
            final_state.pos.clone(),
        );
        v1_rt::rc_list_push(
            final_state.tokens.clone(),
            make_token("".to_string(), eof_span, TokenShape::ShEof),
        )
    }
}

pub fn scan_next_token(source: Rc<SourceRef>, pos: Rc<TokPos>) -> Rc<ScanResult> {
    {
        let ch = source_code_point(source.clone(), pos.pos.clone());
        if (ch.clone() == 10) {
            return Rc::new(ScanResult {
                pos: (pos.pos.clone() + 1),
                token: make_token(
                    "\n".to_string(),
                    make_file_span(source.file.clone(), pos.pos.clone(), (pos.pos.clone() + 1)),
                    TokenShape::ShNewline,
                ),
                interp_depth: pos.interp_depth.clone(),
            });
        }
        if ((ch.clone() == 125) && ((pos.interp_depth.clone().len() as i64) > 0)) {
            {
                let top = pos.interp_depth.clone().last().cloned().clone().unwrap();
                if (top.clone() == 0) {
                    {
                        let popped = drop_last(pos.interp_depth.clone());
                        let cont_pos = Rc::new(TokPos {
                            pos: (pos.pos.clone() + 1),
                            interp_depth: popped,
                        });
                        return scan_str_cont(source.clone(), cont_pos, pos.pos.clone());
                    }
                } else {
                    return Rc::new(ScanResult {
                        pos: (pos.pos.clone() + 1),
                        token: make_token(
                            "}".to_string(),
                            make_file_span(
                                source.file.clone(),
                                pos.pos.clone(),
                                (pos.pos.clone() + 1),
                            ),
                            TokenShape::ShRBrace,
                        ),
                        interp_depth: replace_last(pos.interp_depth.clone(), (top.clone() - 1)),
                    });
                }
            }
        }
        scan_token(source.clone(), pos.clone(), ch.clone())
    }
}

pub fn tokenize_loop(
    mut source: Rc<SourceRef>,
    mut tokens: Rc<Vec<Rc<Token>>>,
    mut pos: Rc<TokPos>,
    mut fuel: i64,
) -> Rc<TokenizerState> {
    loop {
        let s = skip_spaces(source.clone(), pos);
        if (s.pos.clone() >= source_len(source.clone())) {
            return Rc::new(TokenizerState {
                pos: s.pos.clone(),
                tokens: tokens.clone(),
                interp_depth: s.interp_depth.clone(),
            });
        }
        let result = scan_next_token(source.clone(), s.clone());
        {
            let __tco_0 = v1_rt::rc_list_push(tokens, result.token.clone());
            let __tco_1 = Rc::new(TokPos {
                pos: result.pos.clone(),
                interp_depth: result.interp_depth.clone(),
            });
            let __tco_2 = (fuel - 1);
            tokens = __tco_0;
            pos = __tco_1;
            fuel = __tco_2;
            continue;
        }
    }
}

pub fn scan_token(source: Rc<SourceRef>, pos: Rc<TokPos>, ch: i64) -> Rc<ScanResult> {
    {
        if (ch.clone() == 34) {
            return scan_string(source.clone(), pos.clone());
        }
        if is_digit(ch.clone()) {
            return scan_number(source.clone(), pos.clone());
        }
        if is_ident_start(ch.clone()) {
            return scan_ident(source.clone(), pos.clone());
        }
        let next_ch = if ((pos.pos.clone() + 1) < source_len(source.clone())) {
            source_code_point(source.clone(), (pos.pos.clone() + 1))
        } else {
            0
        };
        if ((ch.clone() == 61) && (next_ch.clone() == 62)) {
            return emit(
                pos.clone(),
                TokenShape::ShFatArrow,
                "=>".to_string(),
                2,
                source.file.clone(),
            );
        }
        if ((ch.clone() == 45) && (next_ch.clone() == 62)) {
            return emit(
                pos.clone(),
                TokenShape::ShArrow,
                "->".to_string(),
                2,
                source.file.clone(),
            );
        }
        if ((ch.clone() == 61) && (next_ch.clone() == 61)) {
            return emit(
                pos.clone(),
                TokenShape::ShEqEq,
                "==".to_string(),
                2,
                source.file.clone(),
            );
        }
        if ((ch.clone() == 33) && (next_ch.clone() == 61)) {
            return emit(
                pos.clone(),
                TokenShape::ShNe,
                "!=".to_string(),
                2,
                source.file.clone(),
            );
        }
        if ((ch.clone() == 60) && (next_ch.clone() == 61)) {
            return emit(
                pos.clone(),
                TokenShape::ShLe,
                "<=".to_string(),
                2,
                source.file.clone(),
            );
        }
        if ((ch.clone() == 62) && (next_ch.clone() == 61)) {
            return emit(
                pos.clone(),
                TokenShape::ShGe,
                ">=".to_string(),
                2,
                source.file.clone(),
            );
        }
        if ((ch.clone() == 38) && (next_ch.clone() == 38)) {
            return emit(
                pos.clone(),
                TokenShape::ShAnd,
                "&&".to_string(),
                2,
                source.file.clone(),
            );
        }
        if ((ch.clone() == 124) && (next_ch.clone() == 124)) {
            return emit(
                pos.clone(),
                TokenShape::ShOr,
                "||".to_string(),
                2,
                source.file.clone(),
            );
        }
        if ((ch.clone() == 124) && (next_ch.clone() == 62)) {
            return emit(
                pos.clone(),
                TokenShape::ShPipeArrow,
                "|>".to_string(),
                2,
                source.file.clone(),
            );
        }
        if (ch.clone() == 124) {
            return emit(
                pos.clone(),
                TokenShape::ShPipe,
                "|".to_string(),
                1,
                source.file.clone(),
            );
        }
        if ((ch.clone() == 63) && (next_ch.clone() == 63)) {
            return emit(
                pos.clone(),
                TokenShape::ShNullCoalesce,
                "??".to_string(),
                2,
                source.file.clone(),
            );
        }
        if ((ch.clone() == 46) && (next_ch.clone() == 46)) {
            return emit(
                pos.clone(),
                TokenShape::ShDotDot,
                "..".to_string(),
                2,
                source.file.clone(),
            );
        }
        if (ch.clone() == 61) {
            return emit(
                pos.clone(),
                TokenShape::ShEq,
                "=".to_string(),
                1,
                source.file.clone(),
            );
        }
        if (ch.clone() == 60) {
            return emit(
                pos.clone(),
                TokenShape::ShLt,
                "<".to_string(),
                1,
                source.file.clone(),
            );
        }
        if (ch.clone() == 62) {
            return emit(
                pos.clone(),
                TokenShape::ShGt,
                ">".to_string(),
                1,
                source.file.clone(),
            );
        }
        if (ch.clone() == 45) {
            return emit(
                pos.clone(),
                TokenShape::ShMinus,
                "-".to_string(),
                1,
                source.file.clone(),
            );
        }
        if (ch.clone() == 33) {
            return emit(
                pos.clone(),
                TokenShape::ShBang,
                "!".to_string(),
                1,
                source.file.clone(),
            );
        }
        if (ch.clone() == 63) {
            return emit(
                pos.clone(),
                TokenShape::ShQuestion,
                "?".to_string(),
                1,
                source.file.clone(),
            );
        }
        if (ch.clone() == 123) {
            {
                let new_depth = if ((pos.interp_depth.clone().len() as i64) > 0) {
                    replace_last(
                        pos.interp_depth.clone(),
                        (pos.interp_depth.clone().last().cloned().clone().unwrap() + 1),
                    )
                } else {
                    pos.interp_depth.clone()
                };
                let tok = make_token(
                    "{".to_string(),
                    make_file_span(source.file.clone(), pos.pos.clone(), (pos.pos.clone() + 1)),
                    TokenShape::ShLBrace,
                );
                return Rc::new(ScanResult {
                    pos: (pos.pos.clone() + 1),
                    token: tok.clone(),
                    interp_depth: new_depth,
                });
            }
        }
        if (ch.clone() == 125) {
            {
                let tok = make_token(
                    "}".to_string(),
                    make_file_span(source.file.clone(), pos.pos.clone(), (pos.pos.clone() + 1)),
                    TokenShape::ShRBrace,
                );
                return Rc::new(ScanResult {
                    pos: (pos.pos.clone() + 1),
                    token: tok.clone(),
                    interp_depth: pos.interp_depth.clone(),
                });
            }
        }
        let ch_text = source_char(source.clone(), pos.pos.clone());
        match v1_rt::lookup(&single_punct(), ch_text.clone()) {
            v1_rt::Witness::Holds { value: sh, .. } => emit(
                pos.clone(),
                sh.clone(),
                ch_text.clone(),
                1,
                source.file.clone(),
            ),
            v1_rt::Witness::Violates { diagnostic: _, .. } => emit(
                pos.clone(),
                TokenShape::ShUnknown,
                ch_text.clone(),
                1,
                source.file.clone(),
            ),
        }
    }
}

pub fn emit(
    pos: Rc<TokPos>,
    shape: TokenShape,
    text: String,
    len: i64,
    file: String,
) -> Rc<ScanResult> {
    {
        let token = make_token(
            text,
            make_file_span(file, pos.pos.clone(), (pos.pos.clone() + len.clone())),
            shape,
        );
        Rc::new(ScanResult {
            pos: (pos.pos.clone() + len.clone()),
            token: token,
            interp_depth: pos.interp_depth.clone(),
        })
    }
}

pub fn scan_ident(source: Rc<SourceRef>, pos: Rc<TokPos>) -> Rc<ScanResult> {
    {
        let end = source_scan_while(source.clone(), pos.pos.clone(), is_ident_char);
        let text = source_substring(source.clone(), pos.pos.clone(), end.clone());
        let shape = if is_keyword_text(text.clone()) {
            TokenShape::ShKeyword
        } else {
            TokenShape::ShIdent
        };
        let token = make_token(
            text.clone(),
            make_file_span(source.file.clone(), pos.pos.clone(), end.clone()),
            shape,
        );
        Rc::new(ScanResult {
            pos: end.clone(),
            token: token,
            interp_depth: pos.interp_depth.clone(),
        })
    }
}

pub fn scan_number(source: Rc<SourceRef>, pos: Rc<TokPos>) -> Rc<ScanResult> {
    {
        let int_end = source_scan_while(source.clone(), pos.pos.clone(), is_digit);
        if ((((int_end.clone() + 1) < source_len(source.clone()))
            && (source_code_point(source.clone(), int_end.clone()) == 46))
            && is_digit(source_code_point(source.clone(), (int_end.clone() + 1))))
        {
            {
                let frac_end = source_scan_while(source.clone(), (int_end.clone() + 1), is_digit);
                let text = source_substring(source.clone(), pos.pos.clone(), frac_end.clone());
                let token = make_token(
                    text.clone(),
                    make_file_span(source.file.clone(), pos.pos.clone(), frac_end.clone()),
                    TokenShape::ShLitFloat,
                );
                return Rc::new(ScanResult {
                    pos: frac_end.clone(),
                    token: token.clone(),
                    interp_depth: pos.interp_depth.clone(),
                });
            }
        }
        let text = source_substring(source.clone(), pos.pos.clone(), int_end.clone());
        let parsed = v1_rt::parse_int(text.clone());
        let shape = match parsed {
            Some(_) => TokenShape::ShLitInt,
            None => TokenShape::ShUnknown,
        };
        let token = make_token(
            text.clone(),
            make_file_span(source.file.clone(), pos.pos.clone(), int_end.clone()),
            shape,
        );
        Rc::new(ScanResult {
            pos: int_end.clone(),
            token: token.clone(),
            interp_depth: pos.interp_depth.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum StringScanResult {
    ClosedString { content: String, end_pos: i64 },
    InterpolationStart { content: String, end_pos: i64 },
    UnterminatedString { content: String, end_pos: i64 },
}
impl StringScanResult {
    pub fn content(&self) -> String {
        match self {
            StringScanResult::ClosedString { content: __val, .. } => __val.clone(),
            StringScanResult::InterpolationStart { content: __val, .. } => __val.clone(),
            StringScanResult::UnterminatedString { content: __val, .. } => __val.clone(),
        }
    }
    pub fn end_pos(&self) -> i64 {
        match self {
            StringScanResult::ClosedString { end_pos: __val, .. } => __val.clone(),
            StringScanResult::InterpolationStart { end_pos: __val, .. } => __val.clone(),
            StringScanResult::UnterminatedString { end_pos: __val, .. } => __val.clone(),
        }
    }
}

pub fn scan_string(source: Rc<SourceRef>, pos: Rc<TokPos>) -> Rc<ScanResult> {
    {
        let span_start = pos.pos.clone();
        let body_start = (pos.pos.clone() + 1);
        let result = scan_string_body(source.clone(), body_start, Rc::new(vec![]));
        match (*result).clone() {
            StringScanResult::ClosedString {
                content, end_pos, ..
            } => {
                let processed = process_escapes(content.clone());
                let token = make_token(
                    processed,
                    make_file_span(source.file.clone(), span_start, (end_pos.clone() + 1)),
                    TokenShape::ShLitStr,
                );
                Rc::new(ScanResult {
                    pos: (end_pos.clone() + 1),
                    token: token,
                    interp_depth: pos.interp_depth.clone(),
                })
            }
            StringScanResult::InterpolationStart {
                content, end_pos, ..
            } => {
                let processed = process_escapes(content.clone());
                let token = make_token(
                    processed,
                    make_file_span(source.file.clone(), span_start, (end_pos.clone() + 1)),
                    TokenShape::ShStrBegin,
                );
                Rc::new(ScanResult {
                    pos: (end_pos.clone() + 1),
                    token: token,
                    interp_depth: Rc::new(v1_rt::append(pos.interp_depth.clone(), 0)),
                })
            }
            StringScanResult::UnterminatedString {
                content, end_pos, ..
            } => {
                let processed = process_escapes(content.clone());
                let token = make_token(
                    processed,
                    make_file_span(source.file.clone(), span_start, end_pos.clone()),
                    TokenShape::ShUnknown,
                );
                Rc::new(ScanResult {
                    pos: end_pos.clone(),
                    token: token,
                    interp_depth: pos.interp_depth.clone(),
                })
            }
        }
    }
}

pub fn scan_str_cont(source: Rc<SourceRef>, pos: Rc<TokPos>, span_start: i64) -> Rc<ScanResult> {
    {
        let result = scan_string_body(source.clone(), pos.pos.clone(), Rc::new(vec![]));
        match (*result).clone() {
            StringScanResult::ClosedString {
                content, end_pos, ..
            } => {
                let processed = process_escapes(content.clone());
                let token = make_token(
                    processed,
                    make_file_span(source.file.clone(), span_start, (end_pos.clone() + 1)),
                    TokenShape::ShStrEnd,
                );
                Rc::new(ScanResult {
                    pos: (end_pos.clone() + 1),
                    token: token,
                    interp_depth: pos.interp_depth.clone(),
                })
            }
            StringScanResult::InterpolationStart {
                content, end_pos, ..
            } => {
                let processed = process_escapes(content.clone());
                let token = make_token(
                    processed,
                    make_file_span(source.file.clone(), span_start, (end_pos.clone() + 1)),
                    TokenShape::ShStrMid,
                );
                Rc::new(ScanResult {
                    pos: (end_pos.clone() + 1),
                    token: token,
                    interp_depth: Rc::new(v1_rt::append(pos.interp_depth.clone(), 0)),
                })
            }
            StringScanResult::UnterminatedString {
                content, end_pos, ..
            } => {
                let processed = process_escapes(content.clone());
                let token = make_token(
                    processed,
                    make_file_span(source.file.clone(), span_start, end_pos.clone()),
                    TokenShape::ShUnknown,
                );
                Rc::new(ScanResult {
                    pos: end_pos.clone(),
                    token: token,
                    interp_depth: pos.interp_depth.clone(),
                })
            }
        }
    }
}

pub fn scan_string_body(
    mut source: Rc<SourceRef>,
    mut pos: i64,
    mut acc: Rc<Vec<String>>,
) -> Rc<StringScanResult> {
    loop {
        if (pos.clone() >= source_len(source.clone())) {
            break Rc::new(StringScanResult::UnterminatedString {
                content: acc.join(&"".to_string()),
                end_pos: pos.clone(),
            });
        } else {
            let ch = source_char(source.clone(), pos.clone());
            if (ch.clone().as_str() == "\"".to_string().as_str()) {
                break Rc::new(StringScanResult::ClosedString {
                    content: acc.join(&"".to_string()),
                    end_pos: pos.clone(),
                });
            } else {
                if (ch.clone().as_str() == "\\".to_string().as_str()) {
                    if ((pos.clone() + 1) < source_len(source.clone())) {
                        let escaped = source_char(source.clone(), (pos.clone() + 1));
                        {
                            let __tco_0 = (pos + 2);
                            let __tco_1 = v1_rt::rc_list_push(
                                v1_rt::rc_list_push(acc, "\\".to_string()),
                                escaped,
                            );
                            pos = __tco_0;
                            acc = __tco_1;
                            continue;
                        }
                    } else {
                        break Rc::new(StringScanResult::UnterminatedString {
                            content: v1_rt::rc_list_push(acc, "\\".to_string())
                                .join(&"".to_string()),
                            end_pos: (pos.clone() + 1),
                        });
                    }
                } else {
                    if (ch.clone().as_str() == "{".to_string().as_str()) {
                        if should_start_interpolation(source.clone(), pos.clone()) {
                            break Rc::new(StringScanResult::InterpolationStart {
                                content: acc.join(&"".to_string()),
                                end_pos: pos.clone(),
                            });
                        } else {
                            {
                                let __tco_0 = (pos + 1);
                                let __tco_1 = v1_rt::rc_list_push(acc, "{".to_string());
                                pos = __tco_0;
                                acc = __tco_1;
                                continue;
                            }
                        }
                    } else {
                        {
                            let __tco_0 = (pos + 1);
                            let __tco_1 = v1_rt::rc_list_push(acc, ch.clone());
                            pos = __tco_0;
                            acc = __tco_1;
                            continue;
                        }
                    }
                }
            }
        }
    }
}

pub fn should_start_interpolation(source: Rc<SourceRef>, pos: i64) -> bool {
    if ((pos.clone() + 1) >= source_len(source.clone())) {
        false
    } else {
        {
            let next = source_code_point(source.clone(), (pos.clone() + 1));
            (((is_ident_start(next.clone()) || (next.clone() == 40)) || (next.clone() == 33))
                || (next.clone() == 45))
        }
    }
}

pub fn process_escapes(raw: String) -> String {
    process_escapes_loop(raw, 0, Rc::new(vec![]))
}

pub fn process_escapes_loop(mut source: String, mut pos: i64, mut acc: Rc<Vec<String>>) -> String {
    loop {
        if (pos.clone() >= v1_rt::string_length(&source)) {
            break acc.join(&"".to_string());
        } else {
            let ch = v1_rt::char_at(&source, pos.clone());
            if ((ch.clone().as_str() == "\\".to_string().as_str())
                && ((pos.clone() + 1) < v1_rt::string_length(&source)))
            {
                let next = v1_rt::char_at(&source, (pos.clone() + 1));
                let resolved = if (next.clone().as_str() == "\"".to_string().as_str()) {
                    "\"".to_string()
                } else {
                    if (next.clone().as_str() == "\\".to_string().as_str()) {
                        "\\".to_string()
                    } else {
                        if (next.clone().as_str() == "n".to_string().as_str()) {
                            "\n".to_string()
                        } else {
                            if (next.clone().as_str() == "t".to_string().as_str()) {
                                "\t".to_string()
                            } else {
                                if (next.clone().as_str() == "{".to_string().as_str()) {
                                    "{".to_string()
                                } else {
                                    if (next.clone().as_str() == "}".to_string().as_str()) {
                                        "}".to_string()
                                    } else {
                                        v1_rt::concat("\\".to_string(), next.clone())
                                    }
                                }
                            }
                        }
                    }
                };
                {
                    let __tco_0 = (pos + 2);
                    let __tco_1 = v1_rt::rc_list_push(acc, resolved);
                    pos = __tco_0;
                    acc = __tco_1;
                    continue;
                }
            } else {
                {
                    let __tco_0 = (pos + 1);
                    let __tco_1 = v1_rt::rc_list_push(acc, ch.clone());
                    pos = __tco_0;
                    acc = __tco_1;
                    continue;
                }
            }
        }
    }
}

pub fn drop_last(stack: Rc<Vec<i64>>) -> Rc<Vec<i64>> {
    {
        let len = (stack.clone().len() as i64);
        Rc::new(
            stack
                .clone()
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, v)| (i as i64, v))
                .collect::<Vec<_>>(),
        )
        .iter()
        .cloned()
        .fold(Rc::new(vec![]), |result: Rc<Vec<i64>>, pair: (i64, i64)| {
            if (pair.0.clone() < (len.clone() - 1)) {
                Rc::new(v1_rt::append(result.clone(), pair.1.clone()))
            } else {
                result.clone()
            }
        })
    }
}

pub fn replace_last(stack: Rc<Vec<i64>>, value: i64) -> Rc<Vec<i64>> {
    {
        let prefix = drop_last(stack);
        Rc::new(v1_rt::append(prefix, value))
    }
}

pub fn skip_spaces(mut source: Rc<SourceRef>, mut pos: Rc<TokPos>) -> Rc<TokPos> {
    let p = source_skip_ws(source.clone(), pos.pos.clone());
    Rc::new(TokPos {
        pos: p.clone(),
        interp_depth: pos.interp_depth.clone(),
    })
}

pub fn is_digit(ch: i64) -> bool {
    ((ch.clone() >= 48) && (ch.clone() <= 57))
}

// UAX31 Unicode 15.1 XID_Start ranges (DerivedCoreProperties.txt).
// Mirrors dsl/std/unicode.dag xid_start_ranges — single authority is the .dag file.
const XID_START_RANGES: &[(i64, i64)] = &[
    (65, 90), (97, 122), (170, 170), (181, 181), (186, 186),
    (192, 214), (216, 246), (248, 705), (710, 721), (736, 740),
    (748, 748), (750, 750), (880, 884), (886, 887), (890, 893),
    (895, 895), (902, 902), (904, 906), (908, 908), (910, 929),
    (931, 1013), (1015, 1153), (1162, 1327), (1329, 1366), (1369, 1369),
    (1376, 1416), (1488, 1514), (1519, 1522), (1568, 1610), (1646, 1647),
    (1649, 1749), (1765, 1766), (1774, 1775), (1786, 1788), (1791, 1791),
    (1808, 1808), (1810, 1839), (1869, 1957), (1969, 1969), (1994, 2026),
    (2036, 2037), (2042, 2042), (2048, 2069), (2074, 2074), (2084, 2084),
    (2088, 2088), (2112, 2136), (2144, 2154), (2160, 2183), (2185, 2190),
    (2208, 2249), (2308, 2361), (2365, 2365), (2384, 2384), (2392, 2401),
    (2417, 2432), (2437, 2444), (2447, 2448), (2451, 2472), (2474, 2480),
    (2482, 2482), (2486, 2489), (2493, 2493), (2510, 2510), (2524, 2525),
    (2527, 2529), (2544, 2545), (2556, 2556), (2565, 2570), (2575, 2576),
    (2579, 2600), (2602, 2608), (2610, 2611), (2613, 2614), (2616, 2617),
    (2649, 2652), (2654, 2654), (2674, 2676), (2693, 2701), (2703, 2705),
    (2707, 2728), (2730, 2736), (2738, 2739), (2741, 2745), (2749, 2749),
    (2768, 2768), (2784, 2785), (2809, 2809), (2821, 2828), (2831, 2832),
    (2835, 2856), (2858, 2864), (2866, 2867), (2869, 2873), (2877, 2877),
    (2908, 2909), (2911, 2913), (2929, 2929), (2947, 2947), (2949, 2954),
    (2958, 2960), (2962, 2965), (2969, 2970), (2972, 2972), (2974, 2975),
    (2979, 2980), (2984, 2986), (2990, 3001), (3024, 3024), (3077, 3084),
    (3086, 3088), (3090, 3112), (3114, 3129), (3133, 3133), (3160, 3162),
    (3165, 3165), (3168, 3169), (3200, 3200), (3205, 3212), (3214, 3216),
    (3218, 3240), (3242, 3251), (3253, 3257), (3261, 3261), (3293, 3294),
    (3296, 3297), (3313, 3314), (3332, 3340), (3342, 3344), (3346, 3396),
    (3398, 3400), (3402, 3406), (3412, 3414), (3423, 3425), (3450, 3455),
    (3461, 3478), (3482, 3505), (3507, 3515), (3517, 3517), (3520, 3526),
    (3585, 3630), (3632, 3632), (3634, 3635), (3648, 3654), (3713, 3714),
    (3716, 3716), (3718, 3722), (3724, 3747), (3749, 3749), (3751, 3760),
    (3762, 3763), (3773, 3773), (3776, 3780), (3782, 3782), (3804, 3807),
    (3840, 3840), (3904, 3911), (3913, 3948), (3976, 3980), (4096, 4138),
    (4159, 4159), (4176, 4181), (4186, 4189), (4193, 4193), (4197, 4198),
    (4206, 4208), (4213, 4225), (4238, 4238), (4256, 4293), (4295, 4295),
    (4301, 4301), (4304, 4346), (4348, 4680), (4682, 4685), (4688, 4694),
    (4696, 4696), (4698, 4701), (4704, 4744), (4746, 4749), (4752, 4784),
    (4786, 4789), (4792, 4798), (4800, 4800), (4802, 4805), (4808, 4822),
    (4824, 4880), (4882, 4885), (4888, 4954), (4992, 5007), (5024, 5109),
    (5112, 5117), (5121, 5740), (5743, 5759), (5761, 5786), (5792, 5866),
    (5870, 5880), (5888, 5905), (5919, 5937), (5952, 5969), (5984, 5996),
    (5998, 6000), (6016, 6067), (6103, 6103), (6108, 6108), (6176, 6264),
    (6272, 6314), (6320, 6389), (6400, 6430), (6480, 6509), (6512, 6516),
    (6528, 6571), (6576, 6601), (6656, 6678), (6688, 6750), (6823, 6823),
    (6917, 6963), (6981, 6988), (7043, 7072), (7086, 7087), (7098, 7141),
    (7168, 7203), (7245, 7247), (7258, 7293), (7296, 7304), (7312, 7354),
    (7357, 7359), (7401, 7418), (7424, 7615), (7680, 8188), (8305, 8305),
    (8319, 8319), (8336, 8348), (8450, 8450), (8455, 8455), (8458, 8467),
    (8469, 8469), (8473, 8477), (8484, 8484), (8486, 8486), (8488, 8488),
    (8490, 8505), (8508, 8511), (8517, 8521), (8526, 8526), (8544, 8584),
    (11264, 11310), (11312, 11492), (11499, 11502), (11506, 11507),
    (11520, 11557), (11559, 11559), (11565, 11565), (11568, 11623),
    (11631, 11631), (11648, 11670), (11680, 11686), (11688, 11694),
    (11696, 11702), (11704, 11710), (11712, 11718), (11720, 11726),
    (11728, 11734), (11736, 11742), (11823, 11823), (12293, 12295),
    (12321, 12329), (12337, 12341), (12344, 12348), (12353, 12438),
    (12445, 12447), (12449, 12543), (12549, 12591), (12593, 12686),
    (12704, 12735), (12784, 12799), (13312, 19903), (19968, 40959),
    (40960, 42191), (42192, 42237), (42240, 42508), (42512, 42527),
    (42538, 42539), (42560, 42606), (42623, 42653), (42656, 42725),
    (42775, 42783), (42786, 42888), (42891, 42954), (42960, 42961),
    (42963, 42963), (42965, 42969), (42994, 43009), (43011, 43013),
    (43015, 43018), (43020, 43042), (43072, 43115), (43138, 43187),
    (43250, 43255), (43259, 43259), (43261, 43262), (43274, 43301),
    (43312, 43334), (43360, 43388), (43396, 43442), (43471, 43471),
    (43488, 43492), (43494, 43503), (43514, 43518), (43520, 43560),
    (43584, 43586), (43588, 43595), (43616, 43638), (43642, 43642),
    (43646, 43695), (43697, 43697), (43701, 43702), (43705, 43710),
    (43712, 43712), (43714, 43714), (43739, 43741), (43744, 43748),
    (43762, 43764), (43777, 43782), (43785, 43790), (43793, 43798),
    (43808, 43814), (43816, 43822), (43824, 43866), (43868, 43881),
    (43888, 43970), (44032, 55203), (55216, 55238), (55243, 55291),
    (63744, 64109), (64112, 64217), (64467, 64829), (64848, 64911),
    (64914, 64967), (65008, 65019), (65136, 65140), (65142, 65276),
    (65313, 65338), (65345, 65370), (65382, 65470), (65474, 65479),
    (65482, 65487), (65490, 65495), (65498, 65500),
];

// XID_Continue-only ranges (not in XID_Start): digits, combining marks, connector punct.
const XID_CONTINUE_ONLY_RANGES: &[(i64, i64)] = &[
    (48, 57), (95, 95), (768, 879), (1155, 1161), (1425, 1479),
    (1552, 1562), (1611, 1631), (1632, 1641), (1648, 1648), (1750, 1756),
    (1759, 1764), (1767, 1768), (1770, 1773), (1776, 1785), (1809, 1809),
    (1840, 1866), (1958, 1968), (1984, 1993), (2027, 2035), (2045, 2047),
    (2070, 2073), (2075, 2083), (2085, 2087), (2089, 2093), (2137, 2139),
    (2200, 2207), (2250, 2273), (2275, 2307), (2362, 2364), (2366, 2383),
    (2385, 2391), (2402, 2416), (2433, 2435), (2492, 2492), (2494, 2500),
    (2503, 2504), (2507, 2509), (2519, 2519), (2530, 2531), (2534, 2543),
    (2558, 2559), (2561, 2563), (2620, 2620), (2622, 2626), (2631, 2632),
    (2635, 2637), (2641, 2641), (2662, 2677), (2689, 2691), (2748, 2748),
    (2750, 2757), (2759, 2761), (2763, 2765), (2786, 2787), (2790, 2799),
    (2817, 2819), (2876, 2876), (2878, 2884), (2887, 2888), (2891, 2893),
    (2901, 2903), (2914, 2915), (2918, 2935), (3006, 3010), (3014, 3016),
    (3018, 3021), (3031, 3031), (3046, 3066), (3072, 3076), (3132, 3132),
    (3134, 3140), (3142, 3144), (3146, 3149), (3157, 3158), (3170, 3171),
    (3174, 3183), (3201, 3212), (3260, 3260), (3263, 3268), (3270, 3272),
    (3274, 3277), (3285, 3286), (3298, 3299), (3302, 3311), (3328, 3331),
    (3387, 3388), (3390, 3396), (3398, 3400), (3402, 3406), (3415, 3415),
    (3426, 3427), (3430, 3455), (3530, 3530), (3535, 3540), (3542, 3542),
    (3544, 3551), (3558, 3567), (3655, 3662), (3664, 3673), (3784, 3789),
    (3792, 3801), (3864, 3866), (3872, 3891), (3953, 3972), (3974, 3975),
    (3981, 3991), (3993, 4028), (4038, 4038), (4141, 4153), (4155, 4159),
    (4160, 4169), (4182, 4185), (4190, 4192), (4194, 4196), (4199, 4205),
    (4209, 4212), (4226, 4237), (4239, 4239), (4250, 4253),
];

// Emoji identifier ranges (UAX31 §2.5 extension; Unicode 15.1 emoji-data.txt).
const EMOJI_IDENT_RANGES: &[(i64, i64)] = &[
    (9728, 9983), (9986, 10160), (11088, 11093), (11904, 11929),
    (126976, 127019), (127024, 127123), (127136, 127150), (127153, 127167),
    (127169, 127183), (127185, 127221), (127245, 127247), (127248, 127278),
    (127280, 127311), (127312, 127337), (127338, 127343), (127344, 127377),
    (127378, 127386), (127744, 128383), (128512, 128591), (128640, 128767),
    (128768, 128883), (128896, 128980), (129280, 129535), (129648, 129791),
];

fn in_ranges(ch: i64, ranges: &[(i64, i64)]) -> bool {
    ranges.iter().any(|&(lo, hi)| ch >= lo && ch <= hi)
}

pub fn is_xid_start(ch: i64) -> bool {
    in_ranges(ch, XID_START_RANGES)
}

pub fn is_xid_continue(ch: i64) -> bool {
    is_xid_start(ch) || in_ranges(ch, XID_CONTINUE_ONLY_RANGES)
}

pub fn is_emoji_ident(ch: i64) -> bool {
    in_ranges(ch, EMOJI_IDENT_RANGES)
}

pub fn is_ident_start(ch: i64) -> bool {
    ch == 95 || is_xid_start(ch) || is_emoji_ident(ch)
}

pub fn is_ident_char(ch: i64) -> bool {
    is_xid_continue(ch) || is_emoji_ident(ch)
}
