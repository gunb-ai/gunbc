// Compilable emit-surface subset for fold_family_head behavioral receipt.
// gunbc emit succeeds for the full module; the emitted entry rs still carries
// FreeMonoid note carriers that fail standalone rustc. This shim keeps the
// fold_family_head arm byte-aligned with gunbc output until the emit surface
// lands (sfl_scaffold_dissolution_trigger).

pub type Symbol = String;

pub fn fold_family_head(sym: Symbol) -> bool {
    sym == "fold" || sym == "fold_list" || sym == "fold_list_right" || sym == "fold_node"
}
