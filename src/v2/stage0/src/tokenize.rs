use crate::v2_core::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

pub static KEYWORDS: std::sync::LazyLock<std::collections::HashMap<String, TokenShape>> = std::sync::LazyLock::new(|| {
    HashMap::from([("module".to_string(), TokenShape::ShKwModule), ("import".to_string(), TokenShape::ShKwImport), ("type".to_string(), TokenShape::ShKwType), ("fn".to_string(), TokenShape::ShKwFn), ("func".to_string(), TokenShape::ShKwFunc), ("service".to_string(), TokenShape::ShKwService), ("resource".to_string(), TokenShape::ShKwResource), ("data".to_string(), TokenShape::ShKwData), ("extern".to_string(), TokenShape::ShKwExtern), ("interface".to_string(), TokenShape::ShKwInterface), ("pipeline".to_string(), TokenShape::ShKwPipeline), ("profile".to_string(), TokenShape::ShKwProfile), ("pattern".to_string(), TokenShape::ShKwPattern), ("let".to_string(), TokenShape::ShKwLet), ("return".to_string(), TokenShape::ShKwReturn), ("match".to_string(), TokenShape::ShKwMatch), ("if".to_string(), TokenShape::ShKwIf), ("else".to_string(), TokenShape::ShKwElse), ("for".to_string(), TokenShape::ShKwFor), ("in".to_string(), TokenShape::ShKwIn), ("where".to_string(), TokenShape::ShKwWhere), ("with".to_string(), TokenShape::ShKwWith), ("true".to_string(), TokenShape::ShKwTrue), ("false".to_string(), TokenShape::ShKwFalse), ("none".to_string(), TokenShape::ShKwNone), ("null".to_string(), TokenShape::ShKwNone), ("acquire".to_string(), TokenShape::ShKwAcquire), ("release".to_string(), TokenShape::ShKwRelease), ("capability".to_string(), TokenShape::ShKwCapability), ("operation".to_string(), TokenShape::ShKwOperation), ("input".to_string(), TokenShape::ShKwInput), ("output".to_string(), TokenShape::ShKwOutput), ("idempotent".to_string(), TokenShape::ShKwIdempotent), ("readonly".to_string(), TokenShape::ShKwReadonly), ("hermetic".to_string(), TokenShape::ShKwHermetic)])
});

pub static SINGLE_PUNCT: std::sync::LazyLock<std::collections::HashMap<String, TokenShape>> = std::sync::LazyLock::new(|| {
    HashMap::from([("(".to_string(), TokenShape::ShLParen), (")".to_string(), TokenShape::ShRParen), ("[".to_string(), TokenShape::ShLBracket), ("]".to_string(), TokenShape::ShRBracket), (":".to_string(), TokenShape::ShColon), (",".to_string(), TokenShape::ShComma), (".".to_string(), TokenShape::ShDot), ("+".to_string(), TokenShape::ShPlus), ("*".to_string(), TokenShape::ShStar), ("%".to_string(), TokenShape::ShPercent), ("/".to_string(), TokenShape::ShSlash)])
});

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TokenizerState {
    pub pos: i64,
    pub tokens: Rc<Vec<Rc<Token>>>,
    pub interp_depth: Rc<Vec<i64>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TokPos {
    pub pos: i64,
    pub interp_depth: Rc<Vec<i64>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScanResult {
    pub pos: i64,
    pub token: Rc<Token>,
    pub interp_depth: Rc<Vec<i64>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceRef {
    pub text: String,
}

pub fn make_token(text: &str, span: SourceSpan, shape: TokenShape) -> Rc<Token> {
    Rc::new(Token { text: text.to_string(), span, shape })
}

pub fn tokenize(source: &str) -> Rc<Vec<Rc<Token>>> {
    let src = Rc::new(SourceRef { text: source.to_string() });
    let initial = Rc::new(TokPos { pos: 0_i64, interp_depth: Rc::new(Vec::new()) });
    let final_state = tokenize_loop(src, Rc::new(Vec::new()), initial);
    let eof_span = SourceSpan { start: final_state.pos.clone(), end: final_state.pos.clone() };
    {
    let __rc_1 = final_state.tokens.clone();
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(make_token("", eof_span, TokenShape::ShEof));
    Rc::new(__appended_0)
}
}

pub fn tokenize_loop(source: Rc<SourceRef>, tokens: Rc<Vec<Rc<Token>>>, pos: Rc<TokPos>) -> Rc<TokenizerState> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_source = source;
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_pos = pos;
        let __hoist_len_source_text = v2_rt::string_length(&__tco_p_source.text);
        loop {
            let source = __tco_p_source;
            let tokens = __tco_p_tokens;
            let pos = __tco_p_pos;
            let s = skip_spaces_and_comments(source.clone(), pos);
            if s.pos.clone() >= __hoist_len_source_text {
    break Rc::new(TokenizerState { pos: s.pos.clone(), tokens: tokens.clone(), interp_depth: s.interp_depth.clone() });
};
            let ch = v2_rt::char_at(&source.text, s.pos.clone());
            if ch.clone() == "\n" {
    let tok = make_token("\n", SourceSpan { start: s.pos.clone(), end: s.pos.clone() + 1_i64 }, TokenShape::ShNewline);
     {
        let __tco_0 = source.clone();
        let __tco_1 = {
    let __rc_1 = tokens;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(tok.clone());
    Rc::new(__appended_0)
};
        let __tco_2 = Rc::new(TokPos { pos: s.pos.clone() + 1_i64, interp_depth: s.interp_depth.clone() });
        __tco_p_source = __tco_0;
        __tco_p_tokens = __tco_1;
        __tco_p_pos = __tco_2;
        continue;
    }

};
            if (ch.clone() == "}") && (({
    let __len_8 = s.interp_depth.clone().len();
    __len_8 as i64
}) > 0_i64) {
    let top = s.interp_depth.clone().last().cloned().unwrap();
    if top.clone() == 0_i64 {
    let popped = { let __v = s.interp_depth.clone(); Rc::new(__v[..__v.len().saturating_sub(1)].to_vec()) };
    let cont_pos = Rc::new(TokPos { pos: s.pos.clone() + 1_i64, interp_depth: popped });
    let result = scan_str_cont(source.clone(), cont_pos, s.pos.clone());
     {
        let __tco_0 = source.clone();
        let __tco_1 = {
    let __rc_3 = tokens;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(result.token.clone());
    Rc::new(__appended_2)
};
        let __tco_2 = Rc::new(TokPos { pos: result.pos.clone(), interp_depth: result.interp_depth.clone() });
        __tco_p_source = __tco_0;
        __tco_p_tokens = __tco_1;
        __tco_p_pos = __tco_2;
        continue;
    }

} else {
    let new_depth = {
    let __rc_5 = s.interp_depth.clone();
    let mut __replaced_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    if let Some(__last) = __replaced_4.last_mut() { *__last = top.clone() - 1_i64; };
    Rc::new(__replaced_4)
};
    let tok = make_token("}", SourceSpan { start: s.pos.clone(), end: s.pos.clone() + 1_i64 }, TokenShape::ShRBrace);
     {
        let __tco_0 = source.clone();
        let __tco_1 = {
    let __rc_7 = tokens;
    let mut __appended_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __appended_6.push(tok.clone());
    Rc::new(__appended_6)
};
        let __tco_2 = Rc::new(TokPos { pos: s.pos.clone() + 1_i64, interp_depth: new_depth });
        __tco_p_source = __tco_0;
        __tco_p_tokens = __tco_1;
        __tco_p_pos = __tco_2;
        continue;
    }

};
};
            let result = scan_token(source.clone(), s.clone(), &ch);
             {
                let __tco_0 = source.clone();
                let __tco_1 = {
    let __rc_10 = tokens;
    let mut __appended_9 = Rc::try_unwrap(__rc_10).unwrap_or_else(|rc| (*rc).clone());
    __appended_9.push(result.token.clone());
    Rc::new(__appended_9)
};
                let __tco_2 = Rc::new(TokPos { pos: result.pos.clone(), interp_depth: result.interp_depth.clone() });
                __tco_p_source = __tco_0;
                __tco_p_tokens = __tco_1;
                __tco_p_pos = __tco_2;
                continue;
            }

        }
    })
}

pub fn scan_token(source: Rc<SourceRef>, pos: Rc<TokPos>, ch: &str) -> Rc<ScanResult> {
    if ch == "\"" {
    return scan_string(source.clone(), pos.clone());
};
    if is_digit(&ch) {
    return scan_number(source.clone(), pos.clone());
};
    if is_ident_start(&ch) {
    return scan_ident(source.clone(), pos.clone());
};
    let next_ch = if (pos.pos.clone() + 1_i64) < v2_rt::string_length(&source.text) {
    v2_rt::char_at(&source.text, pos.pos.clone() + 1_i64)
} else {
    "".to_string()
};
    if (ch == "=") && (next_ch.clone() == ">") {
    return emit(pos.clone(), TokenShape::ShFatArrow, "=>", 2_i64);
};
    if (ch == "-") && (next_ch.clone() == ">") {
    return emit(pos.clone(), TokenShape::ShArrow, "->", 2_i64);
};
    if (ch == "=") && (next_ch.clone() == "=") {
    return emit(pos.clone(), TokenShape::ShEqEq, "==", 2_i64);
};
    if (ch == "!") && (next_ch.clone() == "=") {
    return emit(pos.clone(), TokenShape::ShNe, "!=", 2_i64);
};
    if (ch == "<") && (next_ch.clone() == "=") {
    return emit(pos.clone(), TokenShape::ShLe, "<=", 2_i64);
};
    if (ch == ">") && (next_ch.clone() == "=") {
    return emit(pos.clone(), TokenShape::ShGe, ">=", 2_i64);
};
    if (ch == "&") && (next_ch.clone() == "&") {
    return emit(pos.clone(), TokenShape::ShAnd, "&&", 2_i64);
};
    if (ch == "|") && (next_ch.clone() == "|") {
    return emit(pos.clone(), TokenShape::ShOr, "||", 2_i64);
};
    if (ch == "|") && (next_ch.clone() == ">") {
    return emit(pos.clone(), TokenShape::ShPipeArrow, "|>", 2_i64);
};
    if ch == "|" {
    return emit(pos.clone(), TokenShape::ShPipe, "|", 1_i64);
};
    if (ch == "?") && (next_ch.clone() == "?") {
    return emit(pos.clone(), TokenShape::ShNullCoalesce, "??", 2_i64);
};
    if (ch == ".") && (next_ch.clone() == ".") {
    return emit(pos.clone(), TokenShape::ShDotDot, "..", 2_i64);
};
    if ch == "=" {
    return emit(pos.clone(), TokenShape::ShEq, "=", 1_i64);
};
    if ch == "<" {
    return emit(pos.clone(), TokenShape::ShLt, "<", 1_i64);
};
    if ch == ">" {
    return emit(pos.clone(), TokenShape::ShGt, ">", 1_i64);
};
    if ch == "-" {
    return emit(pos.clone(), TokenShape::ShMinus, "-", 1_i64);
};
    if ch == "!" {
    return emit(pos.clone(), TokenShape::ShBang, "!", 1_i64);
};
    if ch == "?" {
    return emit(pos.clone(), TokenShape::ShQuestion, "?", 1_i64);
};
    if ch == "{" {
    let new_depth = if ({
    let __len_2 = pos.interp_depth.clone().len();
    __len_2 as i64
}) > 0_i64 {
    {
    let __rc_1 = pos.interp_depth.clone();
    let mut __replaced_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    if let Some(__last) = __replaced_0.last_mut() { *__last = pos.interp_depth.clone().last().cloned().unwrap() + 1_i64; };
    Rc::new(__replaced_0)
}
} else {
    pos.interp_depth.clone()
};
    let tok = make_token("{", SourceSpan { start: pos.pos.clone(), end: pos.pos.clone() + 1_i64 }, TokenShape::ShLBrace);
    return Rc::new(ScanResult { pos: pos.pos.clone() + 1_i64, token: tok.clone(), interp_depth: new_depth });
};
    if ch == "}" {
    let tok = make_token("}", SourceSpan { start: pos.pos.clone(), end: pos.pos.clone() + 1_i64 }, TokenShape::ShRBrace);
    return Rc::new(ScanResult { pos: pos.pos.clone() + 1_i64, token: tok.clone(), interp_depth: pos.interp_depth.clone() });
};
    match v2_rt::lookup(&SINGLE_PUNCT, ch.to_string()) {
    Some(sh) => {
        emit(pos.clone(), sh, &ch, 1_i64)
    }
    None => {
        emit(pos.clone(), TokenShape::ShUnknown, &ch, 1_i64)
    }
}
}

pub fn emit(pos: Rc<TokPos>, shape: TokenShape, text: &str, len: i64) -> Rc<ScanResult> {
    let token = make_token(&text, SourceSpan { start: pos.pos.clone(), end: pos.pos.clone() + len.clone() }, shape);
    Rc::new(ScanResult { pos: pos.pos.clone() + len.clone(), token, interp_depth: pos.interp_depth.clone() })
}

pub fn scan_ident(source: Rc<SourceRef>, pos: Rc<TokPos>) -> Rc<ScanResult> {
    let end = v2_rt::scan_while(&source.text, pos.pos.clone(), is_ident_char);
    let text = v2_rt::substring(&source.text, pos.pos.clone(), end.clone());
    let shape = match v2_rt::lookup(&KEYWORDS, text.clone()) {
    Some(sh) => {
        sh
    }
    None => {
        TokenShape::ShIdent
    }
};
    let token = make_token(&text, SourceSpan { start: pos.pos.clone(), end: end.clone() }, shape);
    Rc::new(ScanResult { pos: end.clone(), token, interp_depth: pos.interp_depth.clone() })
}

pub fn scan_number(source: Rc<SourceRef>, pos: Rc<TokPos>) -> Rc<ScanResult> {
    let int_end = v2_rt::scan_while(&source.text, pos.pos.clone(), is_digit.clone());
    if (((int_end.clone() + 1_i64) < v2_rt::string_length(&source.text)) && (v2_rt::char_at(&source.text, int_end.clone()) == ".")) && is_digit(&v2_rt::char_at(&source.text, int_end.clone() + 1_i64)) {
    let frac_end = v2_rt::scan_while(&source.text, int_end.clone() + 1_i64, is_digit.clone());
    let text = v2_rt::substring(&source.text, pos.pos.clone(), frac_end.clone());
    let token = make_token(&text, SourceSpan { start: pos.pos.clone(), end: frac_end.clone() }, TokenShape::ShLitFloat);
    return Rc::new(ScanResult { pos: frac_end.clone(), token: token.clone(), interp_depth: pos.interp_depth.clone() });
};
    let text = v2_rt::substring(&source.text, pos.pos.clone(), int_end.clone());
    let parsed = text.clone().parse::<i64>().ok();
    let shape = match parsed {
    Some(_) => {
        TokenShape::ShLitInt
    }
    None => {
        TokenShape::ShUnknown
    }
};
    let token = make_token(&text, SourceSpan { start: pos.pos.clone(), end: int_end.clone() }, shape);
    Rc::new(ScanResult { pos: int_end.clone(), token: token.clone(), interp_depth: pos.interp_depth.clone() })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringScanResult {
    ClosedString { content: String, end_pos: i64 },
    InterpolationStart { content: String, end_pos: i64 },
    UnterminatedString { content: String, end_pos: i64 },
}

impl Default for StringScanResult {
    fn default() -> Self {
        StringScanResult::ClosedString { content: Default::default(), end_pos: Default::default() }
    }
}

impl StringScanResult {
    pub fn content(&self) -> String {
        match self {
            StringScanResult::ClosedString { content, .. } => content.clone(),
            StringScanResult::InterpolationStart { content, .. } => content.clone(),
            StringScanResult::UnterminatedString { content, .. } => content.clone()
        }
    }

    pub fn end_pos(&self) -> i64 {
        match self {
            StringScanResult::ClosedString { end_pos, .. } => end_pos.clone(),
            StringScanResult::InterpolationStart { end_pos, .. } => end_pos.clone(),
            StringScanResult::UnterminatedString { end_pos, .. } => end_pos.clone()
        }
    }
}

pub fn scan_string(source: Rc<SourceRef>, pos: Rc<TokPos>) -> Rc<ScanResult> {
    let span_start = pos.pos.clone();
    let body_start = pos.pos.clone() + 1_i64;
    let result = scan_string_body(source, body_start, Rc::new(Vec::new()));
    match result.as_ref() {
    StringScanResult::ClosedString { content, end_pos, .. } => {
        {
    let processed = process_escapes(&content);
    let token = make_token(&processed, SourceSpan { start: span_start.clone(), end: end_pos.clone() + 1_i64 }, TokenShape::ShLitStr);
    Rc::new(ScanResult { pos: end_pos.clone() + 1_i64, token: token.clone(), interp_depth: pos.interp_depth.clone() })
}
    }
    StringScanResult::InterpolationStart { content, end_pos, .. } => {
        {
    let processed = process_escapes(&content);
    let token = make_token(&processed, SourceSpan { start: span_start.clone(), end: end_pos.clone() + 1_i64 }, TokenShape::ShStrBegin);
    Rc::new(ScanResult { pos: end_pos.clone() + 1_i64, token: token.clone(), interp_depth: {
    let __rc_1 = pos.interp_depth.clone();
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(0_i64);
    Rc::new(__appended_0)
} })
}
    }
    StringScanResult::UnterminatedString { content, end_pos, .. } => {
        {
    let processed = process_escapes(&content);
    let token = make_token(&processed, SourceSpan { start: span_start.clone(), end: end_pos.clone() }, TokenShape::ShUnknown);
    Rc::new(ScanResult { pos: end_pos.clone(), token: token.clone(), interp_depth: pos.interp_depth.clone() })
}
    }
}
}

pub fn scan_str_cont(source: Rc<SourceRef>, pos: Rc<TokPos>, span_start: i64) -> Rc<ScanResult> {
    let result = scan_string_body(source, pos.pos.clone(), Rc::new(Vec::new()));
    match result.as_ref() {
    StringScanResult::ClosedString { content, end_pos, .. } => {
        {
    let processed = process_escapes(&content);
    let token = make_token(&processed, SourceSpan { start: span_start.clone(), end: end_pos.clone() + 1_i64 }, TokenShape::ShStrEnd);
    Rc::new(ScanResult { pos: end_pos.clone() + 1_i64, token: token.clone(), interp_depth: pos.interp_depth.clone() })
}
    }
    StringScanResult::InterpolationStart { content, end_pos, .. } => {
        {
    let processed = process_escapes(&content);
    let token = make_token(&processed, SourceSpan { start: span_start.clone(), end: end_pos.clone() + 1_i64 }, TokenShape::ShStrMid);
    Rc::new(ScanResult { pos: end_pos.clone() + 1_i64, token: token.clone(), interp_depth: {
    let __rc_1 = pos.interp_depth.clone();
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(0_i64);
    Rc::new(__appended_0)
} })
}
    }
    StringScanResult::UnterminatedString { content, end_pos, .. } => {
        {
    let processed = process_escapes(&content);
    let token = make_token(&processed, SourceSpan { start: span_start.clone(), end: end_pos.clone() }, TokenShape::ShUnknown);
    Rc::new(ScanResult { pos: end_pos.clone(), token: token.clone(), interp_depth: pos.interp_depth.clone() })
}
    }
}
}

pub fn scan_string_body(source: Rc<SourceRef>, pos: i64, acc: Rc<Vec<String>>) -> Rc<StringScanResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_source = source;
        let mut __tco_p_pos = pos;
        let mut __tco_p_acc = acc;
        loop {
            let source = __tco_p_source;
            let pos = __tco_p_pos;
            let acc = __tco_p_acc;
            if pos.clone() >= v2_rt::string_length(&source.text) {
    break Rc::new(StringScanResult::UnterminatedString { content: {
    let mut __joined_0 = String::new();
    let mut __first_2 = true;
    for __elem_1 in acc.iter().cloned() {
        if !__first_2 {
    __joined_0.push_str(&"".to_string());
};
        __first_2 = false;
        __joined_0.push_str(&__elem_1);
    }
    __joined_0
}, end_pos: pos.clone() });
} else {
    let ch = v2_rt::char_at(&source.text, pos.clone());
    if ch.clone() == "\"" {
    break Rc::new(StringScanResult::ClosedString { content: {
    let mut __joined_3 = String::new();
    let mut __first_5 = true;
    for __elem_4 in acc.iter().cloned() {
        if !__first_5 {
    __joined_3.push_str(&"".to_string());
};
        __first_5 = false;
        __joined_3.push_str(&__elem_4);
    }
    __joined_3
}, end_pos: pos.clone() });
} else {
    if ch.clone() == "\\" {
    if (pos.clone() + 1_i64) < v2_rt::string_length(&source.text) {
    let escaped = v2_rt::char_at(&source.text, pos.clone() + 1_i64);
     {
        let __tco_0 = source.clone();
        let __tco_1 = pos.clone() + 2_i64;
        let __tco_2 = {
    let __rc_9 = {
    let __rc_7 = acc;
    let mut __appended_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __appended_6.push("\\".to_string());
    Rc::new(__appended_6)
};
    let mut __appended_8 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    __appended_8.push(escaped);
    Rc::new(__appended_8)
};
        __tco_p_source = __tco_0;
        __tco_p_pos = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

} else {
    break Rc::new(StringScanResult::UnterminatedString { content: {
    let mut __joined_12 = String::new();
    let mut __first_14 = true;
    for __elem_13 in ({
    let __rc_11 = acc;
    let mut __appended_10 = Rc::try_unwrap(__rc_11).unwrap_or_else(|rc| (*rc).clone());
    __appended_10.push("\\".to_string());
    Rc::new(__appended_10)
}).iter().cloned() {
        if !__first_14 {
    __joined_12.push_str(&"".to_string());
};
        __first_14 = false;
        __joined_12.push_str(&__elem_13);
    }
    __joined_12
}, end_pos: pos.clone() + 1_i64 });
};
} else {
    if ch.clone() == "{" {
    if should_start_interpolation(source.clone(), pos.clone()) {
    break Rc::new(StringScanResult::InterpolationStart { content: {
    let mut __joined_15 = String::new();
    let mut __first_17 = true;
    for __elem_16 in acc.iter().cloned() {
        if !__first_17 {
    __joined_15.push_str(&"".to_string());
};
        __first_17 = false;
        __joined_15.push_str(&__elem_16);
    }
    __joined_15
}, end_pos: pos.clone() });
} else {
     {
        let __tco_0 = source.clone();
        let __tco_1 = pos.clone() + 1_i64;
        let __tco_2 = {
    let __rc_19 = acc;
    let mut __appended_18 = Rc::try_unwrap(__rc_19).unwrap_or_else(|rc| (*rc).clone());
    __appended_18.push("{".to_string());
    Rc::new(__appended_18)
};
        __tco_p_source = __tco_0;
        __tco_p_pos = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
} else {
     {
        let __tco_0 = source.clone();
        let __tco_1 = pos.clone() + 1_i64;
        let __tco_2 = {
    let __rc_21 = acc;
    let mut __appended_20 = Rc::try_unwrap(__rc_21).unwrap_or_else(|rc| (*rc).clone());
    __appended_20.push(ch.clone());
    Rc::new(__appended_20)
};
        __tco_p_source = __tco_0;
        __tco_p_pos = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
};
};
};
        }
    })
}

pub fn should_start_interpolation(source: Rc<SourceRef>, pos: i64) -> bool {
    if (pos.clone() + 1_i64) >= v2_rt::string_length(&source.text) {
    false
} else {
    let next = v2_rt::char_at(&source.text, pos.clone() + 1_i64);
    ((is_ident_start(&next) || (next.clone() == "(")) || (next.clone() == "!")) || (next.clone() == "-")
}
}

pub fn process_escapes(raw: &str) -> String {
    process_escapes_loop(&raw, 0_i64, Rc::new(Vec::new()))
}

pub fn process_escapes_loop(source: &str, pos: i64, acc: Rc<Vec<String>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_source = source.to_string();
        let mut __tco_p_pos = pos;
        let mut __tco_p_acc = acc;
        loop {
            let source = __tco_p_source;
            let pos = __tco_p_pos;
            let acc = __tco_p_acc;
            if pos.clone() >= v2_rt::string_length(&source) {
    {
    let mut __joined_0 = String::new();
    let mut __first_2 = true;
    for __elem_1 in acc.iter().cloned() {
        if !__first_2 {
    __joined_0.push_str(&"".to_string());
};
        __first_2 = false;
        __joined_0.push_str(&__elem_1);
    }
    break __joined_0;
};
} else {
    let ch = v2_rt::char_at(&source, pos.clone());
    if (ch.clone() == "\\") && ((pos.clone() + 1_i64) < v2_rt::string_length(&source)) {
    let next = v2_rt::char_at(&source, pos.clone() + 1_i64);
    let resolved = if next.clone() == "\"" {
    "\"".to_string()
} else {
    if next.clone() == "\\" {
    "\\".to_string()
} else {
    if next.clone() == "n" {
    "\n".to_string()
} else {
    if next.clone() == "t" {
    "	".to_string()
} else {
    v2_rt::concat("\\".to_string(), next.clone())
}
}
}
};
     {
        let __tco_0 = source;
        let __tco_1 = pos.clone() + 2_i64;
        let __tco_2 = {
    let __rc_4 = acc;
    let mut __appended_3 = Rc::try_unwrap(__rc_4).unwrap_or_else(|rc| (*rc).clone());
    __appended_3.push(resolved);
    Rc::new(__appended_3)
};
        __tco_p_source = __tco_0;
        __tco_p_pos = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

} else {
     {
        let __tco_0 = source;
        let __tco_1 = pos.clone() + 1_i64;
        let __tco_2 = {
    let __rc_6 = acc;
    let mut __appended_5 = Rc::try_unwrap(__rc_6).unwrap_or_else(|rc| (*rc).clone());
    __appended_5.push(ch.clone());
    Rc::new(__appended_5)
};
        __tco_p_source = __tco_0;
        __tco_p_pos = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
};
        }
    })
}

pub fn drop_last(stack: Rc<Vec<i64>>) -> Rc<Vec<i64>> {
    let len = {
    let __len_0 = stack.clone().len();
    __len_0 as i64
};
    {
    let mut __acc_4: Rc<Vec<i64>> = Rc::new(Vec::new());
    for __elem_5 in ({
    let mut __enumerated_1 = Vec::new();
    for (__idx_2, __elem_3) in stack.clone().iter().enumerate() {
        __enumerated_1.push((__idx_2 as i64, __elem_3.clone()));
    }
    Rc::new(__enumerated_1)
}).iter().cloned() {
        __acc_4 = if __elem_5.0.clone() < (len.clone() - 1_i64) {
    {
    let __rc_7 = __acc_4;
    let mut __appended_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __appended_6.push(__elem_5.1.clone());
    Rc::new(__appended_6)
}
} else {
    __acc_4
};
    }
    __acc_4
}
}

pub fn replace_last(stack: Rc<Vec<i64>>, value: i64) -> Rc<Vec<i64>> {
    let prefix = { let __v = stack; Rc::new(__v[..__v.len().saturating_sub(1)].to_vec()) };
    {
    let __rc_1 = prefix;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(value);
    Rc::new(__appended_0)
}
}

pub fn skip_spaces_and_comments(source: Rc<SourceRef>, pos: Rc<TokPos>) -> Rc<TokPos> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_source = source;
        let mut __tco_p_pos = pos;
        loop {
            let source = __tco_p_source;
            let pos = __tco_p_pos;
            let p = v2_rt::skip_horizontal_ws(&source.text, pos.pos.clone());
            if (((p.clone() + 1_i64) < v2_rt::string_length(&source.text)) && (v2_rt::char_at(&source.text, p.clone()) == "/")) && (v2_rt::char_at(&source.text, p.clone() + 1_i64) == "/") {
    let eol = v2_rt::scan_to_eol(&source.text, p.clone());
     {
        let __tco_0 = source.clone();
        let __tco_1 = Rc::new(TokPos { pos: eol, interp_depth: pos.interp_depth.clone() });
        __tco_p_source = __tco_0;
        __tco_p_pos = __tco_1;
        continue;
    }

};
            break Rc::new(TokPos { pos: p.clone(), interp_depth: pos.interp_depth.clone() });
        }
    })
}

pub fn is_digit(ch: &str) -> bool {
    (ch >= "0") && (ch <= "9")
}

pub fn is_ident_start(ch: &str) -> bool {
    (((ch >= "a") && (ch <= "z")) || ((ch >= "A") && (ch <= "Z"))) || (ch == "_")
}

pub fn is_ident_char(ch: &str) -> bool {
    is_ident_start(&ch) || is_digit(&ch)
}

