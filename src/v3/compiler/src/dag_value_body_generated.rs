// AUTO-GENERATED from `src/v3/std/substrate.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone)]
pub enum ValueBody {
    Unparsed(SourceSpan),
    Structural {
        fields: Vec<(String, FieldValue)>,
    },
    Scalar(LiteralBits),
    List(Vec<FieldValue>),
    Map(FieldMap),
}
