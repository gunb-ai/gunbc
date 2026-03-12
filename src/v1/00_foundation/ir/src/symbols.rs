//! Symbol system: visual vocabulary for the entire codebase.
//!
//! Types, data tables, and per-variant methods (`emoji()`, `unicode()`,
//! `ascii()`, `color()`, `code()`) are DSL-generated — see
//! `dsl/std/symbols.dag`.  This file provides the runtime glue that the
//! DSL cannot express: trait-like resolution by tier, the `SymbolSet`
//! collection, and terminal-control constants.

use crate::dag::{Dag, Port};
use crate::node::Node;

// ---------------------------------------------------------------------------
// Re-exports from generated code (DSL is the source of truth)
// ---------------------------------------------------------------------------

pub use crate::generated::{
    AnsiMapping, SemanticColor, SymbolEntry, SymbolId, Tier, ANSI_MAPPINGS, STANDARD_SYMBOLS,
};

/// Backwards-compatible alias — the DSL calls the struct `SymbolEntry`.
pub type Symbol = SymbolEntry;

// ---------------------------------------------------------------------------
// SemanticColor runtime methods
// ---------------------------------------------------------------------------

impl SemanticColor {
    /// ANSI escape code for this color.
    pub fn ansi(self) -> &'static str {
        self.code()
    }

    pub fn reset() -> &'static str {
        "\x1b[0m"
    }

    pub fn css_class(self) -> &'static str {
        match self {
            Self::Default => "sym-default",
            Self::Success => "sym-success",
            Self::Warning => "sym-warning",
            Self::Error => "sym-error",
            Self::Info => "sym-info",
            Self::Dim => "sym-dim",
            Self::Active => "sym-active",
            Self::Accent => "sym-accent",
        }
    }
}

// ---------------------------------------------------------------------------
// Symbol (SymbolEntry) runtime methods
// ---------------------------------------------------------------------------

impl SymbolEntry {
    /// Resolve this symbol to a string for the given tier.
    pub fn resolve(&self, tier: Tier) -> &'static str {
        match tier {
            Tier::Emoji => self.emoji,
            Tier::Unicode => self.unicode,
            Tier::Ascii => self.ascii,
        }
    }
}

// ---------------------------------------------------------------------------
// Terminal control constants
// ---------------------------------------------------------------------------

pub const CURSOR_HIDE: &str = "\x1b[?25l";
pub const CURSOR_SHOW: &str = "\x1b[?25h";

// ---------------------------------------------------------------------------
// SymbolSet — themed collection with lookup-by-id
// ---------------------------------------------------------------------------

pub struct SymbolSet {
    pub name: &'static str,
    pub default_tier: Tier,
    symbols: &'static [SymbolEntry],
}

impl SymbolSet {
    pub fn get(&self, id: SymbolId) -> &SymbolEntry {
        self.symbols
            .iter()
            .find(|s| s.id == id)
            .expect("SymbolSet missing symbol — STANDARD set must be exhaustive")
    }

    pub fn resolve(&self, id: SymbolId) -> &'static str {
        self.get(id).resolve(self.default_tier)
    }

    pub fn resolve_tier(&self, id: SymbolId, tier: Tier) -> &'static str {
        self.get(id).resolve(tier)
    }
}

pub static STANDARD: SymbolSet = SymbolSet {
    name: "standard",
    default_tier: Tier::Emoji,
    symbols: STANDARD_SYMBOLS,
};

// ---------------------------------------------------------------------------
// SubDag compositional model (SymbolOp)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolOp {
    Config,
    Atom { tier: Tier, value: String },
    Resolve,
    Compose,
    Frame { index: u8 },
    Cycle,
}

pub fn build_symbol_subdag(symbol: &SymbolEntry) -> Node<SymbolOp> {
    let mut inner = Dag::new();

    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("color", "String"),
        ],
        SymbolOp::Config,
    ));

    inner.add_node(Node::opaque(
        "emoji",
        vec![],
        vec![Port::scalar("value", "String")],
        SymbolOp::Atom {
            tier: Tier::Emoji,
            value: symbol.emoji.to_string(),
        },
    ));
    inner.add_node(Node::opaque(
        "unicode",
        vec![],
        vec![Port::scalar("value", "String")],
        SymbolOp::Atom {
            tier: Tier::Unicode,
            value: symbol.unicode.to_string(),
        },
    ));
    inner.add_node(Node::opaque(
        "ascii",
        vec![],
        vec![Port::scalar("value", "String")],
        SymbolOp::Atom {
            tier: Tier::Ascii,
            value: symbol.ascii.to_string(),
        },
    ));

    inner.add_node(Node::opaque(
        "resolve",
        vec![
            Port::scalar("tier", "Tier"),
            Port::scalar("emoji", "String"),
            Port::scalar("unicode", "String"),
            Port::scalar("ascii", "String"),
        ],
        vec![Port::scalar("output", "String")],
        SymbolOp::Resolve,
    ));

    inner.add_edge(crate::dag::build::edge(
        "emoji", "value", "resolve", "emoji",
    ));
    inner.add_edge(crate::dag::build::edge(
        "unicode", "value", "resolve", "unicode",
    ));
    inner.add_edge(crate::dag::build::edge(
        "ascii", "value", "resolve", "ascii",
    ));

    Node::subdag(format!("symbol_{:?}", symbol.id), inner)
}

pub fn build_spinner_subdag(symbol_set: &SymbolSet) -> Node<SymbolOp> {
    let mut inner = Dag::new();

    let spinner_ids = [
        SymbolId::Spinner0,
        SymbolId::Spinner1,
        SymbolId::Spinner2,
        SymbolId::Spinner3,
        SymbolId::Spinner4,
        SymbolId::Spinner5,
        SymbolId::Spinner6,
        SymbolId::Spinner7,
        SymbolId::Spinner8,
        SymbolId::Spinner9,
    ];

    for (i, _id) in spinner_ids.iter().enumerate() {
        inner.add_node(Node::opaque(
            format!("frame_{}", i),
            if i == 0 {
                vec![Port::scalar("cycle", "Unit")]
            } else {
                vec![Port::scalar("prev", "Unit")]
            },
            vec![
                Port::scalar("value", "String"),
                Port::scalar("next", "Unit"),
            ],
            SymbolOp::Frame { index: i as u8 },
        ));
    }

    inner.add_node(Node::opaque(
        "cycle",
        vec![Port::scalar("last", "Unit")],
        vec![Port::scalar("first", "Unit")],
        SymbolOp::Cycle,
    ));

    for i in 0..spinner_ids.len() - 1 {
        inner.add_edge(crate::dag::build::edge(
            &format!("frame_{}", i),
            "next",
            &format!("frame_{}", i + 1),
            "prev",
        ));
    }
    inner.add_edge(crate::dag::build::edge(
        &format!("frame_{}", spinner_ids.len() - 1),
        "next",
        "cycle",
        "last",
    ));

    let _ = symbol_set;

    Node::subdag("spinner", inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_all_tiers() {
        let sym = STANDARD.get(SymbolId::NodeCompleted);
        assert_eq!(sym.resolve(Tier::Emoji), "✅");
        assert_eq!(sym.resolve(Tier::Unicode), "●");
        assert_eq!(sym.resolve(Tier::Ascii), "[x]");
    }

    #[test]
    fn semantic_color_ansi() {
        assert_eq!(SemanticColor::Success.ansi(), "\x1b[38;5;34m");
        assert_eq!(SemanticColor::Error.ansi(), "\x1b[38;5;196m");
    }

    #[test]
    fn symbol_set_resolve() {
        assert_eq!(STANDARD.resolve(SymbolId::DagRunning), "🚀");
        assert_eq!(
            STANDARD.resolve_tier(SymbolId::DagRunning, Tier::Ascii),
            "[>]"
        );
    }

    #[test]
    fn build_symbol_subdag_works() {
        let sym = STANDARD.get(SymbolId::NodeCompleted);
        let node = build_symbol_subdag(sym);
        match &node.body {
            crate::node::NodeBody::SubDag(inner, _) => {
                assert!(
                    inner.nodes.len() >= 4,
                    "should have config + 3 atoms + resolve"
                );
            }
            _ => panic!("expected SubDag"),
        }
    }

    #[test]
    fn build_spinner_subdag_works() {
        let node = build_spinner_subdag(&STANDARD);
        match &node.body {
            crate::node::NodeBody::SubDag(inner, _) => {
                assert_eq!(inner.nodes.len(), 11);
            }
            _ => panic!("expected SubDag"),
        }
    }
}
