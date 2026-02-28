// Transitional extraction for C11: keep a single compiled implementation in
// core/resolve while preserving the existing source file location until the
// follow-up physical move lands.
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../gunbc-dag/src/resolve_service.rs"
));
