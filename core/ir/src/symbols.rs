//! Symbol system: DAG-modeled visual vocabulary for the entire codebase.
//!
//! Provides a composable symbol system with three encoding tiers
//! (Emoji, Unicode, ASCII) and semantic colors, following the same
//! SubDag compositional pattern as the language module.
//!
//! # Architecture
//!
//! - [`Tier`]: Encoding capability (Emoji > Unicode > ASCII)
//! - [`SemanticColor`]: Meaning-based color that maps to ANSI/CSS/CI
//! - [`SymbolId`]: What concept a symbol represents
//! - [`Symbol`]: A single symbol with all three tier encodings
//! - [`SymbolSet`]: A themed collection of symbols (like a TypeRegistry)
//!
//! # Compositional Model (SubDag pattern)
//!
//! Symbols follow the same SubDag pattern as the language module:
//! - Encoding tiers are a SubDag: emoji → unicode → ascii
//! - Individual symbols are SubDags: config + encoding atoms + resolve
//! - Animation frames are SubDags with cycle edges
//!
//! [`SymbolOp`] parallels [`LanguageOp`], and [`Dag<SymbolOp>`] is a
//! composable symbol definition graph.

use crate::dag::{Dag, Port};
use crate::node::Node;

/// Encoding capability tier, ordered from richest to most compatible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    /// Full emoji support (default — we control the environment)
    Emoji,
    /// Unicode box-drawing and geometric shapes
    Unicode,
    /// Pure ASCII (CI logs, pipes, legacy terminals)
    Ascii,
}

/// Meaning-based color that maps to concrete representations per context.
///
/// Maps to: ANSI escape codes (terminal), CSS classes (web),
/// Mermaid style classes (diagrams), CI annotation levels (GitHub/GitLab).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticColor {
    Default,
    Success,
    Warning,
    Error,
    Info,
    Dim,
    Active,
    Accent,
}

impl SemanticColor {
    /// ANSI escape code for this color (256-color palette).
    ///
    /// Uses the 256-color palette from `gunb.ai/pkg/fermi/colors.go` for
    /// consistent, professional appearance across modern terminals:
    ///   - Green (34):  Success, safe, completed
    ///   - Orange (208): Attention, in-progress, warning
    ///   - Red (196):   Danger, error, failure
    ///   - Cyan (39):   Info, user action needed
    ///   - Soft blue (75): Calm, informational, accent
    ///   - Dim (SGR 2): Inactive, pending, secondary
    ///   - Bold white:  Active, running
    pub fn ansi(self) -> &'static str {
        match self {
            Self::Default => "\x1b[0m",
            Self::Success => "\x1b[38;5;34m",  // 256-color green
            Self::Warning => "\x1b[38;5;208m", // 256-color orange
            Self::Error => "\x1b[38;5;196m",   // 256-color red
            Self::Info => "\x1b[38;5;39m",     // 256-color cyan
            Self::Dim => "\x1b[2m",            // SGR dim
            Self::Active => "\x1b[38;5;208m",  // 256-color orange (matches gunb.ai)
            Self::Accent => "\x1b[38;5;75m",   // 256-color soft blue
        }
    }

    /// ANSI reset code.
    pub fn reset() -> &'static str {
        "\x1b[0m"
    }

    /// CSS class name for this color (web renderer, future).
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
// Terminal control constants (matching gunb.ai/pkg/fermi/colors.go)
// ---------------------------------------------------------------------------

/// Hide the terminal cursor (useful during animations).
pub const CURSOR_HIDE: &str = "\x1b[?25l";

/// Show (restore) the terminal cursor.
pub const CURSOR_SHOW: &str = "\x1b[?25h";

/// Move cursor to column 0 (carriage return).
pub const CURSOR_TO_COL0: &str = "\r";

/// Clear the entire current line.
pub const CLEAR_LINE: &str = "\x1b[2K";

/// Clear from cursor position to end of screen.
pub const CLEAR_TO_END: &str = "\x1b[J";

/// What concept a symbol represents.
///
/// Mirrors [`TypeId`] — identifies a symbol by its semantic role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolId {
    // Node states
    NodePending,
    NodeRunning,
    NodeCompleted,
    NodeFailed,
    NodeSkipped,
    NodeIntercepted,

    // Edge states
    EdgeIdle,
    EdgeFlowing,
    EdgeDone,
    EdgeDead,

    // DAG phases
    DagNotStarted,
    DagRunning,
    DagCompleted,
    DagFailed,

    // Structural
    BoundaryMarker,

    // Spinners (animation frames — braille dots, 10 frames)
    Spinner0,
    Spinner1,
    Spinner2,
    Spinner3,
    Spinner4,
    Spinner5,
    Spinner6,
    Spinner7,
    Spinner8,
    Spinner9,

    // Status indicators (general purpose)
    Success,
    Failure,
    Warning,
    Info,

    // Data type indicators
    DataList,
    DataMap,
    DataSecret,
    DataUrl,
    DataTimer,

    // Connectors (for layout)
    ConnectorHorizontal,
    ConnectorVertical,
    ConnectorTeeDown,
    ConnectorTeeUp,
    ConnectorCornerBottomLeft,
    ConnectorCornerTopLeft,
}

/// A single symbol with all three tier encodings and a semantic color.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub id: SymbolId,
    pub emoji: &'static str,
    pub unicode: &'static str,
    pub ascii: &'static str,
    pub color: SemanticColor,
}

impl Symbol {
    /// Resolve this symbol to a string for the given tier.
    pub fn resolve(&self, tier: Tier) -> &'static str {
        match tier {
            Tier::Emoji => self.emoji,
            Tier::Unicode => self.unicode,
            Tier::Ascii => self.ascii,
        }
    }

    /// Resolve with ANSI color wrapping.
    pub fn resolve_colored(&self, tier: Tier) -> String {
        format!(
            "{}{}{}",
            self.color.ansi(),
            self.resolve(tier),
            SemanticColor::reset()
        )
    }
}

/// A themed collection of symbols — the visual vocabulary.
///
/// Mirrors [`TypeRegistry`] — lookup by [`SymbolId`].
pub struct SymbolSet {
    pub name: &'static str,
    pub default_tier: Tier,
    symbols: &'static [Symbol],
}

impl SymbolSet {
    /// Look up a symbol by id.
    pub fn get(&self, id: SymbolId) -> &Symbol {
        self.symbols
            .iter()
            .find(|s| s.id == id)
            .expect("SymbolSet missing symbol — STANDARD set must be exhaustive")
    }

    /// Resolve a symbol to a string for the default tier.
    pub fn resolve(&self, id: SymbolId) -> &'static str {
        self.get(id).resolve(self.default_tier)
    }

    /// Resolve a symbol to a string for a specific tier.
    pub fn resolve_tier(&self, id: SymbolId, tier: Tier) -> &'static str {
        self.get(id).resolve(tier)
    }

    /// Resolve with ANSI color wrapping.
    pub fn resolve_colored(&self, id: SymbolId) -> String {
        self.get(id).resolve_colored(self.default_tier)
    }

    /// All symbols in this set.
    pub fn all(&self) -> &[Symbol] {
        self.symbols
    }
}

// ---------------------------------------------------------------------------
// Standard SymbolSet
// ---------------------------------------------------------------------------

/// The standard symbol set — default visual vocabulary.
pub static STANDARD: SymbolSet = SymbolSet {
    name: "standard",
    default_tier: Tier::Emoji,
    symbols: &STANDARD_SYMBOLS,
};

static STANDARD_SYMBOLS: [Symbol; 40] = [
    // Node states
    Symbol {
        id: SymbolId::NodePending,
        emoji: "⏳",
        unicode: "○",
        ascii: "[ ]",
        color: SemanticColor::Dim,
    },
    Symbol {
        id: SymbolId::NodeRunning,
        emoji: "🔄",
        unicode: "◐",
        ascii: "[~]",
        color: SemanticColor::Active,
    },
    Symbol {
        id: SymbolId::NodeCompleted,
        emoji: "✅",
        unicode: "●",
        ascii: "[x]",
        color: SemanticColor::Success,
    },
    Symbol {
        id: SymbolId::NodeFailed,
        emoji: "❌",
        unicode: "✗",
        ascii: "[!]",
        color: SemanticColor::Error,
    },
    Symbol {
        id: SymbolId::NodeSkipped,
        emoji: "⏭️",
        unicode: "◌",
        ascii: "[-]",
        color: SemanticColor::Dim,
    },
    Symbol {
        id: SymbolId::NodeIntercepted,
        emoji: "🔮",
        unicode: "◇",
        ascii: "[m]",
        color: SemanticColor::Info,
    },
    // Edge states
    Symbol {
        id: SymbolId::EdgeIdle,
        emoji: "─",
        unicode: "─",
        ascii: "-",
        color: SemanticColor::Dim,
    },
    Symbol {
        id: SymbolId::EdgeFlowing,
        emoji: "⚡",
        unicode: "═",
        ascii: "=",
        color: SemanticColor::Accent,
    },
    Symbol {
        id: SymbolId::EdgeDone,
        emoji: "─",
        unicode: "─",
        ascii: "-",
        color: SemanticColor::Success,
    },
    Symbol {
        id: SymbolId::EdgeDead,
        emoji: "┄",
        unicode: "┄",
        ascii: ".",
        color: SemanticColor::Dim,
    },
    // DAG phases
    Symbol {
        id: SymbolId::DagNotStarted,
        emoji: "🔲",
        unicode: "□",
        ascii: "[ ]",
        color: SemanticColor::Dim,
    },
    Symbol {
        id: SymbolId::DagRunning,
        emoji: "🚀",
        unicode: "▶",
        ascii: "[>]",
        color: SemanticColor::Active,
    },
    Symbol {
        id: SymbolId::DagCompleted,
        emoji: "🏁",
        unicode: "■",
        ascii: "[X]",
        color: SemanticColor::Success,
    },
    Symbol {
        id: SymbolId::DagFailed,
        emoji: "💥",
        unicode: "■",
        ascii: "[!]",
        color: SemanticColor::Error,
    },
    // Structural
    Symbol {
        id: SymbolId::BoundaryMarker,
        emoji: "🌐",
        unicode: "◈",
        ascii: "[B]",
        color: SemanticColor::Info,
    },
    // Spinners — braille dots (10 frames, matching gunb.ai)
    // Braille characters are universally supported in modern terminals across all tiers.
    Symbol {
        id: SymbolId::Spinner0,
        emoji: "⠋",
        unicode: "⠋",
        ascii: "|",
        color: SemanticColor::Active,
    },
    Symbol {
        id: SymbolId::Spinner1,
        emoji: "⠙",
        unicode: "⠙",
        ascii: "/",
        color: SemanticColor::Active,
    },
    Symbol {
        id: SymbolId::Spinner2,
        emoji: "⠹",
        unicode: "⠹",
        ascii: "-",
        color: SemanticColor::Active,
    },
    Symbol {
        id: SymbolId::Spinner3,
        emoji: "⠸",
        unicode: "⠸",
        ascii: "\\",
        color: SemanticColor::Active,
    },
    Symbol {
        id: SymbolId::Spinner4,
        emoji: "⠼",
        unicode: "⠼",
        ascii: "|",
        color: SemanticColor::Active,
    },
    Symbol {
        id: SymbolId::Spinner5,
        emoji: "⠴",
        unicode: "⠴",
        ascii: "/",
        color: SemanticColor::Active,
    },
    Symbol {
        id: SymbolId::Spinner6,
        emoji: "⠦",
        unicode: "⠦",
        ascii: "-",
        color: SemanticColor::Active,
    },
    Symbol {
        id: SymbolId::Spinner7,
        emoji: "⠧",
        unicode: "⠧",
        ascii: "\\",
        color: SemanticColor::Active,
    },
    Symbol {
        id: SymbolId::Spinner8,
        emoji: "⠇",
        unicode: "⠇",
        ascii: "|",
        color: SemanticColor::Active,
    },
    Symbol {
        id: SymbolId::Spinner9,
        emoji: "⠏",
        unicode: "⠏",
        ascii: "/",
        color: SemanticColor::Active,
    },
    // Status indicators
    Symbol {
        id: SymbolId::Success,
        emoji: "✅",
        unicode: "✓",
        ascii: "OK",
        color: SemanticColor::Success,
    },
    Symbol {
        id: SymbolId::Failure,
        emoji: "❌",
        unicode: "✗",
        ascii: "FAIL",
        color: SemanticColor::Error,
    },
    Symbol {
        id: SymbolId::Warning,
        emoji: "⚠️",
        unicode: "⚠",
        ascii: "WARN",
        color: SemanticColor::Warning,
    },
    Symbol {
        id: SymbolId::Info,
        emoji: "ℹ️",
        unicode: "ℹ",
        ascii: "INFO",
        color: SemanticColor::Info,
    },
    // Data types
    Symbol {
        id: SymbolId::DataList,
        emoji: "📋",
        unicode: "≡",
        ascii: "[L]",
        color: SemanticColor::Default,
    },
    Symbol {
        id: SymbolId::DataMap,
        emoji: "🗂️",
        unicode: "⊞",
        ascii: "[M]",
        color: SemanticColor::Default,
    },
    Symbol {
        id: SymbolId::DataSecret,
        emoji: "🔒",
        unicode: "▪",
        ascii: "[*]",
        color: SemanticColor::Warning,
    },
    Symbol {
        id: SymbolId::DataUrl,
        emoji: "🔗",
        unicode: "↗",
        ascii: "[U]",
        color: SemanticColor::Info,
    },
    Symbol {
        id: SymbolId::DataTimer,
        emoji: "⏱️",
        unicode: "⏱",
        ascii: "[T]",
        color: SemanticColor::Dim,
    },
    // Connectors
    Symbol {
        id: SymbolId::ConnectorHorizontal,
        emoji: "──",
        unicode: "──",
        ascii: "--",
        color: SemanticColor::Dim,
    },
    Symbol {
        id: SymbolId::ConnectorVertical,
        emoji: "│",
        unicode: "│",
        ascii: "|",
        color: SemanticColor::Dim,
    },
    Symbol {
        id: SymbolId::ConnectorTeeDown,
        emoji: "┬",
        unicode: "┬",
        ascii: "+",
        color: SemanticColor::Dim,
    },
    Symbol {
        id: SymbolId::ConnectorTeeUp,
        emoji: "┴",
        unicode: "┴",
        ascii: "+",
        color: SemanticColor::Dim,
    },
    Symbol {
        id: SymbolId::ConnectorCornerBottomLeft,
        emoji: "└",
        unicode: "└",
        ascii: "+",
        color: SemanticColor::Dim,
    },
    Symbol {
        id: SymbolId::ConnectorCornerTopLeft,
        emoji: "┌",
        unicode: "┌",
        ascii: "+",
        color: SemanticColor::Dim,
    },
];

// ---------------------------------------------------------------------------
// SubDag compositional model (SymbolOp)
// ---------------------------------------------------------------------------

/// Operations for symbol definition DAGs.
///
/// Parallels [`LanguageOp`] — each variant is a node operation
/// in a `Dag<SymbolOp>` that defines symbol resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolOp {
    /// Symbol metadata: id, color.
    Config,
    /// Concrete encoding for a specific tier.
    Atom { tier: Tier, value: String },
    /// Tier → atom selection (picks the right encoding).
    Resolve,
    /// Multi-symbol concatenation.
    Compose,
    /// Animation frame content.
    Frame { index: u8 },
    /// Frame loop control (cycle back to first frame).
    Cycle,
}

/// Build a symbol definition SubDag for a single symbol.
///
/// Structure: config → [emoji_atom, unicode_atom, ascii_atom] → resolve
///
/// Parallels [`build_html_subdag()`] in the language module.
pub fn build_symbol_subdag(symbol: &Symbol) -> Node<SymbolOp> {
    let mut inner = Dag::new();

    // Config node — symbol metadata
    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::scalar("color", "String"),
        ],
        SymbolOp::Config,
    ));

    // Atom nodes — one per tier
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

    // Resolve node — selects atom based on tier input
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

    // Edges: atoms → resolve
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

/// Build a spinner animation SubDag (10 braille frames + cycle).
///
/// Structure: frame_0 → frame_1 → … → frame_9 → (cycle back to 0)
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

    // Cycle node — connects last frame back to first
    inner.add_node(Node::opaque(
        "cycle",
        vec![Port::scalar("last", "Unit")],
        vec![Port::scalar("first", "Unit")],
        SymbolOp::Cycle,
    ));

    // Chain: frame_0 → frame_1 → … → frame_9 → cycle → frame_0
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
    // Note: cycle → frame_0 edge creates a cycle — modeled as metadata, not an actual DAG edge

    let _ = symbol_set; // Used to select frame content per tier at resolve time

    Node::subdag("spinner", inner)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_all_tiers() {
        let sym = STANDARD.get(SymbolId::NodeCompleted);
        assert_eq!(sym.resolve(Tier::Emoji), "✅");
        assert_eq!(sym.resolve(Tier::Unicode), "●");
        assert_eq!(sym.resolve(Tier::Ascii), "[x]");
    }

    #[test]
    fn test_standard_set_exhaustive() {
        // Every SymbolId variant should be present in STANDARD
        let all_ids = [
            SymbolId::NodePending,
            SymbolId::NodeRunning,
            SymbolId::NodeCompleted,
            SymbolId::NodeFailed,
            SymbolId::NodeSkipped,
            SymbolId::NodeIntercepted,
            SymbolId::EdgeIdle,
            SymbolId::EdgeFlowing,
            SymbolId::EdgeDone,
            SymbolId::EdgeDead,
            SymbolId::DagNotStarted,
            SymbolId::DagRunning,
            SymbolId::DagCompleted,
            SymbolId::DagFailed,
            SymbolId::BoundaryMarker,
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
            SymbolId::Success,
            SymbolId::Failure,
            SymbolId::Warning,
            SymbolId::Info,
            SymbolId::DataList,
            SymbolId::DataMap,
            SymbolId::DataSecret,
            SymbolId::DataUrl,
            SymbolId::DataTimer,
            SymbolId::ConnectorHorizontal,
            SymbolId::ConnectorVertical,
            SymbolId::ConnectorTeeDown,
            SymbolId::ConnectorTeeUp,
            SymbolId::ConnectorCornerBottomLeft,
            SymbolId::ConnectorCornerTopLeft,
        ];
        for id in &all_ids {
            STANDARD.get(*id); // panics if missing
        }
    }

    #[test]
    fn test_semantic_color_ansi() {
        assert_eq!(SemanticColor::Success.ansi(), "\x1b[38;5;34m");
        assert_eq!(SemanticColor::Error.ansi(), "\x1b[38;5;196m");
        assert_eq!(SemanticColor::Warning.ansi(), "\x1b[38;5;208m");
        assert_eq!(SemanticColor::Info.ansi(), "\x1b[38;5;39m");
        assert_eq!(SemanticColor::Accent.ansi(), "\x1b[38;5;75m");
        assert_eq!(SemanticColor::Dim.ansi(), "\x1b[2m");
        assert_eq!(SemanticColor::Active.ansi(), "\x1b[38;5;208m");
    }

    #[test]
    fn test_resolve_colored() {
        let sym = STANDARD.get(SymbolId::NodeCompleted);
        let colored = sym.resolve_colored(Tier::Ascii);
        assert!(colored.contains("[x]"));
        assert!(colored.contains("\x1b[38;5;34m")); // 256-color green for Success
        assert!(colored.contains("\x1b[0m")); // Reset
    }

    #[test]
    fn test_terminal_control_constants() {
        assert_eq!(CURSOR_HIDE, "\x1b[?25l");
        assert_eq!(CURSOR_SHOW, "\x1b[?25h");
        assert_eq!(CLEAR_LINE, "\x1b[2K");
        assert_eq!(CLEAR_TO_END, "\x1b[J");
    }

    #[test]
    fn test_symbol_set_resolve() {
        assert_eq!(STANDARD.resolve(SymbolId::DagRunning), "🚀");
        assert_eq!(
            STANDARD.resolve_tier(SymbolId::DagRunning, Tier::Ascii),
            "[>]"
        );
    }

    #[test]
    fn test_spinner_frames_distinct() {
        let frames: Vec<&str> = [
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
        ]
        .iter()
        .map(|id| STANDARD.resolve_tier(*id, Tier::Unicode))
        .collect();

        // All ten frames should be distinct
        for i in 0..frames.len() {
            for j in (i + 1)..frames.len() {
                assert_ne!(
                    frames[i], frames[j],
                    "spinner frames {} and {} are identical",
                    i, j
                );
            }
        }
    }

    #[test]
    fn test_build_symbol_subdag() {
        let sym = STANDARD.get(SymbolId::NodeCompleted);
        let node = build_symbol_subdag(sym);
        // SubDag — interface inferred from inner DAG entrypoints/boundaries
        match &node.body {
            crate::node::NodeBody::SubDag(inner) => {
                assert!(
                    inner.nodes.len() >= 4,
                    "should have config + 3 atoms + resolve"
                );
            }
            _ => panic!("expected SubDag"),
        }
        // Entrypoint: the resolve node's "tier" port (unconnected input)
        assert!(node.inputs.iter().any(|p| p.name.0 == "tier"));
        // Boundary: the resolve node's "output" port (unconnected output)
        assert!(node.outputs.iter().any(|p| p.name.0 == "output"));
    }

    #[test]
    fn test_build_spinner_subdag() {
        let node = build_spinner_subdag(&STANDARD);
        match &node.body {
            crate::node::NodeBody::SubDag(inner) => {
                // 10 frame nodes + 1 cycle node = 11
                assert_eq!(inner.nodes.len(), 11);
            }
            _ => panic!("expected SubDag"),
        }
    }

    #[test]
    fn test_all_tiers_nonempty() {
        for sym in STANDARD.all() {
            assert!(!sym.emoji.is_empty(), "{:?} has empty emoji", sym.id);
            assert!(!sym.unicode.is_empty(), "{:?} has empty unicode", sym.id);
            assert!(!sym.ascii.is_empty(), "{:?} has empty ascii", sym.id);
        }
    }
}
