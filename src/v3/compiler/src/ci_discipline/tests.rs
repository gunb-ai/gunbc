use super::{
    check_banked_dissolutions, check_fabrication_sentinels, check_rust_toolchain_single_authority,
};

#[test]
fn fabrication_sentinels_passes_on_clean_tree() {
    check_fabrication_sentinels().expect("repo must not contain __BUG_NO_PROFILE_");
}

#[test]
fn banked_dissolutions_passes_on_clean_tree() {
    check_banked_dissolutions().expect("lane/phase docs must not restate forbidden shapes");
}

#[test]
fn rust_toolchain_single_authority_passes_on_clean_tree() {
    check_rust_toolchain_single_authority()
        .expect("rust-toolchain.toml must be sole channel authority");
}
