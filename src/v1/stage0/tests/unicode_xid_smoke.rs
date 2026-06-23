use v1_compiler::v1_compiler_tokenize::{is_reserved_emit_sentinel, sentinel_prefix_matches};
use v1_compiler::v1_rt;

#[test]
fn xid_start_accepts_latin_extended() {
    assert!(v1_rt::is_xid_start(233), "U+00E9 é must be XID_Start");
    assert!(v1_rt::is_xid_start(945), "U+03B1 α must be XID_Start");
    assert!(v1_rt::is_xid_start(65), "U+0041 A must be XID_Start");
}

#[test]
fn xid_start_rejects_digit_and_space() {
    assert!(!v1_rt::is_xid_start(48), "U+0030 digit 0 must NOT be XID_Start");
    assert!(!v1_rt::is_xid_start(160), "U+00A0 NBSP must NOT be XID_Start");
}

#[test]
fn xid_continue_includes_digit() {
    assert!(v1_rt::is_xid_continue(48), "U+0030 digit 0 must be XID_Continue");
    assert!(v1_rt::is_xid_continue(233), "U+00E9 é must be XID_Continue");
    assert!(!v1_rt::is_xid_continue(160), "U+00A0 NBSP must NOT be XID_Continue");
}

#[test]
fn emoji_ident_classification() {
    // 😀 U+1F600 is Emoji and NOT XID_Continue — canonical emoji ident
    assert!(v1_rt::is_emoji_ident(0x1F600), "U+1F600 😀 must be emoji_ident");
    // # U+0023 is not emoji_ident (not Emoji_Char)
    assert!(!v1_rt::is_emoji_ident(35), "U+0023 # must NOT be emoji_ident");
    // Latin letters are XID_Continue, not in the disjoint emoji partition
    assert!(!v1_rt::is_emoji_ident(65), "U+0041 A must NOT be emoji_ident");
}

#[test]
fn sentinel_detection() {
    // canonical form: _Eu + uppercase-hex + _
    assert!(
        is_reserved_emit_sentinel("_Eu1F600_".to_string()),
        "_Eu1F600_ is the 😀 sentinel"
    );
    // wrong prefix case — must be rejected
    assert!(
        !is_reserved_emit_sentinel("_eu1F600_".to_string()),
        "_eu1F600_ has wrong-case prefix, must fail"
    );
    // no hex body — only prefix+suffix with one middle char minimum
    assert!(
        !is_reserved_emit_sentinel("_Eu_".to_string()),
        "_Eu_ has no hex body, must fail"
    );
    // non-hex body char 'G'
    assert!(
        !is_reserved_emit_sentinel("_Eu1F600G_".to_string()),
        "_Eu1F600G_ contains non-hex 'G', must fail"
    );
    // plain identifier
    assert!(
        !is_reserved_emit_sentinel("normal_ident".to_string()),
        "plain identifier must not match sentinel"
    );
    // sentinel_prefix_matches itself: exact and mismatch
    assert!(sentinel_prefix_matches("_Eu".to_string(), "_Eu".to_string(), 0, 3));
    assert!(!sentinel_prefix_matches("_ex".to_string(), "_Eu".to_string(), 0, 3));
}
