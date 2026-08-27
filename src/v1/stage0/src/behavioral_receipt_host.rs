#![allow(clippy::disallowed_macros)]

use crate::cli_run::namespace_wave_admission::git_stdout;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BehavioralHostTermination {
    ObservationHeld,
    ObservationDidNotHold,
    SubjectUnreached,
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehavioralHostOutcome {
    pub termination: BehavioralHostTermination,
    pub message: String,
}

// the emitter to reproduce the seed's warts. The evidence it asks for instead is execution:
// the emitted module compiles and is behaviorally equivalent to the seed on a discriminating
// corpus.
//
// WHY THE NAIVE FORM OF THAT IS VACUOUS TODAY, and this is the whole reason the mode is shaped
// the way it is. After the mirror convergence every emitted candidate is byte-identical to its
// committed mirror (measured: 0 drifted of 129). Emitting both sides and diffing therefore
// compares a program against itself — a receipt that cannot fail is not a weak receipt, it is
// not a receipt. The question with content is the one every future authority edit raises: THE
// AUTHORITY CHANGED AND THE EMISSION CHANGED WITH IT — is the new module still behaviorally the
// same? That is exactly what byte-equality cannot answer, and it has content precisely when
// candidate and seed differ.
//
// SELECTION IS DEMAND-DIRECTED AND DERIVED. Two compiler builds per module across 129 modules is
// not a per-PR gate, it is a budget denominated in the corpus rather than in the change — the
// cost shape DESIGN §5 names, where the bill grows with the repository until it breaks. So the
// subject is the modules whose `.dag` AUTHORITY moved in this diff, and the mapping from an
// authority to its emitted mirror is read off the mirror's own authority header rather
// than from any authored roster. Nothing here can be forged by editing a list.
//
// AND IF THE SELECTION CANNOT BE COMPUTED IT REFUSES. It does not widen to the whole population.
// "I could not determine what changed" and "everything changed" are different states, and
// rendering the first as the second is the absorbing fallback — nothing is missed, so it looks
// safe, while the deficit's frequency drops to zero by construction and the cost lands on every
// future run.

/// Why a changed authority produced no receipt. Every arm is COUNTED and NAMED in the report:
/// a module that silently produces nothing is indistinguishable from a module that passed, and
/// that conflation is what makes an unexecuted receipt read as a green one.
///
/// EVERY ARM HAS A PRODUCER. An earlier revision declared three, adding `NoEmissionChange`
/// (authority moved but emission did not) and `CorpusNotDerivable`. Neither was ever constructed:
/// the first needs the emission this fragment does not yet compute, and the second was a second
/// name for `ModuleCorpusPlan::refused`, which already carries every refusal with the function
/// and type that caused it. A declared-but-unconstructed variant is vocabulary claiming a
/// distinction nothing draws, so both were deleted rather than carried until a producer appeared.
/// `NoEmissionChange` returns when the two-build differential lands and can actually observe it.
///
/// `NoFunctionHasACorpus` is the one that came back, WITH a producer and at a different grain --
/// see its own comment.
#[derive(Debug, Clone, PartialEq)]
enum ReceiptExclusion {
    /// No emitted mirror in the generated population names this authority, under EITHER header
    /// convention the corpus uses (`MirrorIndex`). This is a fact rather than a lookup failure:
    /// the index refuses outright if any self-declared generated file is unindexable, so it
    /// cannot reach this arm while blind. Legitimate whenever an authority emits something that
    /// is not a Rust module mirror -- a workflow YAML, a fixture -- and named rather than
    /// skipped, because a changed authority that maps to nothing must be visible.
    NoEmittedMirror { module_path: String },

    /// The authority declares NO functions at all, so it has no behaviour that could diverge.
    ///
    /// SEPARATE FROM `NoFunctionHasACorpus` BECAUSE THEY ARE DIFFERENT STATES WITH DIFFERENT
    /// REMEDIES, and the arm that used to carry both said something false about this one. A
    /// module with twenty functions none of whose corpora this fragment can derive is a DEFICIT
    /// in the derivation -- it ranks, it names the types responsible, and closing them makes the
    /// module checkable. A module with zero functions is not a deficit at all: there is nothing
    /// to derive and never will be, so reporting it as `none of its 0 declared functions yields
    /// a call` sends a reader to fix a derivation that has no subject, and inflates the
    /// non-derivability population with rows no work can remove.
    NoFunctionDeclared { module_path: String },

    /// NOT ONE FUNCTION in this authority yields a call, so there is no corpus to compare -- and
    /// the arm carries EVERY declared function with the cause that stopped it, because the
    /// deficit lives at the function and a module-level "nothing derived" cannot be acted on.
    ///
    /// WHY THIS IS AN EXCLUSION AND NOT A REFUSAL, which is the whole of the change that
    /// introduced it. The verdict used to be decided by a FILE-LEVEL count crossing zero: a
    /// module where 1 of 20 functions derived ran and reported EQUIVALENT, and a module where 0
    /// of 20 derived hard-failed required CI. One deficit -- functions whose corpus this fragment
    /// cannot derive -- and two opposite verdicts, separated by nothing but where the count
    /// happened to land. Worse, the red had no closing move: an author cannot make
    /// `List<T>` enumerable to get their diff through, so the only way past the gate was to stop
    /// touching the authority. A gate whose sole closing move does not exist does not enforce,
    /// it launders (DESIGN.md, the fixed-point repair).
    ///
    /// So non-derivability is reported at the grain it occurs at -- per function, typed, located,
    /// counted -- and a module with no covered function is EXCLUDED from a differential it can
    /// never take, exactly as `NoEmittedMirror` is. It is not silence: the count of uncovered
    /// functions is printed on every run beside the module count, so the deficit stays rankable
    /// rather than having its frequency zeroed.
    NoFunctionHasACorpus {
        module_path: String,
        /// `(function, why it yields no call)`, one row per declared function.
        uncovered: Vec<(String, RefusalCause)>,
    },
}

/// The domain a corpus actually enumerated, reported as a DERIVED FACT rather than a label.
///
/// Both arms are COVERAGE CLAIMS. There is deliberately no "bounded sample" arm: an earlier
/// revision enumerated `Int` over [-2,2] and `List` to length 3 and reported the result beside
/// genuine exhaustive coverage. That arm is DELETED rather than widened, because a window is not
/// a weaker proof of the same thing — it is a receipt that USUALLY cannot fail, and a
/// usually-passing receipt reports as done while a refusal is counted and ranks for work. The
/// window was also measurably absurd in place: it enumerated a lower-hex-digit predicate over
/// five values containing no hex digit at all.
#[derive(Debug, Clone, PartialEq)]
enum EnumeratedDomain {
    /// Finite closed domain, fully covered: closed nullary enums, Bool, and records over them.
    /// Enumeration IS the domain.
    Exhaustive { cardinality: usize },
    /// Infinite domain, fully covered ANYWAY, because the function cannot distinguish the values
    /// inside a class.
    ///
    /// The argument, in full, because the claim is only as good as it: if a parameter's every
    /// occurrence in the body is an operand of a comparison against an integer literal, then the
    /// literals cut the integers into finitely many classes -- the points themselves and the open
    /// gaps between them -- and every comparison in the function yields the same answer for any
    /// two values drawn from one class. The function's behaviour is therefore constant within a
    /// class, so one representative per class covers the type. This is exhaustive in the same
    /// sense as a closed enum, NOT an approximation of it.
    ///
    /// The premise is what makes it sound, and it is checked rather than assumed: the moment the
    /// parameter is returned, embedded in a record, passed to another function, or arithmetically
    /// combined, its VALUE reaches the output and two members of one class stop agreeing --
    /// `fn shard_count_positive(n: Int) -> Int { if n <= 0 { 1 } else { n } }` returns 5 for 5 and
    /// 7 for 7, both in the class `n > 0`. So any occurrence that is not a literal comparison
    /// REFUSES the whole parameter. The check is conservative in the safe direction: an
    /// occurrence it cannot classify refuses rather than being assumed harmless.
    ExhaustiveOverDerivedPartition {
        cardinality: usize,
        partition: String,
    },
}

/// One parameter's domain as the ACTUAL VALUES, rendered as Rust expressions against the emitted
/// mirror, plus how that domain was established.
///
/// The count is `values.len()`. It is not carried separately, because a cardinality computed
/// beside an enumeration is a second producer of one fact: the two can disagree, and the one that
/// gets reported is the one that never ran. An earlier revision derived only the count, which is
/// why the corpus could be described in the report but never executed.
#[derive(Debug, Clone)]
struct ParameterDomain {
    values: Vec<String>,
    partition: Option<String>,
}

impl EnumeratedDomain {
    fn report(&self) -> String {
        match self {
            EnumeratedDomain::Exhaustive { cardinality } => {
                format!("exhaustive(|domain|={cardinality})")
            }
            EnumeratedDomain::ExhaustiveOverDerivedPartition {
                cardinality,
                partition,
            } => format!("exhaustive-over-partition(|reps|={cardinality}, {partition})"),
        }
    }
}

/// A type declared by the authority, in the only two shapes the fragment can enumerate.
#[derive(Debug, Clone)]
enum DagTypeDecl {
    /// `type AxisGoal = HigherIsBetter | LowerIsBetter` — a closed coproduct of NULLARY variants.
    /// A variant carrying a payload is deliberately NOT this: it would need its payload's domain
    /// enumerated too, and admitting it here without doing that would silently under-cover.
    ClosedNullaryEnum { variants: Vec<String> },
    /// `type DominanceTally { saw_better: Bool, saw_worse: Bool }` — a record over named fields.
    Record { fields: Vec<(String, String)> },
    /// A closed coproduct whose variants carry payloads. NOT enumerable here — each payload needs
    /// its own domain — but recorded as its own kind rather than left unparsed, because the two
    /// produce the same refusal COUNT and completely different refusal MEANINGS. Reporting "not a
    /// closed type declared by this authority" about a type that is declared and IS closed sends
    /// the reader to close something already closed.
    PayloadCoproduct { variant_count: usize },
    /// DECLARED, but carrying no constructor set this fragment can enumerate: an opaque type, an
    /// alias, a form the reader does not model, or a record whose field types it could not read.
    ///
    /// This arm exists so those declarations are REGISTERED rather than dropped. Dropped, they
    /// were reported as "no module in the corpus declares this type" -- a positive claim about the
    /// corpus produced by the reader's own silence, which is the empty-observation narrow with a
    /// wildcard for a cause. Registered, they refuse honestly and rank at zero work.
    DeclaredNotEnumerable { form: String },
}

/// Parse one `.dag` source through the GRAMMAR-OWNED parser and return its module node.
///
/// This replaces a hand-rolled line reader, and the replacement is the point rather than a
/// tidying. The line reader recognised only declarations that fit on one line: measured across
/// the corpus that was 989 of 8796, so it was blind to 88% of the type declarations in the
/// repository, and every one of them was then refused as "not a closed type declared by this
/// authority" -- false, since they are declared and many are closed. Nothing unsound was claimed,
/// because the refusal did stop the line; but a refusal's job beyond stopping is to RANK, and
/// that one sent the reader to close types that were already closed.
///
/// The deeper defect is that a hand-rolled reader is a SECOND PARSER for `.dag` standing beside
/// the real one, which is the single-authority violation in its plainest form: it will be wrong
/// again whenever the grammar moves, and wrong SILENTLY, because a line reader cannot distinguish
/// "did not match" from "is not there". `parse_with_table` can: it returns an error arm, so a
/// source it cannot read REFUSES instead of yielding zero declarations.
fn parse_dag_module_node(file: &str, source: &str) -> Result<Rc<crate::v1_std_core::Node>, String> {
    use crate::v1_compiler_parse::parse_with_table;
    use crate::v1_compiler_tokenize::tokenize;
    use crate::v1_std_core::{build_newline_index, empty_intern_table, NewlineIndex};

    // The map is built with the runtime's own constructors rather than a `std::HashMap`: the
    // parser's source-index map is an `im::HashMap`, and reaching for the concrete type here
    // would be this file asserting a representation the parser owns.
    let index = build_newline_index(file.to_string(), source.to_string());
    let indices = crate::v1_rt::rc_map_insert(
        crate::v1_rt::rc_empty_map::<String, Rc<NewlineIndex>>(),
        file.to_string(),
        index,
    );
    let parsed = parse_with_table(
        tokenize(source.to_string(), file.to_string()),
        indices,
        empty_intern_table(),
    );
    if let Some(err) = parsed.result.error.clone() {
        return Err(format!(
            "{file}: the grammar refused this source: {:?}",
            err.diagnostic
        ));
    }
    parsed
        .result
        .module
        .clone()
        .ok_or_else(|| format!("{file}: the parse produced neither a module nor an error"))
}

/// Render a type-annotation node back to the name the corpus writes, generics included.
fn type_text(node: &crate::v1_std_core::Node) -> String {
    // Generic ARGUMENTS are children of the type node. Reading `params` instead rendered
    // `List<AxisComparison>` as bare `List`, which then failed the `List<` prefix test and fell
    // through to "not a closed type declared by this authority" — a refusal naming the wrong
    // cause for an entire population, and the reason the first corpus histogram had no List row
    // at all despite lists being the second-largest blocker.
    let args: Vec<String> = node.children.iter().map(|c| type_text(c)).collect();
    if args.is_empty() {
        return node.name.clone();
    }
    format!("{}<{}>", node.name, args.join(", "))
}

/// Read every type declaration off a parsed module node.
///
/// A declaration's SHAPE is read from the substrate's own connective rather than from
/// punctuation: `Disj` is a coproduct and `Conj` is a record. That is why this reader does not
/// care whether the author wrote the body on one line or ten -- the distinction the line reader
/// tripped on does not exist at this layer, which is the strongest evidence that the layer is the
/// right one.
fn type_decls_from_module(
    module: &crate::v1_std_core::Node,
) -> std::collections::HashMap<String, DagTypeDecl> {
    use crate::v1_std_core::Connective;
    let mut out = std::collections::HashMap::new();
    for decl in module.children.iter() {
        // WHICH CHILDREN ARE TYPE DECLARATIONS AT ALL -- measured, not named from memory. The
        // grouped shape census over one module's children is unambiguous:
        //
        //   is_type=true   conn=Conj/Disj    body=false  params=0  inferred=false
        //   is_type=false  conn=NoConnective body=true   (compare_int, tally_verdict, no_names)
        //
        // A function or a data row carries a BODY; a type declaration does not. Without this the
        // arm has no notion of its own subject, which is the single cause of BOTH failures here:
        // filtering implicitly on two connectives was too narrow (70% of declarations dropped),
        // and filtering on nothing was too wide (every function registered as a type, 6509 read
        // against 957 authored). Neither was a bug in the filter; both were its absence.
        if decl.body.is_some() {
            continue;
        }
        match decl.connective {
            Connective::Disj => {
                let variants: Vec<String> = decl.children.iter().map(|v| v.name.clone()).collect();
                if variants.is_empty() {
                    continue;
                }
                // A variant with no fields of its own is nullary. A variant carrying a payload
                // needs that payload's domain enumerated too, so the whole coproduct is recorded
                // as payload-carrying rather than being enumerated one-value-per-variant, which
                // would under-cover silently.
                let all_nullary = decl.children.iter().all(|v| v.children.is_empty());
                if all_nullary {
                    out.insert(
                        decl.name.clone(),
                        DagTypeDecl::ClosedNullaryEnum { variants },
                    );
                } else {
                    out.insert(
                        decl.name.clone(),
                        DagTypeDecl::PayloadCoproduct {
                            variant_count: variants.len(),
                        },
                    );
                }
            }
            Connective::Conj => {
                // A field's TYPE is its child, exactly as a parameter's is. Reading
                // `type_annotation` here yielded no fields for every record, so each one was
                // dropped from the type environment and later refused as "not a closed type
                // declared by this authority" — false, and pointing at the wrong repair.
                // A RECORD FIELD'S TYPE IS IN `inferred`, NOT IN `children`. Measured, not
                // assumed -- the fourth node-shape fact this reader needed and the third one an
                // assumption got wrong. A PARAMETER's type is its child, which is true and stays
                // true; reading a field the same way returned nothing for every field of every
                // record, so `filter_map` emptied the list and the guard below dropped the whole
                // declaration. std.pareto read 6 of 13 types and ZERO of its 7 records.
                let mut fields: Vec<(String, String)> = Vec::new();
                let mut unreadable: Vec<String> = Vec::new();
                for f in decl.children.iter() {
                    match f.inferred.as_ref().map(|i| i.as_ref()) {
                        Some(crate::v1_std_core::InferredNode::Resolved { node }) => {
                            fields.push((f.name.clone(), type_text(node)))
                        }
                        _ => unreadable.push(f.name.clone()),
                    }
                }
                // A PARTIALLY READ RECORD IS NOT A RECORD. Enumerating the fields that happened
                // to resolve would produce a Cartesian product over a SUBSET of the record's
                // fields -- a constructor expression missing fields, which does not compile, and
                // a domain claim that is simply false. So the whole declaration refuses, naming
                // the fields responsible, rather than silently narrowing itself.
                if unreadable.is_empty() && !fields.is_empty() {
                    out.insert(decl.name.clone(), DagTypeDecl::Record { fields });
                } else {
                    out.insert(
                        decl.name.clone(),
                        DagTypeDecl::DeclaredNotEnumerable {
                            form: format!(
                                "record with {} field(s) whose declared type the reader could not \
                                 read: {}",
                                unreadable.len(),
                                unreadable.join(", ")
                            ),
                        },
                    );
                }
            }
            // NO WILDCARD. The declaration vocabulary is CLOSED, and a catch-all here discards
            // exactly the guarantee that closure exists to provide -- silently, in the seed, where
            // nothing enforces the exhaustiveness the substrate would. Every remaining form is
            // registered as declared-but-not-enumerable so it refuses by name instead of becoming
            // a false claim that the corpus does not declare it. Opaque types and aliases are the
            // bulk of this arm and are genuinely not enumerable; that answer is now SAID rather
            // than inferred from an absence.
            other => {
                out.insert(
                    decl.name.clone(),
                    DagTypeDecl::DeclaredNotEnumerable {
                        form: format!(
                            "declaration form {other:?} carries no enumerable constructor set"
                        ),
                    },
                );
            }
        }
    }
    out
}

/// The classes an `Int` parameter's own comparisons cut the integers into, with one
/// representative per class.
#[derive(Debug, Clone, PartialEq)]
struct IntPartition {
    /// The integer literals this parameter is compared against, sorted and deduplicated.
    literals: Vec<i64>,
    /// One value per equivalence class. For literals L the classes are each point `l` and each
    /// open gap between consecutive points, plus the two unbounded ends; the representative set
    /// below hits every non-empty one of them.
    representatives: Vec<i64>,
}

impl IntPartition {
    /// Representatives are `{l-1, l, l+1}` over every literal `l`.
    ///
    /// That this covers every class is a three-case check, not an approximation: the point class
    /// `{l}` is hit by `l`; the gap `(l_i, l_{i+1})` is hit by `l_i + 1` unless the gap is empty
    /// (`l_{i+1} == l_i + 1`); and the two unbounded ends are hit by `min-1` and `max+1`. The
    /// `±1` values are also exactly where an off-by-one divergence lives, which is why they are
    /// taken rather than an arbitrary interior point.
    fn from_literals(mut literals: Vec<i64>) -> IntPartition {
        literals.sort_unstable();
        literals.dedup();
        let mut reps: Vec<i64> = Vec::new();
        for l in &literals {
            for r in [l.saturating_sub(1), *l, l.saturating_add(1)] {
                if !reps.contains(&r) {
                    reps.push(r);
                }
            }
        }
        // No literals means the parameter is never compared against one. Reached only when it is
        // never mentioned at all -- any other use refuses upstream -- so the function ignores it
        // and a single arbitrary value covers the whole type.
        if reps.is_empty() {
            reps.push(0);
        }
        reps.sort_unstable();
        IntPartition {
            literals,
            representatives: reps,
        }
    }

    fn describe(&self) -> String {
        let lits = if self.literals.is_empty() {
            "unused by the body".to_string()
        } else {
            format!(
                "literals {{{}}}",
                self.literals
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        format!(
            "{lits}, reps {{{}}}",
            self.representatives
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

/// Derive the partition for one `Int` parameter from the BODY NODE that uses it, or REFUSE
/// naming the occurrence that defeated it.
///
/// This is the whole soundness argument in code. Every occurrence of the parameter must be an
/// operand of a comparison whose other operand is an integer literal. Everything else -- being
/// returned, passed as an argument, added, having a field read off it -- lets the parameter's
/// VALUE reach the output, at which point two members of one class stop agreeing and the
/// partition claim is false. `fn shard_count_positive(n: Int) -> Int { if n <= 0 { 1 } else { n } }`
/// compares `n` against a literal AND returns it, so it returns 5 for 5 and 7 for 7 while both
/// sit in the class `n > 0`. That function refuses here.
///
/// IT WALKS THE PARSED TREE, NOT TEXT. An earlier revision lexed the body itself, and carried the
/// apparatus that implies: a hand tokenizer, a whitelist of what may abut a literal operand so
/// that `a + 1 < n` refused instead of reading `1`, comment stripping so an annotation naming the
/// parameter was not read as a use of it, and a lambda-parameter scan because it could not model
/// scope. Every one of those was a workaround for not having the parse, and every one is deleted:
/// `ExprBinOp` already carries its operator, `ExprLiteral` already holds a typed `LitInt`, and a
/// rebound name is simply a different node. The tell that the old version was in the wrong layer
/// is that its bug -- misreading `a - 1 < n` -- cannot be expressed in this one.
/// WHY a parameter's domain could not be derived -- as a typed cause, not a sentence.
///
/// The census ranks refusals to decide what to ground next, and ranking needs the SUBJECT: the
/// type, or the class of obstacle. The first revision carried only a formatted message and ranked
/// on it, which put `x (used outside a literal comparison...)` at the top with 1500 -- every
/// parameter in the corpus that happens to be named `x`, collapsed into one row that names no
/// type and no work. A parameter name is not a unit of work; the string was doing double duty as
/// an identity and as prose, and it was wrong at the identity job.
///
/// `describe()` is derived from this, so the sentence and the ranking key cannot disagree.
#[derive(Clone, PartialEq, Eq, Hash)]
enum RefusalCause {
    UnboundedString {
        ty: String,
    },
    UnboundedSequence {
        ty: String,
    },
    PayloadCoproduct {
        ty: String,
        variants: usize,
    },
    DeclaredNotEnumerable {
        ty: String,
        form: String,
    },
    /// The type is declared SOMEWHERE in the corpus but is not visible from the module under
    /// plan -- an import-closure gap in the reader, not a property of the type.
    TypeNotVisibleHere {
        ty: String,
    },
    /// No module in the corpus declares this type. Genuinely outside what the authority carries.
    TypeNotDeclaredAnywhere {
        ty: String,
    },
    ProductTooLarge {
        ty: String,
    },
    NestedTooDeep {
        ty: String,
    },
    /// The Int class. Keyed WITHOUT the parameter name -- the name is in the message for locating
    /// it, never in the identity, or one class fragments into as many rows as there are spellings.
    IntValueEscapesComparison {
        param: String,
    },
    IntComparedToNonLiteral {
        param: String,
    },
    IntThroughContainer,
    IntNoBody {
        param: String,
    },
    TupleBudgetExceeded {
        param: String,
    },
    /// Every parameter's domain derived, and the product of them contains NO tuple -- so the
    /// function yields no call while looking, in a count of derivable functions, exactly like one
    /// that yields a thousand.
    ///
    /// COUNTED SEPARATELY FROM THE NEVER-DERIVABLE ONES ON PURPOSE: the remedies differ. A
    /// function refused for `List<T>` needs a length partition; a function with an empty product
    /// needs the enumerator that produced an empty value set fixed. Summing them would name one
    /// piece of work where there are two.
    ///
    /// REACHABILITY, STATED RATHER THAN IMPLIED: no enumerator in this fragment returns an empty
    /// value set today (a zero-variant coproduct is dropped before it becomes a `DagTypeDecl`, and
    /// a zero-field record enumerates to the one empty literal), so there is no live specimen in
    /// the corpus. The producer is the classification in `function_grain_coverage`, which is
    /// executed against this state by `empty_derived_domain_is_uncovered_not_covered`. It exists
    /// because a zero that reads as success is the exact failure this whole arm closes: a
    /// function counted as covered while contributing nothing to the transcript.
    EmptyDerivedDomain,
    /// A refusal reached through a record field, carrying the field path for locating and the
    /// UNDERLYING cause for ranking -- grounding the inner type unlocks the outer record too.
    ViaField {
        ty: String,
        field: String,
        inner: Box<RefusalCause>,
    },
}

impl RefusalCause {
    /// The subject the work would be done to. Ranking key: a `ViaField` ranks as its inner cause,
    /// because the fix is the inner type and counting the wrapper separately would split one
    /// piece of work across as many rows as there are records that embed it.
    fn subject(&self) -> String {
        match self {
            RefusalCause::UnboundedString { ty }
            | RefusalCause::UnboundedSequence { ty }
            | RefusalCause::PayloadCoproduct { ty, .. }
            | RefusalCause::DeclaredNotEnumerable { ty, .. }
            | RefusalCause::ProductTooLarge { ty }
            | RefusalCause::NestedTooDeep { ty } => ty.clone(),
            // THE KIND IS PART OF THE KEY for these two, because the two name different work and
            // the first revision of the split keyed on the bare type name -- so the ranked list
            // printed exactly what it printed before the split, and the whole correction was
            // invisible in its own output. `declared_anywhere` is corpus-global, so a given type
            // falls entirely into one bucket; the tag is therefore stable per type, not a source
            // of fragmentation.
            RefusalCause::TypeNotVisibleHere { ty } => {
                format!("{ty} [declared in corpus, NOT VISIBLE to the reader]")
            }
            RefusalCause::TypeNotDeclaredAnywhere { ty } => {
                format!("{ty} [undeclared anywhere in corpus]")
            }
            RefusalCause::IntValueEscapesComparison { .. } => {
                "Int (value escapes literal comparison)".to_string()
            }
            RefusalCause::IntComparedToNonLiteral { .. } => {
                "Int (compared against a non-literal)".to_string()
            }
            RefusalCause::IntThroughContainer => "Int (reached through a container)".to_string(),
            RefusalCause::IntNoBody { .. } => "Int (no attached body node)".to_string(),
            RefusalCause::TupleBudgetExceeded { .. } => {
                format!("(combination exceeds {MAX_TUPLES_PER_FUNCTION} tuples)")
            }
            RefusalCause::EmptyDerivedDomain => "(derived domain contains no values)".to_string(),
            RefusalCause::ViaField { inner, .. } => inner.subject(),
        }
    }

    fn describe(&self) -> String {
        match self {
            RefusalCause::UnboundedString { ty } => format!("{ty} (unbounded String domain)"),
            RefusalCause::UnboundedSequence { ty } => {
                format!("{ty} (unbounded sequence length; a length partition is not derived)")
            }
            RefusalCause::PayloadCoproduct { ty, variants } => format!(
                "{ty} (closed coproduct, but {variants} variants carry payloads whose domains are \
                 not enumerated)"
            ),
            // THESE TWO WERE ONE ARM, AND THE ONE ARM MISDIRECTED. It read "not a closed type
            // declared by this authority", which parses as "declared, but not closed" -- while
            // the branch is reached ONLY when the type is not in the type environment at all.
            // Node topped the corpus ranking at 798 under that label and was read, by me, as the
            // one big groundable item. It is nothing of the sort: it is a 20-field record with an
            // unbounded String, a recursive List<Node>, and self-reference. Splitting the arm is
            // what makes the difference between "my reader cannot see it" and "the corpus does
            // not have it" reportable, and only the first is work anyone can do.
            RefusalCause::DeclaredNotEnumerable { ty, form } => {
                format!("{ty} (declared, but not enumerable: {form})")
            }
            RefusalCause::TypeNotVisibleHere { ty } => format!(
                "{ty} (declared elsewhere in the corpus but not visible from this module -- an \
                 import-closure gap in the reader, not a property of the type)"
            ),
            RefusalCause::TypeNotDeclaredAnywhere { ty } => {
                format!("{ty} (no module in the corpus declares this type)")
            }
            RefusalCause::ProductTooLarge { ty } => format!(
                "{ty} (record product exceeds {MAX_TUPLES_PER_FUNCTION} values; refusing rather \
                 than sampling)"
            ),
            RefusalCause::NestedTooDeep { ty } => {
                format!("{ty} (nesting deeper than the fragment enumerates)")
            }
            RefusalCause::IntValueEscapesComparison { param } => format!(
                "{param} (used outside a literal comparison, so its value reaches the result and \
                 members of one class need not agree)"
            ),
            RefusalCause::IntComparedToNonLiteral { param } => format!(
                "{param} (compared against a non-literal; the partition is derived from literals, \
                 and a comparison against another parameter or a call would need a joint partition \
                 this fragment does not derive)"
            ),
            RefusalCause::IntThroughContainer => {
                "Int (reached through a container; a partition is \
                 derived from a parameter's own comparisons and does not follow into a field or \
                 element)"
                    .to_string()
            }
            RefusalCause::IntNoBody { param } => {
                format!("{param} (Int parameter on a function with no attached body node)")
            }
            RefusalCause::TupleBudgetExceeded { param } => format!(
                "{param} (corpus exceeds {MAX_TUPLES_PER_FUNCTION} tuples; refusing rather than \
                 sampling)"
            ),
            RefusalCause::EmptyDerivedDomain => {
                "every parameter derived, but their product contains no tuple, so this function \
                 yields no call at all"
                    .to_string()
            }
            RefusalCause::ViaField { ty, field, inner } => {
                format!("{ty}.{field}: {}", inner.describe())
            }
        }
    }
}

/// `Debug` IS `describe()`. A derived `Debug` would print the variant name, which is the one
/// rendering of a refusal that cannot be acted on -- and a test asserting on it would pin the
/// spelling of a Rust identifier rather than the fact.
impl std::fmt::Debug for RefusalCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.describe())
    }
}

fn derive_int_partition(
    param: &str,
    body: &crate::v1_std_core::Node,
) -> Result<IntPartition, RefusalCause> {
    let mut literals = Vec::new();
    visit_int_param_occurrences(param, body, None, &mut literals)?;
    Ok(IntPartition::from_literals(literals))
}

/// Recurse the body, carrying the enclosing comparison so an occurrence can be judged in context.
///
/// `enclosing` is `Some((op_is_comparison, sibling))` when this node is a direct operand of a
/// binary operation. An occurrence of the parameter is admitted ONLY when that context is a
/// comparison and the sibling is an integer literal.
fn visit_int_param_occurrences(
    param: &str,
    node: &crate::v1_std_core::Node,
    enclosing: Option<(bool, &crate::v1_std_core::Node)>,
    literals: &mut Vec<i64>,
) -> Result<(), RefusalCause> {
    use crate::std_syntax::BinOp;
    use crate::v1_std_core::ExprData;

    if matches!(node.expr_data.as_ref(), ExprData::ExprVar { .. }) && node.name == param {
        return match enclosing {
            Some((true, sibling)) => match int_literal_of(sibling) {
                Some(v) => {
                    literals.push(v);
                    Ok(())
                }
                None => Err(RefusalCause::IntComparedToNonLiteral {
                    param: param.to_string(),
                }),
            },
            _ => Err(RefusalCause::IntValueEscapesComparison {
                param: param.to_string(),
            }),
        };
    }

    let comparison = match node.expr_data.as_ref() {
        ExprData::ExprBinOp { op, .. } => Some(matches!(
            op,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge
        )),
        _ => None,
    };
    // A binary operation's two operands are each other's sibling; every other node's children
    // are visited with NO enclosing comparison, which is what makes the default a refusal.
    if let (Some(is_cmp), 2) = (comparison, node.children.len()) {
        let lhs = &node.children[0];
        let rhs = &node.children[1];
        visit_int_param_occurrences(param, lhs, Some((is_cmp, rhs)), literals)?;
        visit_int_param_occurrences(param, rhs, Some((is_cmp, lhs)), literals)?;
        return Ok(());
    }
    for child in node.children.iter() {
        visit_int_param_occurrences(param, child, None, literals)?;
    }
    for p in node.params.iter() {
        visit_int_param_occurrences(param, p, None, literals)?;
    }
    if let Some(b) = node.body.as_ref() {
        visit_int_param_occurrences(param, b, None, literals)?;
    }
    Ok(())
}

fn int_literal_of(node: &crate::v1_std_core::Node) -> Option<i64> {
    use crate::std_syntax::LiteralValue;
    use crate::v1_std_core::ExprData;
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitInt { value } => Some(*value),
            _ => None,
        },
        _ => None,
    }
}

/// The cap on one function's corpus. Exceeding it REFUSES rather than sampling: a receipt that
/// ran a subset while reporting the whole is the fabricated-plausible-output failure, and the
/// cheapest way to get one is a Cartesian product nobody bounded.
const MAX_TUPLES_PER_FUNCTION: usize = 4096;

/// Enumerate a parameter's domain as Rust expressions, or REFUSE naming the type that defeated it.
///
/// `module_alias` is the emitted mirror's module, so every constructor is written against the
/// artifact under test rather than against a guess about where a type lives.
fn enumerate_parameter_values(
    ty: &str,
    types: &std::collections::HashMap<String, DagTypeDecl>,
    declared_anywhere: &std::collections::HashSet<String>,
    depth: usize,
    int_partition: Option<&IntPartition>,
    module_alias: &str,
) -> Result<ParameterDomain, RefusalCause> {
    if depth > 4 {
        return Err(RefusalCause::NestedTooDeep { ty: ty.to_string() });
    }
    let ty = ty.trim();
    if ty == "Bool" {
        return Ok(ParameterDomain {
            values: vec!["false".to_string(), "true".to_string()],
            partition: None,
        });
    }
    if ty == "Int" {
        return match int_partition {
            Some(p) => Ok(ParameterDomain {
                values: p
                    .representatives
                    .iter()
                    .map(|r| format!("{r}i64"))
                    .collect(),
                partition: Some(p.describe()),
            }),
            None => Err(RefusalCause::IntThroughContainer),
        };
    }
    if ty.starts_with("List<") {
        return Err(RefusalCause::UnboundedSequence { ty: ty.to_string() });
    }
    match types.get(ty) {
        Some(DagTypeDecl::ClosedNullaryEnum { variants }) => Ok(ParameterDomain {
            values: variants
                .iter()
                .map(|v| format!("{module_alias}::{ty}::{v}"))
                .collect(),
            partition: None,
        }),
        Some(DagTypeDecl::PayloadCoproduct { variant_count }) => {
            Err(RefusalCause::PayloadCoproduct {
                ty: ty.to_string(),
                variants: *variant_count,
            })
        }
        Some(DagTypeDecl::DeclaredNotEnumerable { form }) => {
            Err(RefusalCause::DeclaredNotEnumerable {
                ty: ty.to_string(),
                form: form.clone(),
            })
        }
        Some(DagTypeDecl::Record { fields }) => {
            // The record's own domain is the Cartesian product of its fields'. Built as literal
            // constructor expressions so the driver names every field, which is also what makes a
            // field added upstream a COMPILE error in the driver rather than a silent default.
            let mut acc: Vec<Vec<(String, String)>> = vec![Vec::new()];
            let mut partitioned = Vec::new();
            for (fname, fty) in fields {
                let d = enumerate_parameter_values(
                    fty,
                    types,
                    declared_anywhere,
                    depth + 1,
                    None,
                    module_alias,
                )
                .map_err(|e| RefusalCause::ViaField {
                    ty: ty.to_string(),
                    field: fname.clone(),
                    inner: Box::new(e),
                })?;
                if let Some(pt) = d.partition.clone() {
                    partitioned.push(format!("{fname}: {pt}"));
                }
                let mut next = Vec::new();
                for prefix in &acc {
                    for v in &d.values {
                        if next.len() > MAX_TUPLES_PER_FUNCTION {
                            return Err(RefusalCause::ProductTooLarge { ty: ty.to_string() });
                        }
                        let mut row = prefix.clone();
                        row.push((fname.clone(), v.clone()));
                        next.push(row);
                    }
                }
                acc = next;
            }
            Ok(ParameterDomain {
                values: acc
                    .into_iter()
                    .map(|row| {
                        let inner: Vec<String> =
                            row.into_iter().map(|(f, v)| format!("{f}: {v}")).collect();
                        format!("{module_alias}::{ty} {{ {} }}", inner.join(", "))
                    })
                    .collect(),
                partition: if partitioned.is_empty() {
                    None
                } else {
                    Some(partitioned.join("; "))
                },
            })
        }
        None => Err(if ty == "String" || ty == "NonEmptyStr" {
            RefusalCause::UnboundedString { ty: ty.to_string() }
        } else if declared_anywhere.contains(ty) {
            RefusalCause::TypeNotVisibleHere { ty: ty.to_string() }
        } else {
            RefusalCause::TypeNotDeclaredAnywhere { ty: ty.to_string() }
        }),
    }
}

/// One function as the authority declares it: its parameters and the body node that uses them.
///
/// The body is carried because an `Int` parameter's domain is a fact about HOW THIS FUNCTION USES
/// IT, not about the type: `Int` has no finite domain, but the comparisons a body performs cut it
/// into finitely many classes. A signature-only planner cannot ask that question at all.
#[derive(Debug, Clone)]
struct DagFnSignature {
    name: String,
    params: Vec<(String, String)>,
    body: Option<Rc<crate::v1_std_core::Node>>,
}

/// Read every function off a parsed module node.
///
/// The third and last hand-rolled reader to go. Its predecessor matched `fn ` at the start of a
/// line and then took the parameter list up to the first `)` on that same line, so a signature
/// spanning lines produced no entry at all -- 14 of `v1.compiler.emit_rust`'s 631 went missing
/// that way, and only a declared-versus-parsed counter made the gap visible rather than reading
/// as a module with a smaller surface. The parser has already separated a declaration's
/// parameters from its body, so none of that arises here.
///
/// That same counter then caught THIS function selecting on the wrong discriminator, which is why
/// it is kept rather than retired once the parser owned the read: two independent readers of one
/// fact disagreeing is the only cheap signal that one of them is wrong.
fn fn_signatures_from_module(module: &crate::v1_std_core::Node) -> Vec<DagFnSignature> {
    let mut out = Vec::new();
    for decl in module.children.iter() {
        // A declaration is a FUNCTION exactly when it carries a body. Measured against the live
        // tree rather than assumed: `std.pareto`'s items report `compare_int` and friends as
        // `conn=NoConnective children=0 params=N body=true`, while its types report `Disj` with
        // variants or `Conj` with fields and no body. `Connective::Arrow` — which an earlier
        // revision selected on — marks a `Callable` TYPE EXPRESSION, not a declaration, so that
        // filter matched nothing and every module reported `parsed=0`.
        // A FUNCTION carries a body AND a resolved return type in `inferred`. A `data` row also
        // carries a body, which is why selecting on the body alone over-counted: `std.pareto`
        // reported 36 parsed against 33 authored `fn` lines, its three `data` rows swept in.
        // Measured discriminator — `data no_names: List<NonEmptyStr> = []` reports
        // `ta=Some("List") inf=none`, while every function reports `ta=None inf=Resolved{..}`:
        // the declared type of a constant lives in `type_annotation`, a function's return type in
        // `inferred`.
        let (Some(_), Some(_)) = (decl.body.as_ref(), decl.inferred.as_ref()) else {
            continue;
        };
        // A parameter's TYPE is its single CHILD, not a `type_annotation`. Measured: every
        // parameter in `std.pareto` reports `ta=None children=1`. An earlier revision read
        // `type_annotation`, got `None` for every parameter, and derived the empty string as the
        // type name — so all 514 corpus refusals named the SAME empty type and the blocker
        // histogram collapsed to one meaningless row. A refusal that names nothing ranks nothing,
        // which is the defect this whole fragment exists to remove, reproduced in its purest form.
        let params: Vec<(String, String)> = decl
            .params
            .iter()
            .map(|p| {
                let ty = p
                    .children
                    .iter()
                    .next()
                    .map(|t| type_text(t))
                    .unwrap_or_else(|| "<parameter with no type node>".to_string());
                (p.name.clone(), ty)
            })
            .collect();
        out.push(DagFnSignature {
            name: decl.name.clone(),
            params,
            body: decl.body.clone(),
        });
    }
    out
}

/// What the fragment can say about one module's surface.
struct ModuleCorpusPlan {
    module_path: String,
    /// Functions whose every parameter domain derived, with the combined domain per function AND
    /// the argument tuples that domain actually consists of. The tuples are what the driver runs;
    /// carrying only the domain description is how an earlier revision could report a corpus it
    /// had never executed.
    derivable: Vec<(String, EnumeratedDomain, Vec<Vec<String>>)>,
    /// Functions that defeated derivation, each naming the type responsible.
    refused: Vec<(String, RefusalCause)>,
    /// `fn` lines the authority declares vs signatures actually parsed. These must agree; a gap
    /// means the parser missed a form, and reporting the pair is what stops a silent miss from
    /// reading as a module with a small surface.
    declared_fn_lines: usize,
    parsed_signatures: usize,
}

/// One module's surface partitioned AT FUNCTION GRAIN: which declared functions actually yield a
/// call, and which yield none and why.
///
/// This is the fact the receipt is denominated in. `ModuleCorpusPlan::derivable` is close to it
/// but not equal to it: a function whose parameter domains all derived while their product is
/// empty sits in `derivable` and contributes nothing, so counting that vector counts a coverage
/// claim rather than coverage.
#[derive(Debug)]
struct FunctionGrainCoverage {
    /// Functions that yield at least one call, with how many.
    covered: Vec<(String, usize)>,
    /// Functions that yield none, each with the cause. `RefusalCause::EmptyDerivedDomain`
    /// distinguishes "derived, but to nothing" from "did not derive".
    uncovered: Vec<(String, RefusalCause)>,
}

impl FunctionGrainCoverage {
    fn calls(&self) -> usize {
        self.covered.iter().map(|(_, n)| n).sum()
    }
}

/// The partition itself. Pure over the plan, which is what lets both arms of it be executed
/// against hand-built states rather than only against whatever the live corpus happens to hold.
fn function_grain_coverage(plan: &ModuleCorpusPlan) -> FunctionGrainCoverage {
    let mut covered = Vec::new();
    let mut uncovered = Vec::new();
    for (name, _domain, tuples) in &plan.derivable {
        if tuples.is_empty() {
            uncovered.push((name.clone(), RefusalCause::EmptyDerivedDomain));
        } else {
            covered.push((name.clone(), tuples.len()));
        }
    }
    for (name, cause) in &plan.refused {
        uncovered.push((name.clone(), cause.clone()));
    }
    FunctionGrainCoverage { covered, uncovered }
}

/// THE PLAN-GRAIN HALF OF THE ONE SELECTION CRITERION: is there a subject to compare?
///
/// `Ok(None)` selects. `Ok(Some(_))` excludes, typed and counted. `Err(_)` refuses, because the
/// third state is the reader's own blindness and it is not a property of the authority.
///
/// The three answers used to be two, and the missing distinction is the whole point. `zero
/// functions covered` was one arm covering three different states -- an authority with no
/// functions at all, an authority whose functions all defeat derivation, and a parse this
/// fragment failed to read -- with one message that was false about two of them. That is the
/// state-space conflation DESIGN.md names: one value standing for several states whose remedies
/// differ. Splitting it costs nothing and makes each population rankable on its own terms.
fn plan_grain_selection(
    plan: &ModuleCorpusPlan,
    coverage: FunctionGrainCoverage,
) -> Result<Option<ReceiptExclusion>, String> {
    // THE READER'S OWN BLINDNESS IS NOT A PROPERTY OF THE AUTHORITY. The `fn ` line count and the
    // parsed signature count are two readers of one fact, and the pair is carried precisely so a
    // disagreement is visible. When the parser sees none of the functions the source declares,
    // `zero covered` is IGNORANCE rather than the fact that nothing is derivable, and excluding
    // on it would publish a reader deficit as a fact about the module.
    if plan.parsed_signatures == 0 && plan.declared_fn_lines > 0 {
        return Err(format!(
            "behavioral-receipt: {} declares {} `fn ` line(s) and the parser produced no \
             signature at all. The two readers disagree, so this selection cannot say whether \
             the module has no derivable function or whether this fragment simply failed to read \
             it -- and those have opposite remedies. Refusing rather than excluding: an exclusion \
             here would publish the reader's blindness as a fact about the authority",
            plan.module_path, plan.declared_fn_lines
        ));
    }
    if plan.parsed_signatures == 0 {
        return Ok(Some(ReceiptExclusion::NoFunctionDeclared {
            module_path: plan.module_path.clone(),
        }));
    }
    if coverage.covered.is_empty() {
        return Ok(Some(ReceiptExclusion::NoFunctionHasACorpus {
            module_path: plan.module_path.clone(),
            uncovered: coverage.uncovered,
        }));
    }
    Ok(None)
}

/// Does a plan-grain exclusion SURVIVE what the generated-artifact population says about the
/// mirror? `None` means it does not -- the module is selected after all.
///
/// PURE, AND SEPARATED FROM THE ASK FOR ONE REASON: the divergence it decides has an EMPTY
/// POPULATION today, so nothing in the live corpus executes the branch that matters. An empty
/// population is not a safe place to leave a behaviour undecided -- the first real member would
/// settle it by accident, and whoever met it would be debugging a difference nobody had written
/// down. Pulling the decision out of the loop lets a planted control state the intent and execute
/// it permanently (DESIGN.md 4b(4): the evidence stays enrolled, it does not retire when the
/// production population catches up).
fn exclusion_survives_generated_artifact_population(
    exclusion: ReceiptExclusion,
    body: &GeneratedArtifactPathBody,
) -> Option<ReceiptExclusion> {
    match body {
        // A POSITIVE ANSWER: this path is a module mirror, not a generated artifact. Only here is
        // "no function yields a call" the whole story, so only here is the exclusion a fact.
        GeneratedArtifactPathBody::NotGenerated => Some(exclusion),
        // Produced OR Refused: either way the path belongs to the generated-artifact population,
        // whose subject is BYTES rather than behaviour -- and for that subject a declared function
        // is not required, because the identity check calls nothing. The differential loop is
        // where that population's answer is reported (identity, drift, or a generator refusal);
        // deciding it here would be a second adjudicator of one question.
        GeneratedArtifactPathBody::Produced(_) | GeneratedArtifactPathBody::Refused(_) => None,
    }
}

/// A plan CARRYING AT LEAST ONE CALL. There is no other way to build one.
///
/// The differential used to check this itself -- `if total == 0 { Refused }` -- which is
/// validation of a state the caller was free to construct (DESIGN.md section 5: prefer making the
/// bad state unwritable to flagging it afterwards). Selection now decides admission at function
/// grain, and hands the differential a value that cannot represent an empty corpus, so the
/// comparison-over-nothing the old guard existed to catch has no spelling. The guard is deleted
/// rather than kept beside the new arm: two answers to one question is the dual authority
/// DESIGN.md section 3 forbids, and the surviving one would be the one that never runs.
struct AdmittedPlan<'a> {
    plan: &'a ModuleCorpusPlan,
    coverage: FunctionGrainCoverage,
}

impl<'a> AdmittedPlan<'a> {
    /// `Err` carries the function-grain partition of a module no function of which yields a call
    /// -- the rows the exclusion is built from, so the refusal and the report read the same fact.
    fn of(plan: &'a ModuleCorpusPlan) -> Result<AdmittedPlan<'a>, FunctionGrainCoverage> {
        let coverage = function_grain_coverage(plan);
        if coverage.covered.is_empty() {
            Err(coverage)
        } else {
            Ok(AdmittedPlan { plan, coverage })
        }
    }
}

/// Every `.dag` module reachable from the source roots, keyed by its declared module path.
fn collect_dag_module_sources(
    source_roots: &[String],
) -> Result<std::collections::HashMap<String, String>, String> {
    let workspace = crate::cli_run::workspace_root();
    let mut out = std::collections::HashMap::new();
    let mut stack: Vec<PathBuf> = source_roots.iter().map(|r| workspace.join(r)).collect();
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(mp) = crate::cli_run::extract_module_path_public(&content) {
                        out.insert(mp, content);
                    }
                }
            }
        }
    }
    Ok(out)
}

/// The types VISIBLE to a module: its own declarations plus those of its transitive imports.
///
/// THIS RESOLVES, IT DOES NOT WIDEN, and the distinction is the whole risk of this function.
/// Reaching further to find a declaration feels like covering more, and the refusal count can
/// quietly drop for the wrong reason. So what is found is put through `derive_parameter_domain`
/// UNCHANGED: a record with a `String` field still refuses, a `String where` refinement still
/// refuses, a coproduct with payload variants still refuses. The only thing that changes is
/// WHERE a declaration may be found — `Ordering` is `Less | Equal | Greater` whether it is
/// declared in this module or in `std.algebra`, and refusing it for its address rather than its
/// shape was an artifact of the implementation, not a property of the fragment.
///
/// The control for that claim is external and specific: `std.content_hash` refuses 26 of its 27
/// functions before that change, and must still refuse 26 of 27 after it. If that number moves,
/// this resolved nothing and widened something.
///
/// BOTH READS GO THROUGH THE GRAMMAR. An earlier revision found declarations with a line-based
/// reader and found imports with `line.strip_prefix("import ")`. Each was a second implementation
/// of something the parser already owns, and the first was measurably wrong -- blind to 88% of
/// the corpus's type declarations. They are gone rather than kept working: a hand reader that
/// currently agrees with the grammar has exactly the standing the line reader had until it was
/// measured. Imports now come from the module node's `params` and declarations from its
/// `children`, which is where the parser puts them.
///
/// A module that will not parse REFUSES the whole plan rather than contributing nothing. Silently
/// skipping it would make an unreadable import indistinguishable from an import that declares no
/// types, and the second is a fine reason to derive nothing while the first is not.
fn visible_type_decls(
    module_path: &str,
    source: &str,
    modules: &std::collections::HashMap<String, String>,
) -> Result<std::collections::HashMap<String, DagTypeDecl>, String> {
    let mut merged = std::collections::HashMap::new();
    let mut seen = std::collections::HashSet::new();
    let mut queue = vec![(module_path.to_string(), source.to_string())];
    let mut depth_guard = 0usize;
    while let Some((mp, src)) = queue.pop() {
        depth_guard += 1;
        if depth_guard > 4096 {
            return Err(format!(
                "{module_path}: import closure exceeded 4096 modules; refusing rather than \
                 reporting a partial type environment"
            ));
        }
        if !seen.insert(mp.clone()) {
            continue;
        }
        let node = parse_dag_module_node(&format!("{mp}.dag"), &src)?;
        // A module's OWN declarations win: a local name shadows an imported one, and taking the
        // import would answer with a different type than the module compiles against.
        for (name, decl) in type_decls_from_module(&node) {
            merged.entry(name).or_insert(decl);
        }
        for imp in node.params.iter() {
            if let Some(next_src) = modules.get(&imp.name) {
                queue.push((imp.name.clone(), next_src.clone()));
            }
        }
    }
    Ok(merged)
}

/// The one entry point. An earlier revision also had a module-local planner that consulted only
/// the types declared in the module under plan; it is DELETED rather than kept beside this one.
/// Two resolvers over one question would answer differently for any imported type — which is the
/// exact defect this change fixes — and keeping the narrower one available is how a caller
/// silently gets the old answer back.
/// Every type name any module in the corpus declares.
///
/// Exists to separate two states a single refusal arm used to conflate: a type the reader could
/// not SEE from this module, and a type the corpus does not HAVE. Only the first is work.
fn declared_type_names(
    modules: &std::collections::HashMap<String, String>,
) -> Result<std::collections::HashSet<String>, String> {
    let mut out = std::collections::HashSet::new();
    for (mp, src) in modules {
        let node = parse_dag_module_node(&format!("{mp}.dag"), src)?;
        for (name, _decl) in type_decls_from_module(&node) {
            out.insert(name);
        }
    }
    Ok(out)
}

fn plan_module_corpus(
    module_path: &str,
    source: &str,
    module: &crate::v1_std_core::Node,
    types: &std::collections::HashMap<String, DagTypeDecl>,
    declared_anywhere: &std::collections::HashSet<String>,
    module_alias: &str,
) -> ModuleCorpusPlan {
    let sigs = fn_signatures_from_module(module);
    // Kept as a CROSS-CHECK, not as the source of truth. The authored `fn ` line count and the
    // parsed function count come from two different readers, so a disagreement means one of them
    // is wrong -- which is exactly how the line reader's 14 missing signatures were found. Now
    // that the parser owns the read they should agree, and the pair is reported so that a future
    // divergence is visible rather than silently halving a module's surface.
    let declared_fn_lines = source
        .lines()
        .filter(|l| l.trim_start().starts_with("fn "))
        .count();
    let mut derivable = Vec::new();
    let mut refused = Vec::new();
    for sig in &sigs {
        let mut partitioned = false;
        let mut partitions = Vec::new();
        let mut failure: Option<RefusalCause> = None;
        // Tuples are accumulated as a Cartesian product across parameters. A zero-parameter
        // function has exactly one tuple -- the empty one -- which is a real call, not an absence.
        let mut tuples: Vec<Vec<String>> = vec![Vec::new()];
        for (pname, pty) in &sig.params {
            // An `Int` parameter is asked about its own occurrences first. The partition IS its
            // domain, so a refusal here refuses the function -- there is no fallback window to
            // drop to, by design.
            let int_partition = if pty.trim() == "Int" {
                // No body means no occurrences to justify a partition. That REFUSES rather than
                // defaulting to "unused, so one value covers it": a declaration whose body the
                // parser did not attach is an unknown, and treating an unknown as an empty set of
                // uses is the narrow that turns "I could not see" into "there was nothing there".
                let Some(body) = sig.body.as_ref() else {
                    failure = Some(RefusalCause::IntNoBody {
                        param: pname.clone(),
                    });
                    break;
                };
                match derive_int_partition(pname, body) {
                    Ok(p) => Some(p),
                    Err(e) => {
                        failure = Some(e);
                        break;
                    }
                }
            } else {
                None
            };
            match enumerate_parameter_values(
                pty,
                types,
                declared_anywhere,
                0,
                int_partition.as_ref(),
                module_alias,
            ) {
                Ok(d) => {
                    if let Some(pt) = d.partition.clone() {
                        partitioned = true;
                        partitions.push(format!("{pname}: {pt}"));
                    }
                    let mut next: Vec<Vec<String>> = Vec::new();
                    for prefix in &tuples {
                        for v in &d.values {
                            if next.len() >= MAX_TUPLES_PER_FUNCTION {
                                break;
                            }
                            let mut row = prefix.clone();
                            row.push(v.clone());
                            next.push(row);
                        }
                    }
                    if next.len() >= MAX_TUPLES_PER_FUNCTION {
                        failure = Some(RefusalCause::TupleBudgetExceeded {
                            param: pname.clone(),
                        });
                        break;
                    }
                    tuples = next;
                }
                Err(offending) => {
                    failure = Some(offending);
                    break;
                }
            }
        }
        match failure {
            Some(offending) => refused.push((sig.name.clone(), offending)),
            None => {
                // The reported cardinality IS the number of tuples that will run. Deriving it
                // separately would let the report and the execution disagree.
                let cardinality = tuples.len();
                let domain = if partitioned {
                    EnumeratedDomain::ExhaustiveOverDerivedPartition {
                        cardinality,
                        partition: partitions.join("; "),
                    }
                } else {
                    EnumeratedDomain::Exhaustive { cardinality }
                };
                derivable.push((sig.name.clone(), domain, tuples));
            }
        }
    }
    ModuleCorpusPlan {
        module_path: module_path.to_string(),
        derivable,
        refused,
        declared_fn_lines,
        parsed_signatures: sigs.len(),
    }
}

/// The verdict for one candidate module. Three arms, and the third is not a soft pass.
#[derive(Debug, Clone, PartialEq)]
enum ReceiptVerdict {
    /// Both builds ran the derived corpus and every call THAT COULD BE COMPARED agreed.
    ///
    /// `nondeterministic_calls` is carried on this arm rather than printed beside it because a
    /// green with an excluded population is a DIFFERENT claim from a green over everything, and
    /// separating the two would let the weaker one be read as the stronger. Zero is the ordinary
    /// case and reads as the full claim.
    Equivalent {
        calls: usize,
        nondeterministic_calls: usize,
        nondeterministic_functions: Vec<String>,
    },
    /// Both builds ran and at least one call disagreed. The first difference is carried because
    /// a count alone cannot be acted on.
    Divergent {
        calls: usize,
        first_difference: String,
    },
    /// The comparison could NOT be taken. Never reported as equivalence: an emit that failed, or
    /// a driver that would not compile, is ignorance, and rendering ignorance as the clean verdict
    /// is the empty-observation narrow. A corpus with nothing in it is NOT among these any more --
    /// it never reaches the differential, because `AdmittedPlan` cannot carry it.
    Refused { reason: String },
    /// EVERY derived call in this module renders unstably, so no comparison exists to take.
    ///
    /// Not `Equivalent` (nothing was compared) and not `Divergent` (nothing disagreed about the
    /// program). Not `Refused` either: a refusal in this fragment means the measurement could not
    /// be attempted, whereas this one WAS attempted and produced a well-defined result -- the
    /// subject is unaskable, and the fix lives in emission rather than in this gate or in the
    /// diff under test.
    NondeterministicRendering {
        unstable_calls: usize,
        functions: Vec<String>,
    },
}

/// Generate the driver: one `println!` per call in the derived corpus.
///
/// The transcript line carries the function name and the argument expressions as authored, so a
/// divergence names the exact call rather than an index into a product nobody can reconstruct.
/// Output goes through `{:?}`, which is why the fragment admits only types the emitted mirror
/// derives `Debug` on.
fn generate_receipt_driver(module_alias: &str, plan: &ModuleCorpusPlan) -> String {
    let mut out = String::new();
    out.push_str("// GENERATED by the behavioral-receipt host realization. Do not edit.\n");
    // FULLY QUALIFIED, with no `use ... as m` alias. An earlier revision aliased the module for
    // calls while the enumerated constructor values were rendered against the bare module name,
    // so the driver referred to the same module two ways and only one of them resolved. One
    // spelling, produced in one place, cannot drift from itself.
    out.push_str("fn main() {\n");
    for (name, _domain, tuples) in &plan.derivable {
        for args in tuples {
            let call = format!("{module_alias}::{name}({})", args.join(", "));
            let shown = args.join(", ").replace('"', "\\\"");
            out.push_str(&format!(
                "    println!(\"{name}({shown}) = {{:?}}\", {call});\n"
            ));
        }
    }
    out.push_str("}\n");
    out
}

/// One driver's output, plus WHICH of its lines are not a function of the program alone.
///
/// A line is `unstable` when two executions of the SAME binary printed different text for it.
/// That is proof, not inference: the code, the inputs and the build are identical across the two
/// runs, so anything that differs came from somewhere other than the program's meaning. In this
/// corpus the somewhere is `HashMap`/`HashSet` iteration order reaching `{:?}`, whose seed is
/// randomized per process -- measured at 20 distinct transcripts over 20 executions of one
/// unchanged binary for `std.algebra::kernel_algebra_profile_value`.
///
/// WHY THIS IS MEASURED RATHER THAN DECIDED FROM THE TYPE. Order-dependent rendering is a
/// property of the value's TRANSITIVE shape: a record CONTAINING a map renders nondeterministically
/// while its own return type says `Record`. Any check keying on the outermost constructor
/// under-refuses by construction, and it under-refuses SILENTLY -- the missed call lands in
/// `Divergent`, indistinguishable from a real divergence. Running the binary twice keys on the
/// property itself, so there is no type walk to keep in sync with the corpus.
///
/// THE RESIDUE, STATED: two randomized renderings can coincide, so a subject that happened to
/// agree twice is not caught. That makes every count derived from this a FLOOR, and the floor is
/// printed as such rather than left in this comment. It is a residue that SHRINKS with more
/// executions, unlike a structural blind spot, but this is not an argument for adding runs
/// speculatively -- one extra run is what the evidence to date justifies.
#[derive(Debug, Clone, PartialEq)]
struct DriverTranscript {
    lines: Vec<String>,
    /// Indices into `lines` that differed between the two executions.
    unstable: std::collections::BTreeSet<usize>,
}

impl DriverTranscript {
    fn of(first: Vec<String>, second: Vec<String>) -> Self {
        // A length difference between two runs of one binary is itself instability, and it is
        // not attributable to any single index -- so every line of the longer run is marked
        // rather than none. Silently comparing the common prefix would hide it.
        let unstable = if first.len() != second.len() {
            (0..first.len().max(second.len())).collect()
        } else {
            first
                .iter()
                .zip(second.iter())
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .map(|(i, _)| i)
                .collect()
        };
        Self {
            lines: first,
            unstable,
        }
    }
}

/// Build the crate as it currently stands, compile the driver against it, and return the
/// transcript. Both halves of the differential go through THIS function, so the two transcripts
/// cannot differ because of how they were produced.
/// WHICH crate the driver is built against and linked to.
///
/// These three were literals inside `run_receipt_driver`. A hardwired `-p v1-compiler` is policy
/// standing inside a mechanism (DESIGN §3: an argv carrying a literal it should receive as a
/// parameter), and it is exactly what made the mode impossible to exercise against anything but
/// the production mirror -- so its own arms could only ever be run by a human with a script.
/// Parameterised, the control and the production path are ONE code path with one argument
/// different, rather than two machineries that could drift apart while both looked green.
struct ReceiptCrate {
    package: String,
    extern_name: String,
    rlib: String,
}

impl ReceiptCrate {
    fn v1_compiler() -> Self {
        Self {
            package: "v1-compiler".to_string(),
            extern_name: "v1_compiler".to_string(),
            rlib: "libv1_compiler.rlib".to_string(),
        }
    }

    fn receipt_fixture() -> Self {
        Self {
            package: "v1-receipt-fixture".to_string(),
            extern_name: "v1_receipt_fixture".to_string(),
            rlib: "libv1_receipt_fixture.rlib".to_string(),
        }
    }
}

fn run_receipt_driver(
    workspace: &std::path::Path,
    krate: &ReceiptCrate,
    driver_src: &str,
    label: &str,
) -> Result<DriverTranscript, String> {
    let drv_dir = workspace.join("target/behavioral-receipt");
    fs::create_dir_all(&drv_dir).map_err(|e| format!("create {}: {e}", drv_dir.display()))?;
    let src_path = drv_dir.join(format!("driver_{label}.rs"));
    fs::write(&src_path, driver_src).map_err(|e| format!("write driver: {e}"))?;

    let lib = Command::new("cargo")
        .args(["build", "--release", "-p", &krate.package, "--lib"])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn cargo ({label}): {e}"))?;
    if !lib.status.success() {
        return Err(format!(
            "{label}: the crate did not build; the candidate is not admissible without a compile. \
             {}",
            String::from_utf8_lossy(&lib.stderr)
                .lines()
                .filter(|l| l.starts_with("error"))
                .take(4)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    let bin_path = drv_dir.join(format!("driver_{label}"));
    let rustc = Command::new("rustc")
        .args(["--edition", "2021", "-O"])
        .arg(&src_path)
        .arg("--extern")
        .arg(format!(
            "{}={}",
            krate.extern_name,
            workspace.join("target/release").join(&krate.rlib).display()
        ))
        .arg("-L")
        .arg(workspace.join("target/release/deps"))
        .arg("-o")
        .arg(&bin_path)
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn rustc ({label}): {e}"))?;
    if !rustc.status.success() {
        return Err(format!(
            "{label}: the driver did not compile against the mirror: {}",
            String::from_utf8_lossy(&rustc.stderr)
                .lines()
                .filter(|l| l.starts_with("error"))
                .take(4)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    // TWICE, AND THE SECOND RUN IS THE POINT. The two are executions of the SAME BINARY -- the
    // cargo build and the rustc compile above are already paid, so this costs milliseconds and
    // not a rebuild. Two executions of one unchanged binary that disagree PROVE the disagreeing
    // call renders nondeterministically, which is the only way to learn that fact: it is a
    // property of the value's rendering, not of its type, so nothing before the run can know it.
    let first = run_driver_binary(workspace, &bin_path, label)?;
    let second = run_driver_binary(workspace, &bin_path, label)?;
    Ok(DriverTranscript::of(first, second))
}

fn run_driver_binary(
    workspace: &std::path::Path,
    bin_path: &std::path::Path,
    label: &str,
) -> Result<Vec<String>, String> {
    let run = Command::new(bin_path)
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn driver ({label}): {e}"))?;
    if !run.status.success() {
        return Err(format!(
            "{label}: the driver ran and exited {}; a corpus call panicking is a behavioural fact, \
             but it is not one this comparison can attribute, so it refuses",
            run.status
        ));
    }
    Ok(String::from_utf8_lossy(&run.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

/// The two-build differential for ONE candidate module.
///
/// Seed transcript first, from the tree as committed. Then the emitted candidate is written over
/// its mirror, the crate is rebuilt, and the SAME driver runs again. The mirror is restored on
/// every path, including failure -- leaving a candidate in the tree would make the next reader's
/// measurement a lie.
///
/// This is what the whole fragment is for. CI proves the committed mirrors equal what the
/// authority emits and that the emit repeats; it never compiles the candidate, let alone runs it.
/// A byte comparison cannot distinguish a rename from a semantic change, and DESIGN §7 says a
/// byte-identical fixed point is explicitly NOT the goal -- behavioural equivalence on a
/// discriminating corpus is.
fn behavioral_differential(
    workspace: &std::path::Path,
    krate: &ReceiptCrate,
    mirror_path: &std::path::Path,
    candidate_source: &str,
    admitted: &AdmittedPlan<'_>,
    module_alias: &str,
) -> ReceiptVerdict {
    // No emptiness check: `AdmittedPlan` cannot be built from a plan with no call in it, so the
    // comparison-over-nothing this function used to guard against is unrepresentable here rather
    // than rejected here.
    let plan = admitted.plan;
    let driver = generate_receipt_driver(module_alias, plan);
    let shown = mirror_path.display().to_string();

    // THE SEED MUST ACTUALLY BE THE SEED (review 54094). This function installs a candidate over
    // the committed mirror and restores it on every path -- but a process killed between the two
    // leaves the candidate in the tree, and the NEXT run would then read it as the committed
    // bytes and compare the candidate against itself. That comparison answers EQUIVALENT, which
    // is the worst available wrong answer: a green that means nothing, produced by a mechanism
    // whose entire job is to be trusted.
    //
    // The residue cannot be prevented -- no arrangement of writes survives SIGKILL -- so it is
    // made LOUD instead. A dirty path is a refusal, not a warning, and it names the recovery.
    match git_stdout(
        workspace,
        &[
            "status",
            "--porcelain",
            "--",
            &mirror_path.to_string_lossy(),
        ],
    ) {
        Ok(out) if !out.trim().is_empty() => {
            return ReceiptVerdict::Refused {
                reason: format!(
                    "{shown} is modified in the working tree, so the bytes this would read as the \
                     SEED are not the committed bytes. Most likely a previous run was killed \
                     between installing a candidate and restoring it -- in which case this run \
                     would compare that candidate against itself and answer EQUIVALENT. Restore \
                     it (`git checkout -- {shown}`) before measuring: {}",
                    out.trim()
                ),
            }
        }
        Ok(_) => {}
        Err(e) => {
            return ReceiptVerdict::Refused {
                reason: format!(
                    "cannot determine whether {shown} is clean ({e}), so cannot establish that the \
                     seed transcript comes from committed bytes"
                ),
            }
        }
    }
    let committed = match fs::read_to_string(mirror_path) {
        Ok(c) => c,
        Err(e) => {
            return ReceiptVerdict::Refused {
                reason: format!("read {shown}: {e}"),
            }
        }
    };

    let seed = match run_receipt_driver(workspace, krate, &driver, "seed") {
        Ok(t) => t,
        Err(e) => return ReceiptVerdict::Refused { reason: e },
    };

    if let Err(e) = fs::write(mirror_path, candidate_source) {
        return ReceiptVerdict::Refused {
            reason: format!("install candidate {shown}: {e}"),
        };
    }
    let candidate = run_receipt_driver(workspace, krate, &driver, "candidate");
    // Restore BEFORE interpreting the result, so no early return can leave the candidate in place.
    if let Err(e) = fs::write(mirror_path, &committed) {
        return ReceiptVerdict::Refused {
            reason: format!(
                "restore {shown} after the candidate build: {e}. The tree may still hold \
                 the candidate; do not trust a later measurement without checking"
            ),
        };
    }
    let candidate = match candidate {
        Ok(t) => t,
        Err(e) => return ReceiptVerdict::Refused { reason: e },
    };

    if seed.lines.len() != candidate.lines.len() {
        return ReceiptVerdict::Divergent {
            calls: seed.lines.len(),
            first_difference: format!(
                "transcript lengths differ: seed {} lines, candidate {} lines",
                seed.lines.len(),
                candidate.lines.len()
            ),
        };
    }
    // THE UNION, NOT THE SEED'S SET ALONE. Each side is its own binary and each was measured
    // independently, so a call can render unstably in one and (by coincidence, on that pair of
    // runs) stably in the other. Comparing a line either side proved unstable would score a
    // coin flip as a behavioural difference, which is the fabricated-difference failure this
    // whole change exists to remove.
    let unstable: std::collections::BTreeSet<usize> =
        seed.unstable.union(&candidate.unstable).copied().collect();
    let excluded = nondeterministic_call_functions(admitted, &unstable);

    let compared: Vec<(usize, (&String, &String))> = seed
        .lines
        .iter()
        .zip(candidate.lines.iter())
        .enumerate()
        .filter(|(i, _)| !unstable.contains(i))
        .collect();

    // Nothing left to compare is NOT equivalence, and it is not a divergence either: it is a
    // module whose every derived call renders unstably, so this fragment cannot ask it anything
    // honestly. Reported as its own verdict rather than folded into either, because the action
    // it calls for -- make emission deterministic -- is neither "fix the diff" nor "nothing to do".
    if compared.is_empty() {
        return ReceiptVerdict::NondeterministicRendering {
            unstable_calls: unstable.len(),
            functions: excluded,
        };
    }
    for (_, (a, b)) in &compared {
        if a != b {
            return ReceiptVerdict::Divergent {
                calls: compared.len(),
                first_difference: format!("seed: {a}  |  candidate: {b}"),
            };
        }
    }
    ReceiptVerdict::Equivalent {
        calls: compared.len(),
        nondeterministic_calls: unstable.len(),
        nondeterministic_functions: excluded,
    }
}

/// Which declared functions own the calls at `unstable`.
///
/// The driver prints one line per call in exactly the order `AdmittedPlan` enumerates them, so
/// the mapping is positional and derived from the same iteration that produced the transcript --
/// not a second traversal that could disagree with it. Names, because a COUNT of excluded calls
/// cannot be acted on and a name can: it is the function whose return value to make deterministic.
fn nondeterministic_call_functions(
    admitted: &AdmittedPlan<'_>,
    unstable: &std::collections::BTreeSet<usize>,
) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut index = 0usize;
    for (name, _domain, tuples) in &admitted.plan.derivable {
        for _ in tuples {
            if unstable.contains(&index) && !names.iter().any(|n| n == name) {
                names.push(name.clone());
            }
            index += 1;
        }
    }
    names
}

/// What the generated-artifact population says about one repo-relative path.
///
/// Three states because the honest answers are three. `NotGenerated` is a POSITIVE answer -- this
/// path is not in the generated-artifact population at all -- and is what routes a caller to the
/// mirror-emit population. Folding it into `Refused` would tell a caller that generation FAILED
/// for an ordinary mirror, which is false and differently actionable.
enum GeneratedArtifactPathBody {
    Produced(String),
    Refused(String),
    NotGenerated,
}

/// Ask the already-resolved generated-artifact authority for the body it generates at a path.
///
/// THIS IS NOT A SECOND PRODUCER. The `.dag` side is a projection over the same three authorities
/// `main_wet` uses -- the committed-artifact roster, `artifact_path`, and the single
/// `artifact_generate` dispatch -- asked by path instead of by artifact. Reaching past it to a
/// per-artifact emitter would have been the forked dispatch DESIGN §3 forbids.
///
/// COST SHAPE, and it is why this takes a CONTEXT rather than `source_roots`. The first draft
/// resolved `generated_artifact_emit`'s whole closure inside the per-module loop, making the unit
/// of computation the corpus while the unit of fact was one path -- DESIGN §6's cost-shape defect,
/// where the rule is that a proven one is fixed regardless of the realized n. The caller resolves
/// once; each path is then one interpreter call against that context.
/// The generated-artifact authority's evaluation context, resolved AT MOST ONCE per run and
/// shared by every caller that needs it.
///
/// One cell rather than one resolve per asking site: selection asks it for a module that yields
/// no call, and the differential loop asks it for every selected module. Two resolves of one
/// closure would be two producers of the same context and would pay the corpus-sized cost twice.
fn generated_artifact_ctx<'a>(
    source_roots: &[String],
    cell: &'a mut Option<crate::v1_interpreter::InterpContext>,
) -> Result<&'a crate::v1_interpreter::InterpContext, String> {
    if cell.is_none() {
        let entry = "dag/gunbc/generated_artifact_emit.dag";
        let (graph, indices) = crate::cli_run::resolve_entry_graph_shared(source_roots, entry)
            .map_err(|e| format!("resolve {entry}: {e}"))?;
        *cell = Some(crate::cli_run::make_eval_context(
            &graph,
            indices,
            // HERMETIC, not Wet. The projection is pure -- it folds a roster and returns a String
            // -- so a host effect reached during it would mean a generator is doing something
            // this gate must not perform on its behalf. Hermetic refuses there instead of
            // carrying it out.
            crate::v1_interpreter::ExecutionMode::Hermetic,
        ));
    }
    Ok(cell.as_ref().expect("the context was just installed"))
}

fn generated_artifact_body_for_path(
    ctx: &crate::v1_interpreter::InterpContext,
    repo_rel_path: &str,
) -> Result<GeneratedArtifactPathBody, String> {
    use crate::v1_interpreter::Value;
    let out = crate::v1_interpreter::run_in_context_with_args(
        ctx,
        "generated_artifact_body_for_path",
        &[(
            Some("path".to_string()),
            Value::Str(repo_rel_path.to_string().into()),
        )],
        false,
    )
    .map_err(|e| format!("generated_artifact_body_for_path({repo_rel_path}): {e:?}"))?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &out
    else {
        // No default arm: a shape this code does not understand is ignorance, and guessing
        // NotGenerated here would silently route a real generated artifact to the mirror emit
        // and refuse it there for the wrong reason.
        return Err(format!(
            "generated_artifact_body_for_path({repo_rel_path}) returned a non-variant value"
        ));
    };
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathNotGenerated") {
        return Ok(GeneratedArtifactPathBody::NotGenerated);
    }
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathBodyProduced") {
        return match ctx.field(fields, "content") {
            Some(Value::Str(c)) => Ok(GeneratedArtifactPathBody::Produced(c.to_string())),
            _ => Err(format!(
                "GeneratedArtifactPathBodyProduced for {repo_rel_path} carried no String content"
            )),
        };
    }
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathBodyRefused") {
        return match ctx.field(fields, "reason") {
            Some(Value::Str(r)) => Ok(GeneratedArtifactPathBody::Refused(r.to_string())),
            _ => Err(format!(
                "GeneratedArtifactPathBodyRefused for {repo_rel_path} carried no String reason"
            )),
        };
    }
    Err(format!(
        "generated_artifact_body_for_path({repo_rel_path}) returned an unknown variant"
    ))
}

/// Map an authority module path to the emitted mirror that declares it as its authority.
///
/// DERIVED, never authored: each generated file names its authority in its own header, written
/// by the emitter, so the mapping is a property of the artifact rather than of a roster someone
/// maintains, and it cannot be forged by editing a list.
///
/// TWO HEADER KEYS, BECAUSE THE CORPUS HAS TWO EMITTERS. The v1 compiler writes
/// `// Source module: <mod>`; `gunbc`'s own artifact emitters write `// Authority: <mod> ...`.
/// Measured over `src/v1/stage0/src`: 130 files declare themselves generated -- 126 by the first
/// key, 2 by the second (`bootstrap_stage0_crate_layout_generated.rs`,
/// `v1_interpreter_dispatch_generated.rs`), and `lib.rs`/`main.rs`, which are crate roots rather
/// than module mirrors. An earlier revision of this function read ONLY the first key, so those
/// two real mirrors of two real authorities were invisible to it: a change to either would have
/// been reported as "no emitted mirror declares it" -- FALSE, and false in the direction that
/// silently skips the check.
///
/// THAT WAS THE SAME TWO-ZEROS CONFLATION THIS MODE FIXES ONE LEVEL UP: "nothing mirrors this
/// authority" and "I could not find what mirrors it under the key I searched" are different
/// states with different remedies, and they printed the same exclusion line. So the index does
/// not merely learn the second key -- learning keys one incident at a time is how the blind spot
/// recurs. It ASSERTS ITS OWN KEY-SPACE COMPLETENESS: any file that declares itself generated
/// and yet carries neither key is a THIRD convention, which means this index has just gone blind
/// again, and the whole selection REFUSES rather than excluding authorities it cannot see.
///
/// The two crate roots are exempt by KIND, not by roster: `lib.rs` and `main.rs` are cargo's
/// crate-root names (an external, versioned authority), they aggregate the module SET rather
/// than mirroring any one authority, and no authority module path can ever select them.
struct MirrorIndex {
    by_module: std::collections::HashMap<String, String>,
}

/// The header keys, as a closed set read in one place. A key added here is a decision recorded
/// once; a key MISSING here is caught by the completeness refusal below rather than by silence.
const MIRROR_AUTHORITY_HEADER_KEYS: [&str; 2] = ["// Source module: ", "// Authority: "];

fn mirror_authority_of_header(content: &str) -> Option<String> {
    for line in content.lines().take(6) {
        let line = line.trim();
        for key in MIRROR_AUTHORITY_HEADER_KEYS {
            if let Some(rest) = line.strip_prefix(key) {
                // The `Authority:` form carries a symbol and a regen recipe after the module
                // path, separated by whitespace or `;`. Take the module path and nothing else --
                // comparing the whole remainder against a module path matches nothing, which is
                // the same silent miss one layer down.
                let module = rest
                    .trim()
                    .split([' ', ';', '\t'])
                    .next()
                    .unwrap_or_default()
                    .trim();
                if !module.is_empty() {
                    return Some(module.to_string());
                }
            }
        }
    }
    None
}

fn build_mirror_index(stage0_src: &std::path::Path) -> Result<MirrorIndex, String> {
    let entries =
        fs::read_dir(stage0_src).map_err(|e| format!("read_dir {}: {e}", stage0_src.display()))?;
    let mut by_module: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut unindexable: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let base = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        match mirror_authority_of_header(&content) {
            Some(module) => {
                by_module.insert(module, base);
            }
            None => {
                let declares_generated = content
                    .lines()
                    .next()
                    .map(|l| l.trim_start().starts_with("// Generated by "))
                    .unwrap_or(false);
                let is_crate_root = matches!(base.as_str(), "lib.rs" | "main.rs");
                if declares_generated && !is_crate_root {
                    unindexable.push(base);
                }
            }
        }
    }
    if !unindexable.is_empty() {
        unindexable.sort();
        return Err(format!(
            "the mirror index cannot see {} generated file(s) that declare themselves generated \
             but carry neither known authority header ({}): {}. A third header convention means \
             every \"no emitted mirror\" answer below is IGNORANCE rather than a fact, so the \
             selection refuses instead of excluding authorities it cannot see. Either the new \
             convention joins MIRROR_AUTHORITY_HEADER_KEYS, or the emitter writes an existing one",
            unindexable.len(),
            MIRROR_AUTHORITY_HEADER_KEYS.join("| "),
            unindexable.join(", ")
        ));
    }
    Ok(MirrorIndex { by_module })
}

/// THE POPULATION, AND WHAT DEFEATS IT -- a census, not a gate.
///
/// The differential answers ONE candidate. This answers the prior question: across every module
/// the seed actually carries, how much of each one's surface can be covered at all, and what
/// stands in the way of the rest. It runs no build and installs no candidate; it is the ranking
/// input for how far to extend a mechanism that is now proven on one module, and it deliberately
/// exits SUCCESS on any population -- a census that refused would be a gate, and nothing here
/// establishes what the right coverage is.
///
/// The population is DERIVED, never authored: every emitted mirror names its authority in its
/// own header, so the roster is a property of the artifacts. A module whose authority source is
/// missing is REPORTED, not skipped -- a census that silently drops what it cannot read reports a
/// smaller corpus as a cleaner one.
fn behavioral_receipt_census(source_roots: &[String]) -> Result<bool, String> {
    let workspace = crate::cli_run::workspace_root();
    let stage0_src = workspace.join("src/v1/stage0/src");
    let modules = collect_dag_module_sources(source_roots)?;
    let declared_anywhere = declared_type_names(&modules)?;

    let mut roster: Vec<(String, String)> = Vec::new();
    let entries =
        fs::read_dir(&stage0_src).map_err(|e| format!("read_dir {}: {e}", stage0_src.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| format!("dir entry: {e}"))?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        // Same reader as the plan mode's index, so the census and the gate cannot disagree
        // about which files mirror an authority -- including the two that name it with the
        // second header convention.
        if let Some(declared) = mirror_authority_of_header(&content) {
            roster.push((
                declared,
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string(),
            ));
        }
    }
    roster.sort();

    let mut fns_total = 0usize;
    let mut fns_derivable = 0usize;
    let mut fns_refused = 0usize;
    let mut calls_total = 0usize;
    let mut modules_planned = 0usize;
    // NOT "no authority": the census cannot distinguish an authority that does not exist from one
    // that exists outside the roots it was given, so it reports what it actually knows and names
    // the roots. The first revision of this line said NO AUTHORITY SOURCE, and 55 of 127 modules
    // hit it -- every one of them a v1.compiler module whose .dag lives under src/v1, which is
    // simply not a scanned root. A refusal that names the wrong cause ranks the wrong work.
    let mut out_of_scope: Vec<String> = Vec::new();
    // Keyed on the TYPE that defeated derivation, because that is the unit of work: grounding one
    // type unlocks every function whose only obstacle was that type. Counting refusals instead
    // would rank the same fix once per site.
    let mut by_blocker: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut types_declared_total = 0usize;
    let mut types_read_total = 0usize;
    let mut type_reader_gaps: Vec<(String, usize, usize, Vec<String>)> = Vec::new();

    for (module_path, mirror) in &roster {
        let Some(source) = modules.get(module_path) else {
            out_of_scope.push(module_path.clone());
            continue;
        };
        let alias = format!("crate::{}", mirror.trim_end_matches(".rs"));
        let node = parse_dag_module_node(&format!("{module_path}.dag"), source)?;
        let types = visible_type_decls(module_path, source, &modules)?;
        let plan = plan_module_corpus(
            module_path,
            source,
            &node,
            &types,
            &declared_anywhere,
            &alias,
        );
        modules_planned += 1;
        fns_total += plan.parsed_signatures;
        fns_derivable += plan.derivable.len();
        fns_refused += plan.refused.len();
        let calls: usize = plan.derivable.iter().map(|(_, _, t)| t.len()).sum();
        calls_total += calls;
        for (_f, why) in &plan.refused {
            *by_blocker.entry(why.subject()).or_insert(0) += 1;
        }
        // THE SAME CROSS-CHECK THE FUNCTION READER CARRIES, for types. Two readers of one fact --
        // an authored `type ` line count and the count the parser actually produced -- so a
        // disagreement means the type reader missed a form. That is not hypothetical: Node ranked
        // 798 as "undeclared anywhere in corpus" while src/v1/00_core.dag declares it plainly, and
        // nothing in the output said the reader had skipped it.
        // The authored names, not just a count: the count says a form was missed, the NAMES say
        // which declarations, and only the second is actionable without guessing.
        let authored: Vec<String> = source
            .lines()
            .filter_map(|l| l.trim_start().strip_prefix("type "))
            .filter_map(|rest| {
                // CUT AT `<` TOO. Without it the authored name of a generic declaration came out
                // as `Magma<T>` or, worse, `Map<key,` -- while the reader registers the bare name
                // -- so the comparison reported 11 modules as having gaps whose type_lines and
                // types_read were EQUAL. The falsifier was manufacturing its own false positives,
                // which is the one failure mode a cross-check cannot be allowed to have: it spends
                // exactly the attention it exists to direct.
                rest.split(|c: char| c.is_whitespace() || c == '{' || c == '=' || c == '<')
                    .find(|t| !t.is_empty())
                    .map(str::to_string)
            })
            .collect();
        let read = type_decls_from_module(&node);
        let type_lines = authored.len();
        let types_read = read.len();
        types_declared_total += type_lines;
        types_read_total += types_read;
        let missed: Vec<String> = authored
            .iter()
            .filter(|n| !read.contains_key(*n))
            .cloned()
            .collect();
        if !missed.is_empty() {
            type_reader_gaps.push((module_path.clone(), type_lines, types_read, missed));
        }
        eprintln!(
            "receipt-census: {module_path} parsed={} derivable={} calls={} refused={} type_lines={type_lines} types_read={types_read}",
            plan.parsed_signatures,
            plan.derivable.len(),
            calls,
            plan.refused.len()
        );
    }

    let mut ranked: Vec<(String, usize)> = by_blocker.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    eprintln!(
        "receipt-census: modules_with_mirror={} planned={} authority_outside_scanned_roots={}",
        roster.len(),
        modules_planned,
        out_of_scope.len()
    );
    // A COUNT IS NOT A SAFE REPORT HERE. Read as a small number, out-of-scope looks like a rounding
    // error; but the same shape at a wrong root set is a hole centred on whatever nobody scanned,
    // and this arm has already been that hole once -- it swallowed 55 of 127 as "no authority"
    // because src/v1 was not a root. So the fraction is stated against the population, and any
    // out-of-scope module at all makes the run say what its coverage actually is rather than
    // leading with the modules it managed to plan.
    if !out_of_scope.is_empty() {
        eprintln!(
            "receipt-census: COVERAGE {}/{} modules planned; {} of the corpus was NOT measured \
             because no scanned root holds its authority. Roots given: {}. A module below is not \
             evidence that its authority is missing -- it is evidence that this run could not see \
             it, and if this fraction is large the root set is the finding, not the corpus",
            modules_planned,
            roster.len(),
            out_of_scope.len(),
            source_roots.join(", ")
        );
    }
    for m in &out_of_scope {
        eprintln!("receipt-census: OUT OF SCOPE {m} — counted, not skipped");
    }
    eprintln!(
        "receipt-census: functions parsed={fns_total} derivable={fns_derivable} refused={fns_refused} calls={calls_total}"
    );
    eprintln!(
        "receipt-census: TYPE READER type_lines={types_declared_total} types_read={types_read_total} modules_with_a_gap={}",
        type_reader_gaps.len()
    );
    if !type_reader_gaps.is_empty() {
        eprintln!(
            "receipt-census: a gap means the reader did not produce a declaration the source \
             authors, so every refusal naming those types is measuring THIS READER, not the corpus"
        );
        let mut worst = type_reader_gaps.clone();
        worst.sort_by(|a, b| b.3.len().cmp(&a.3.len()));
        for (m, lines, read, missed) in worst.iter() {
            eprintln!(
                "receipt-census:   GAP {m} type_lines={lines} types_read={read} missed={}",
                missed.join(", ")
            );
        }
        // WHAT SHAPE the missed declarations actually have, for one module, printed rather than
        // assumed. Three times now a node-shape assumption has been wrong and each time the cost
        // was a whole measurement built on it; the shape is cheap to report and there is no reason
        // for the next reader of this output to have to re-derive it.
        // WHAT DISTINGUISHES A TYPE DECLARATION FROM A FUNCTION OR A DATA ROW, printed for one
        // module rather than assumed. The reader must filter on a property of the node; every
        // previous attempt to name one from memory has been wrong.
        if let Some(src) = modules.get("std.pareto") {
            if let Ok(node) = parse_dag_module_node("std.pareto.dag", src) {
                let authored: std::collections::HashSet<String> = src
                    .lines()
                    .filter_map(|l| l.trim_start().strip_prefix("type "))
                    .filter_map(|r| {
                        r.split(|c: char| c.is_whitespace() || c == '{' || c == '=' || c == '<')
                            .find(|t| !t.is_empty())
                            .map(str::to_string)
                    })
                    .collect();
                let mut shapes: std::collections::BTreeMap<String, (usize, Vec<String>)> =
                    std::collections::BTreeMap::new();
                for c in node.children.iter() {
                    let key = format!(
                        "is_type={} conn={:?} body={} params={} inferred={} children={}",
                        authored.contains(&c.name),
                        c.connective,
                        c.body.is_some(),
                        c.params.len(),
                        c.inferred.is_some(),
                        c.children.len()
                    );
                    let e = shapes.entry(key).or_insert((0, Vec::new()));
                    e.0 += 1;
                    if e.1.len() < 3 {
                        e.1.push(c.name.clone());
                    }
                }
                for (k, (n, ex)) in &shapes {
                    eprintln!(
                        "receipt-census:   CHILDSHAPE {n:4}  {k}  e.g. {}",
                        ex.join(", ")
                    );
                }
            }
        }
        if let Some((m, _, _, missed)) = worst.iter().find(|(m, _, _, _)| m == "std.pareto") {
            if let Some(src) = modules.get(m) {
                if let Ok(node) = parse_dag_module_node(&format!("{m}.dag"), src) {
                    for name in missed {
                        match node.children.iter().find(|c| &c.name == name) {
                            Some(c) => {
                                let f = c.children.iter().next();
                                eprintln!(
                                    "receipt-census:   SHAPE {m}::{name} connective={:?} \
                                     children={} | field name={:?} children={} conn={:?} \
                                     type_annotation={:?} inferred={}",
                                    c.connective,
                                    c.children.len(),
                                    f.map(|f| f.name.clone()),
                                    f.map(|f| f.children.len()).unwrap_or(0),
                                    f.map(|f| f.connective.clone()),
                                    f.and_then(|f| f.type_annotation.as_ref())
                                        .map(|t| t.name.clone()),
                                    f.map(|f| f.inferred.is_some()).unwrap_or(false)
                                )
                            }
                            None => eprintln!(
                                "receipt-census:   SHAPE {m}::{name} — NO module child carries \
                                 this name; the declaration is not where the reader looks"
                            ),
                        }
                    }
                }
            }
        }
    }
    eprintln!("receipt-census: refusals ranked by the type responsible");
    for (why, n) in ranked.iter().take(40) {
        eprintln!("receipt-census:   {n:5}  {why}");
    }
    if ranked.len() > 40 {
        let tail: usize = ranked.iter().skip(40).map(|(_, n)| n).sum();
        eprintln!(
            "receipt-census:   {tail:5}  [{} further distinct causes, not shown]",
            ranked.len() - 40
        );
    }
    Ok(true)
}

fn run_behavioral_receipt_census(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    match behavioral_receipt_census(source_roots) {
        Ok(_) => Ok(ExitCode::SUCCESS),
        Err(e) => {
            eprintln!("receipt-census: REFUSED — {e}");
            Err(ExitCode::from(1))
        }
    }
}

pub fn run_census(source_roots: &[String]) -> BehavioralHostOutcome {
    match behavioral_receipt_census(source_roots) {
        Ok(true) => BehavioralHostOutcome {
            termination: BehavioralHostTermination::ObservationHeld,
            message: "behavioral-receipt-census: completed; detailed census emitted above".into(),
        },
        Ok(false) => BehavioralHostOutcome {
            termination: BehavioralHostTermination::ObservationDidNotHold,
            message: "behavioral-receipt-census: completed with an unsatisfied observation".into(),
        },
        Err(cause) => BehavioralHostOutcome {
            termination: BehavioralHostTermination::Refused,
            message: format!("behavioral-receipt-census: refused: {cause}"),
        },
    }
}

fn behavioral_receipt_selftest(source_roots: &[String]) -> Result<bool, String> {
    let workspace = crate::cli_run::workspace_root();
    // NOT under src/v1: regen seeds every .dag there into the stage0 compile closure, and this
    // authority must never be emitted. Measured, not assumed -- placing it there made the emit
    // produce receipt_fixture.rs with no committed mirror, and required-regen refused the whole
    // surface as a population mismatch.
    let fixture = workspace.join("fixtures/receipt_fixture");
    let module_path = "receipt.fixture";
    let alias = "v1_receipt_fixture";

    let authority = fixture.join("authority.dag");
    let source =
        fs::read_to_string(&authority).map_err(|e| format!("read {}: {e}", authority.display()))?;
    let modules = collect_dag_module_sources(source_roots)?;
    let node = parse_dag_module_node(&format!("{module_path}.dag"), &source)?;
    let types = visible_type_decls(module_path, &source, &modules)?;
    let declared_anywhere = declared_type_names(&modules)?;
    let plan = plan_module_corpus(
        module_path,
        &source,
        &node,
        &types,
        &declared_anywhere,
        alias,
    );

    eprintln!(
        "receipt-selftest: fixture parsed={} derivable={} refused={}",
        plan.parsed_signatures,
        plan.derivable.len(),
        plan.refused.len()
    );
    for (f, d, tuples) in &plan.derivable {
        eprintln!(
            "receipt-selftest:   derivable {f} {} calls={}",
            d.report(),
            tuples.len()
        );
    }
    for (f, why) in &plan.refused {
        eprintln!("receipt-selftest:   REFUSED {f} — {}", why.describe());
    }

    // THE PRECONDITION THE ARMS DEPEND ON, checked before the arms rather than assumed by them.
    //
    // Arm 2 changes `band_of` at exactly one input in all of i64: level = 100. If the boundary
    // enumeration ever regresses to sampling, or to a window that excludes 100, arm 2 goes GREEN
    // and this whole control dies without a sound. So the tuple set is required to CONTAIN that
    // input, and the requirement is stated over the enumerated corpus -- the thing that will
    // actually run -- not over the domain description of it.
    let band_of = plan
        .derivable
        .iter()
        .find(|(f, _, _)| f == "band_of")
        .ok_or_else(|| {
            format!(
                "the fixture's band_of did not derive, so arm 2 could not discriminate even if it \
                 ran. Refusing rather than reporting a control that cannot fail. Refusals: {}",
                plan.refused
                    .iter()
                    .map(|(f, w)| format!("{f}: {}", w.describe()))
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        })?;
    if !matches!(
        band_of.1,
        EnumeratedDomain::ExhaustiveOverDerivedPartition { .. }
    ) {
        return Err(format!(
            "band_of derived as {}, not over a derived partition. The Int partition is the thing \
             arm 2 exercises; covering it some other way would leave that arm untested",
            band_of.1.report()
        ));
    }
    if !band_of.2.iter().any(|t| t == &vec!["100i64".to_string()]) {
        return Err(format!(
            "the enumerated corpus for band_of does not contain the boundary input 100i64, so arm \
             2's single behavioural difference is outside what the receipt would run. Enumerated: \
             {:?}",
            band_of.2
        ));
    }

    let seed_path = fixture.join("src/lib.rs");
    let mut ok = true;
    let krate = ReceiptCrate::receipt_fixture();
    // The fixture must ADMIT, and it says so here rather than deep inside an arm: a fixture that
    // stopped yielding calls would otherwise turn both arms into an exclusion and this control
    // into a control that cannot fail.
    let admitted = AdmittedPlan::of(&plan).map_err(|c| {
        format!(
            "the fixture yields no call at all, so neither arm could discriminate. Uncovered: {}",
            c.uncovered
                .iter()
                .map(|(f, w)| format!("{f}: {}", w.describe()))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;

    for (arm, candidate_file, expect_equivalent) in [
        ("preserving", "behaviour_preserving.rs", true),
        ("changing", "behaviour_changing.rs", false),
    ] {
        let cand_path = fixture.join("candidates").join(candidate_file);
        let candidate = fs::read_to_string(&cand_path)
            .map_err(|e| format!("read {}: {e}", cand_path.display()))?;
        // An arm whose candidate bytes equal the seed's proves nothing in EITHER direction: the
        // preserving arm would report equivalence for the trivial reason, and the changing arm
        // would report equivalence and be read as a regression. Checked, not trusted.
        let seed = fs::read_to_string(&seed_path)
            .map_err(|e| format!("read {}: {e}", seed_path.display()))?;
        if seed == candidate {
            return Err(format!(
                "arm {arm}: {candidate_file} is byte-identical to the fixture seed, so this arm \
                 compares a file with itself"
            ));
        }

        let verdict =
            behavioral_differential(&workspace, &krate, &seed_path, &candidate, &admitted, alias);
        match (&verdict, expect_equivalent) {
            (
                ReceiptVerdict::Equivalent {
                    calls,
                    nondeterministic_calls,
                    ..
                },
                true,
            ) => {
                // FALSE-POSITIVE CONTROL for the two-run instability probe, and it is the reason
                // this arm now reads a second field. The fixture's corpus is deterministic, so
                // the probe must find NOTHING unstable in it. Without this, a probe that marked
                // every line unstable would still print EQUIVALENT here -- over an empty compared
                // set it would not even reach this arm, but over a partially-marked one it would,
                // and the arm would pass while the gate had quietly stopped comparing anything.
                if *nondeterministic_calls != 0 {
                    eprintln!(
                        "receipt-selftest: arm {arm} EQUIVALENT but the instability probe marked {nondeterministic_calls} call(s) unstable in a DETERMINISTIC fixture — the probe is producing false positives, so its exclusions cannot be trusted"
                    );
                    ok = false;
                } else {
                    eprintln!("receipt-selftest: arm {arm} EQUIVALENT over {calls} derived calls — as required");
                }
            }
            (
                ReceiptVerdict::Divergent {
                    calls,
                    first_difference,
                },
                false,
            ) => {
                // Not merely THAT it diverged: divergence at the wrong call would mean the arm is
                // catching something other than the difference it was authored to catch, and a
                // control that passes for the wrong reason is not a control.
                if !(first_difference.contains("band_of") && first_difference.contains("100i64")) {
                    eprintln!(
                        "receipt-selftest: arm {arm} DIVERGENT over {calls} calls but at the WRONG \
                         call — expected band_of(100i64): {first_difference}"
                    );
                    ok = false;
                } else {
                    eprintln!(
                        "receipt-selftest: arm {arm} DIVERGENT over {calls} derived calls at the \
                         authored difference — {first_difference}"
                    );
                }
            }
            (v, _) => {
                eprintln!(
                    "receipt-selftest: arm {arm} expected {} but got {v:?}",
                    if expect_equivalent {
                        "EQUIVALENT"
                    } else {
                        "DIVERGENT"
                    }
                );
                ok = false;
            }
        }
    }
    Ok(ok)
}

fn run_behavioral_receipt_selftest(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    match behavioral_receipt_selftest(source_roots) {
        Ok(true) => Ok(ExitCode::SUCCESS),
        Ok(false) => {
            eprintln!(
                "receipt-selftest: REFUSED — the behavioral receipt's own arms no longer \
                 discriminate. Until this is green, no verdict the mode reports is evidence"
            );
            Err(ExitCode::from(1))
        }
        Err(e) => {
            eprintln!("receipt-selftest: REFUSED — {e}");
            Err(ExitCode::from(1))
        }
    }
}

pub fn run_selftest(source_roots: &[String]) -> BehavioralHostOutcome {
    match behavioral_receipt_selftest(source_roots) {
        Ok(true) => BehavioralHostOutcome {
            termination: BehavioralHostTermination::ObservationHeld,
            message: "behavioral-receipt-selftest: arms discriminate".into(),
        },
        Ok(false) => BehavioralHostOutcome {
            termination: BehavioralHostTermination::ObservationDidNotHold,
            message: "behavioral-receipt-selftest: arms do not discriminate".into(),
        },
        Err(cause) => BehavioralHostOutcome {
            termination: BehavioralHostTermination::Refused,
            message: format!("behavioral-receipt-selftest: refused: {cause}"),
        },
    }
}

/// STANDALONE INVOCATION, where an absent subject is a MISUSE rather than a state to report.
///
/// Naming this mode on the command line asserts there is a pull request to check. If the merge
/// base is the head there is not, so the invocation is answered rather than silently succeeding.
/// This mode has no other caller: the 2026-08-21 operator ruling cut the receipt out of
/// `--required-ci`, so nothing invokes it automatically and its only route is someone typing it.
///
/// A DIFF-SUBJECT GATE CANNOT BE EXERCISED BY THE BRANCH IT PROTECTS, and this paragraph is
/// carried here because the property is a fact about THIS gate's subject rather than about the
/// phase that used to run it. It was learned expensively and it survives its enrolment.
///
/// The subject is a diff against the merge base. On main the merge base IS the head, so there is
/// no pull request to check and never was -- which means NO main run, green or otherwise, is
/// evidence about this gate: comparing a PR's red against main's green compares a run against a
/// SKIP, not against a pass. Coverage is entirely PR-side by construction. The consequence that
/// makes it worth writing down: a defect that reds every PR touching one class of authority can
/// sit indefinitely while main stays green, because the only runs that could have seen it are
/// the ones a person reads as "my branch is broken". That is not hypothetical -- it is how the
/// wet-actuator selection defect (gunbc#8704, excluded at selection since) survived.
///
/// SO IT BINDS ANY FUTURE PROPOSAL, not just this one. If a diff-subject gate is ever enrolled
/// in CI again -- this receipt or another -- read this first: it needs a real subject on main
/// (the PUSH RANGE is one, and is a DIFFERENT subject rather than a stand-in for a PR diff), or
/// it is accepting PR-only coverage knowingly. What must NOT move is the absent-subject arm:
/// making it return an answer gives it a deficit frequency of zero by construction, which is the
/// absorbing fallback wearing the fix's clothes (DESIGN section 5).
fn run_behavioral_receipt_plan(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    match behavioral_receipt_plan(source_roots) {
        Ok(ReceiptPlanOutcome::Ran { agreed: true }) => Ok(ExitCode::SUCCESS),
        Ok(ReceiptPlanOutcome::Ran { agreed: false }) => Err(ExitCode::from(1)),
        Ok(ReceiptPlanOutcome::NoSubject { head }) => {
            eprintln!(
                "behavioral-receipt: NO SUBJECT — the merge base resolves to HEAD ({head}), so \
                 the diff compares this commit against itself and cannot observe what changed. \
                 This is not an empty selection and is not reported as a pass: `nothing changed` \
                 and `I could not see what changed` are different states. This mode's subject is \
                 a pull request against main; invoke it there"
            );
            Err(ExitCode::from(1))
        }
        Err(refusal) => {
            eprintln!("behavioral-receipt: refused: {refusal}");
            Err(ExitCode::from(1))
        }
    }
}

/// What the per-PR receipt run found, as a state rather than a bool.
///
/// `NoSubject` exists because "this run has nothing to check" and "this run checked and agreed"
/// are the two zeros this mode was corrected for once already, one level down. Collapsing them
/// into `Ok(true)` is precisely how the vacuous pass on `push: main` was written.
#[derive(Debug, Clone, PartialEq)]
enum ReceiptPlanOutcome {
    /// The merge base resolves to HEAD, so no diff exists to read. Not a pass, not a failure --
    /// an absent subject.
    NoSubject { head: String },
    /// A selection was computed and every selected module reached a verdict.
    Ran { agreed: bool },
}

pub fn run_plan(source_roots: &[String]) -> BehavioralHostOutcome {
    match behavioral_receipt_plan(source_roots) {
        Ok(ReceiptPlanOutcome::Ran { agreed: true }) => BehavioralHostOutcome {
            termination: BehavioralHostTermination::ObservationHeld,
            message: "behavioral-receipt-plan: agreed".into(),
        },
        Ok(ReceiptPlanOutcome::Ran { agreed: false }) => BehavioralHostOutcome {
            termination: BehavioralHostTermination::ObservationDidNotHold,
            message: "behavioral-receipt-plan: diverged".into(),
        },
        Ok(ReceiptPlanOutcome::NoSubject { head }) => BehavioralHostOutcome {
            termination: BehavioralHostTermination::SubjectUnreached,
            message: format!("behavioral-receipt-plan: no subject at head {head}"),
        },
        Err(cause) => BehavioralHostOutcome {
            termination: BehavioralHostTermination::Refused,
            message: format!("behavioral-receipt-plan: refused: {cause}"),
        },
    }
}

fn behavioral_receipt_plan(source_roots: &[String]) -> Result<ReceiptPlanOutcome, String> {
    let workspace = crate::cli_run::workspace_root();
    let stage0_src = workspace.join("src/v1/stage0/src");

    // BASELINE FIRST, asserted and printed, exactly as the mirror-drift gate does and for the
    // same reason: a selection computed against an unresolvable baseline is ignorance, and the
    // tempting fallback -- treat everything as changed -- is the absorbing arm that turns a
    // per-change gate into a per-corpus one.
    let head = git_stdout(&workspace, &["rev-parse", "HEAD"])?;
    let base = git_stdout(&workspace, &["merge-base", "origin/main", "HEAD"]).map_err(|e| {
        format!(
            "cannot resolve the merge base against origin/main ({e}). The selection is NOT \
             widened to the whole population in this case: `I could not determine what changed` \
             and `everything changed` are different states, and two compiler builds per module \
             across the corpus is a budget breach denominated in the repository rather than in \
             the change. Fetch the base first: \
             `git fetch --depth=200 origin main:refs/remotes/origin/main`"
        )
    })?;
    eprintln!("behavioral-receipt: merge_base={base} head={head}");

    // A BASELINE THAT IS THE HEAD IS NOT AN EMPTY SELECTION -- IT IS NO OBSERVATION AT ALL.
    //
    // On a push to main, `git merge-base origin/main HEAD` resolves to HEAD itself, so the diff
    // compares the commit against itself and yields zero changed authorities. The empty-selection
    // arm below would then report a real pass over a corpus that was never looked at. That is the
    // empty-observation narrow DESIGN names by its live specimen -- a push whose baseline ref IS
    // the pushed ref -- and it is the mirror of the absorbing fallback: a widen is merely
    // expensive, a narrow is silently uncovered.
    //
    // `nothing changed` and `I could not see what changed` are different states with different
    // remedies, so they get different answers. This one refuses, and it names the invocation that
    // makes sense instead of guessing at a substitute baseline: the subject of this gate is a
    // pull request, and a push to a branch that IS the baseline has no such subject.
    if base == head {
        return Ok(ReceiptPlanOutcome::NoSubject { head });
    }

    let changed = git_stdout(
        &workspace,
        &["diff", "--name-only", &base, &head, "--", "*.dag"],
    )?;
    let changed: Vec<String> = changed
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    // Loaded once for the whole run: the type a module compiles against may be declared in any
    // module it transitively imports.
    let modules = collect_dag_module_sources(source_roots)?;
    let declared_anywhere = declared_type_names(&modules)?;

    // Built ONCE, before any authority is classified, and it refuses if it cannot see the whole
    // generated population -- so an exclusion below is a fact about the corpus rather than a fact
    // about what this reader happened to recognise.
    let mirror_index = build_mirror_index(&stage0_src)?;

    let mut exclusions: Vec<ReceiptExclusion> = Vec::new();
    let mut plans: Vec<(String, ModuleCorpusPlan, String)> = Vec::new();
    // BUILT ON FIRST DEMAND, not before the loop. Only a module that yields no call needs to ask
    // the generated-artifact population during selection, which is a small population and often
    // an empty one; resolving its closure unconditionally would put a corpus-sized resolve on
    // every run of a per-change gate -- the cost shape DESIGN §6 names, where the unit of
    // computation is the corpus and the unit of fact is one path.
    let mut generated_ctx_cell: Option<crate::v1_interpreter::InterpContext> = None;
    // BOTH GRAINS, ACCUMULATED ACROSS EVERY CHANGED AUTHORITY -- including the excluded ones,
    // which is the point: a function that yields no call is uncovered whether or not its module
    // had a sibling that saved it from exclusion.
    let mut declared_functions = 0usize;
    let mut covered_functions = 0usize;

    for rel in &changed {
        let abs = workspace.join(rel);
        let Ok(source) = fs::read_to_string(&abs) else {
            continue;
        };
        let Some(module_path) = crate::cli_run::extract_module_path_public(&source) else {
            continue;
        };
        match mirror_index.by_module.get(&module_path).cloned() {
            None => exclusions.push(ReceiptExclusion::NoEmittedMirror { module_path }),
            Some(mirror) => {
                // The Rust module path is the mirror's basename without its extension — derived
                // from the artifact, like the mapping that found it, never spelled out here.
                // The full crate path to the emitted mirror, derived from the artifact's own
                // basename. Both the enumerated constructor values and the generated calls are
                // written against THIS string, so there is one spelling of the module under test.
                let alias = format!("crate::{}", mirror.trim_end_matches(".rs"));
                let node = parse_dag_module_node(&format!("{module_path}.dag"), &source)?;
                let types = visible_type_decls(&module_path, &source, &modules)?;
                let plan = plan_module_corpus(
                    &module_path,
                    &source,
                    &node,
                    &types,
                    &declared_anywhere,
                    &alias,
                );
                // ADMISSION IS DECIDED HERE, AT FUNCTION GRAIN, and it is decided from a fact the
                // source already carries -- no build, no emit, no differential is spent on a
                // module that cannot produce a call. The decision itself is a PURE function, so
                // both of its exclusion arms and its refusal can be executed against hand-built
                // states rather than only against whatever the live corpus happens to hold.
                let coverage = function_grain_coverage(&plan);
                declared_functions += coverage.covered.len() + coverage.uncovered.len();
                covered_functions += coverage.covered.len();
                match plan_grain_selection(&plan, coverage)? {
                    None => plans.push((mirror, plan, alias)),
                    Some(exclusion) => {
                        // A ZERO-CALL MODULE IS NOT AUTOMATICALLY A SUBJECTLESS ONE, and getting
                        // this wrong is how an exclusion becomes the thing it was meant to
                        // prevent. #8753 established the fact by measurement: for a mirror in the
                        // GENERATED-ARTIFACT population the subject is the artifact's BYTES, not
                        // its behaviour, and that population's only drift observer anywhere is the
                        // identity check inside the differential loop. Excluding here on `no
                        // function yields a call` would delete that observer for exactly the
                        // artifacts that have no functions to call -- silently, and while printing
                        // a line that says the module has nothing to compare.
                        //
                        // So the population is ASKED, and it is asked with the same projection
                        // the loop asks (`generated_artifact_body_for_path`), not with a second
                        // roster. `NotGenerated` is a positive answer -- this path is a module
                        // mirror -- and only then is the exclusion a fact.
                        let repo_rel = format!("src/v1/stage0/src/{mirror}");
                        let ctx = generated_artifact_ctx(source_roots, &mut generated_ctx_cell)?;
                        let body = generated_artifact_body_for_path(ctx, &repo_rel)?;
                        match exclusion_survives_generated_artifact_population(exclusion, &body) {
                            Some(exclusion) => exclusions.push(exclusion),
                            None => plans.push((mirror, plan, alias)),
                        }
                    }
                }
            }
        }
    }

    // BOTH GRAINS, SIDE BY SIDE, ON EVERY RUN -- even when they agree (operator directive,
    // 2026-08-21). A module count alone reads as coverage ("3 modules, one pass"); a function
    // count alone reads as what happened but hides how much of the corpus was in scope at all.
    // The gap between the two sentences is the thing worth seeing, and a reader given one number
    // infers the other one wrongly.
    //
    // EXCLUSIONS ARE COUNTED PER ARM, not as one total. A total says how many changed authorities
    // reached no verdict; only the split says WHY, and the arms rank differently: a
    // non-derivability row is work someone can do, an outside-the-population row is a fact no
    // work removes. Collapsing them is how a population that cannot shrink gets read as debt that
    // simply has not been paid.
    let mut excluded_no_mirror = 0usize;
    let mut excluded_no_function_declared = 0usize;
    let mut excluded_no_corpus = 0usize;
    for e in &exclusions {
        match e {
            ReceiptExclusion::NoEmittedMirror { .. } => excluded_no_mirror += 1,
            ReceiptExclusion::NoFunctionDeclared { .. } => excluded_no_function_declared += 1,
            ReceiptExclusion::NoFunctionHasACorpus { .. } => excluded_no_corpus += 1,
        }
    }
    eprintln!(
        "behavioral-receipt: GRAIN module: changed_authorities={} selected={} excluded={} \
         (no-emitted-mirror={excluded_no_mirror} \
         no-function-declared={excluded_no_function_declared} \
         no-function-has-a-corpus={excluded_no_corpus}) | \
         function: declared={declared_functions} covered={covered_functions} uncovered={}",
        changed.len(),
        plans.len(),
        exclusions.len(),
        declared_functions - covered_functions
    );
    for e in &exclusions {
        match e {
            ReceiptExclusion::NoEmittedMirror { module_path } => eprintln!(
                "behavioral-receipt: excluded {module_path} — no emitted mirror in the \
                 generated population names it as its authority, under either header \
                 convention (the index refuses outright if any generated file is \
                 unindexable, so this is a fact about the corpus and not a lookup miss)"
            ),
            ReceiptExclusion::NoFunctionDeclared { module_path } => eprintln!(
                "behavioral-receipt: excluded {module_path} — the authority declares no \
                 functions, so it carries no behaviour that could diverge and there is nothing \
                 for a differential to compare. NOT a derivation deficit: it contributes no \
                 uncovered function to the FUNCTION-GRAIN line, because there is no function to \
                 cover"
            ),
            ReceiptExclusion::NoFunctionHasACorpus {
                module_path,
                uncovered,
            } => {
                eprintln!(
                    "behavioral-receipt: excluded {module_path} — none of its {} declared \
                     functions yields a call, so there is no corpus to compare. NOT a failure of \
                     this diff: the same non-derivability in a module with one derivable sibling \
                     runs and passes, so refusing here would be a verdict decided by where a \
                     file-level count landed. Every uncovered function is named below and counted \
                     in the FUNCTION-GRAIN line",
                    uncovered.len()
                );
                for (f, why) in uncovered {
                    eprintln!(
                        "behavioral-receipt:   uncovered {module_path}::{f} — {}",
                        why.describe()
                    );
                }
            }
        }
    }
    for (_mirror, p, _alias) in &plans {
        // Both counts are coverage claims, so they are reported as the two ways coverage was
        // ESTABLISHED -- closed type versus derived partition -- and not as strong-versus-weak.
        // There is no third number here any more; the bounded column it replaced counted
        // functions that had been sampled, not covered.
        let closed = p
            .derivable
            .iter()
            .filter(|(_, d, _)| matches!(d, EnumeratedDomain::Exhaustive { .. }))
            .count();
        eprintln!(
            "behavioral-receipt: {} fn_lines={} parsed={} derivable={} (closed-type={} derived-partition={}) refused={}",
            p.module_path,
            p.declared_fn_lines,
            p.parsed_signatures,
            p.derivable.len(),
            closed,
            p.derivable.len() - closed,
            p.refused.len()
        );
        for (f, d, _tuples) in &p.derivable {
            eprintln!(
                "behavioral-receipt:   derivable {}::{f} {}",
                p.module_path,
                d.report()
            );
        }
        for (f, why) in &p.refused {
            eprintln!(
                "behavioral-receipt:   REFUSED {}::{f} — corpus not derivable: {}",
                p.module_path,
                why.describe()
            );
        }
    }

    // THE DIFFERENTIAL. Everything above decides WHAT to run; this runs it.
    //
    // The emit happens once for the whole selection rather than once per module: it is the
    // expensive step, and asking for it per candidate would make a two-module change cost twice
    // what a one-module change costs for no additional information.
    if plans.is_empty() {
        eprintln!(
            "behavioral-receipt: no changed authority module reached the differential — every one \
             was excluded above, each under one of the three typed arms counted on the \
             module-grain line. Nothing to compare, so this run costs nothing. That \
             is a real pass over an EMPTY selection, stated rather than printed as a bare PASS, \
             and the FUNCTION-GRAIN counts above say how much surface that silence covers"
        );
        return Ok(ReceiptPlanOutcome::Ran { agreed: true });
    }

    // A DECLARED CAP, REFUSED ABOVE RATHER THAN SAMPLED (operator ruling, 2026-08-20).
    //
    // The differential costs one crate build per selected module plus one for the seed. The
    // tempting arm when a PR touches many authorities is to check the first few and report a pass
    // -- the absorbing fallback exactly: the deficit's frequency goes to zero by construction and
    // nobody learns the gate stopped covering things. So an over-cap selection REFUSES, typed and
    // counted, and an over-cap PR is visible rather than quietly under-checked.
    //
    // WHAT KIND OF NUMBER THIS IS, because a bare literal in a merge-blocking check is exactly
    // what DESIGN §5 tells reviewers to distrust (review 54096 asked, correctly). It is a POLICY
    // BUDGET -- one of the four sanctioned grounds -- and the resource it caps is CI wall clock:
    // each selected module costs a full v1-compiler release build, so four modules is roughly
    // forty minutes on a job that already runs thirty. It is NOT a measurement copied from the
    // tree, and automating its update would not collapse this check to `measure() == measure()`.
    //
    // DISSOLUTION, so it is a policy rather than a bounded scaffold wearing one's clothes: this
    // cap exists only because the differential rebuilds the whole crate per candidate. It is
    // RAISED, not removed, when that stops being true -- the seed transcript is captured once for
    // a whole selection today, but each candidate install still forces a full rebuild, and a
    // per-module compilation unit would make the cost linear in the changed surface rather than
    // in the crate. Until then a four-authority PR is genuinely more than this gate can check,
    // and saying so is the honest answer rather than checking three of four and printing a pass.
    // A PR that legitimately needs more splits, or the operator raises the number here.
    const MAX_SELECTED_MODULES_PER_RUN: usize = 3;
    if plans.len() > MAX_SELECTED_MODULES_PER_RUN {
        eprintln!(
            "behavioral-receipt: REFUSED — {} authority modules selected, above the declared cap \
             of {MAX_SELECTED_MODULES_PER_RUN}. Not sampling the first {MAX_SELECTED_MODULES_PER_RUN}: \
             a partial check reported as a pass is how a gate stops covering things without anyone \
             finding out. Selected: {}",
            plans.len(),
            plans
                .iter()
                .map(|(_, p, _)| p.module_path.clone())
                .collect::<Vec<_>>()
                .join(", ")
        );
        return Ok(ReceiptPlanOutcome::Ran { agreed: false });
    }

    let emitted = crate::cli_run::emitted_generated_sources()?;
    let mut all_equivalent = true;
    // Accumulated as each module is admitted and printed after every verdict, so the denominator
    // is the SAME partition the admission was decided from. Recomputing it from the plan
    // afterwards would be a second producer of one fact, and the one that gets reported is the
    // one that never gated anything.
    // ONE RESOLVE for the whole run -- see `generated_artifact_body_for_path`'s cost-shape note.
    // The cell may already hold it: selection asks the same authority when a module yields no
    // call, and asking it twice would resolve one closure twice.
    let generated_ctx = generated_artifact_ctx(source_roots, &mut generated_ctx_cell)?;
    let mut denominators: Vec<String> = Vec::new();
    let mut nondeterministic_calls_total = 0usize;
    let mut nondeterministic_modules = 0usize;
    for (mirror, plan, alias) in &plans {
        // WHICH POPULATION OWNS THIS MIRROR, decided before anything is fetched.
        //
        // Two generators write into `src/v1/stage0/src`: the v1 compiler emits module mirrors,
        // and gunbc's artifact emitters write a handful of generated files whose headers say so.
        // This fragment used to ask only the first and refuse when it came back empty, which made
        // the SECOND population permanently unaskable -- any change to the interpreter dispatch
        // roster or the stage0 crate layout redded this run with "a missing candidate is
        // ignorance", correct as written and with no reachable green.
        //
        // The population is asked FIRST and answers positively. This is deliberately not a
        // fallback from the mirror-emit miss: a fallback would make absence in one producer mean
        // presence in the other, so a genuinely unknown path would silently be regenerated from
        // nothing instead of refused. Here each population answers for what it owns, and a path
        // in neither still refuses.
        let repo_rel = format!("src/v1/stage0/src/{mirror}");
        let generated = match generated_artifact_body_for_path(&generated_ctx, &repo_rel) {
            Ok(g) => g,
            Err(e) => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — could not ask the generated-artifact \
                     population about {repo_rel}: {e}. Not equivalence: an unanswered question \
                     is ignorance",
                    plan.module_path
                );
                all_equivalent = false;
                continue;
            }
        };
        let owned_candidate: Option<String> = match generated {
            GeneratedArtifactPathBody::Produced(content) => Some(content),
            GeneratedArtifactPathBody::Refused(reason) => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — {repo_rel} is a generated artifact and its \
                     generator refused: {reason}",
                    plan.module_path
                );
                all_equivalent = false;
                continue;
            }
            GeneratedArtifactPathBody::NotGenerated => None,
        };
        // A GENERATED ARTIFACT IS CHECKED BY IDENTITY, NOT BY CALLING IT -- and this is the
        // correct check for it, not a weaker stand-in for the differential.
        //
        // MEASURED, after the first draft of this change got it wrong. Producing the candidate
        // was necessary and not sufficient: with the candidate in hand the differential went one
        // step further and refused again, because it compiles a driver that CALLS the authority's
        // declared functions against the mirror --
        //
        //   the driver did not compile against the mirror: error[E0425]: cannot find function
        //   `v1_interpreter_arm_shape_derivability` in module crate::v1_interpreter_dispatch_generated
        //
        // -- and `v1_interpreter_dispatch_generated.rs` exposes enums and `lookup_*` fns. It is a
        // file DERIVED FROM the authority's data, not a Rust projection of the authority's
        // functions, so those functions are not there and never will be. The differential's
        // precondition (the mirror answers the same calls the authority declares) simply does not
        // hold for this population.
        //
        // For a generated artifact the whole content IS the product, so byte identity between a
        // freshly generated candidate and the committed file is the complete statement of
        // correctness -- which is exactly the drift check that has had no owner since the
        // generated-artifact drift gates were dropped in the floor cut.
        let candidate_source: &String = match owned_candidate.as_ref() {
            Some(candidate) => {
                let committed_path = workspace.join(&repo_rel);
                match fs::read_to_string(&committed_path) {
                    Err(e) => {
                        eprintln!(
                            "behavioral-receipt: {} REFUSED — {repo_rel} is a generated artifact \
                             but its committed bytes could not be read ({e}), so identity cannot \
                             be established",
                            plan.module_path
                        );
                        all_equivalent = false;
                    }
                    Ok(committed_bytes) if committed_bytes == *candidate => {
                        eprintln!(
                            "behavioral-receipt: {} ARTIFACT-IDENTICAL — {repo_rel} regenerates \
                             byte-for-byte from its authority. This is identity, not behavioural \
                             equivalence: the artifact exposes no function this fragment could \
                             call, so its bytes are the whole claim",
                            plan.module_path
                        );
                    }
                    Ok(_) => {
                        eprintln!(
                            "behavioral-receipt: {} ARTIFACT-DRIFT — {repo_rel} does not match \
                             what its authority generates. Regenerate it (main_wet on \
                             dag/tools/generated_artifact_gate.dag) and commit the result",
                            plan.module_path
                        );
                        all_equivalent = false;
                    }
                }
                continue;
            }
            None => match emitted.get(mirror) {
                Some(c) => c,
                None => {
                    eprintln!(
                        "behavioral-receipt: {} REFUSED — {mirror} is in neither population: the \
                         v1 emit produced no mirror for it and it is not a committed generated \
                         artifact. Not equivalence: a missing candidate is ignorance",
                        plan.module_path
                    );
                    all_equivalent = false;
                    continue;
                }
            },
        };
        // Infallible in fact -- selection only pushed plans that admitted -- but derived here
        // rather than asserted, so the differential's precondition is carried by the value it
        // receives instead of by a comment about an earlier loop.
        let admitted = match AdmittedPlan::of(plan) {
            Ok(a) => a,
            Err(coverage) => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — selected but yields no call ({} uncovered \
                     functions). Selection and admission disagree, which is a defect in this \
                     fragment, not in the authority",
                    plan.module_path,
                    coverage.uncovered.len()
                );
                all_equivalent = false;
                continue;
            }
        };
        denominators.push(format!(
            "behavioral-receipt: DENOMINATOR {} — {} derived calls over {} of {} declared \
             functions; the other {} yield no call and are NOT covered by this verdict",
            plan.module_path,
            admitted.coverage.calls(),
            admitted.coverage.covered.len(),
            plan.parsed_signatures,
            admitted.coverage.uncovered.len()
        ));
        match behavioral_differential(
            &workspace,
            &ReceiptCrate::v1_compiler(),
            &workspace.join("src/v1/stage0/src").join(mirror),
            candidate_source,
            &admitted,
            alias,
        ) {
            ReceiptVerdict::Equivalent {
                calls,
                nondeterministic_calls,
                nondeterministic_functions,
            } => {
                if nondeterministic_calls == 0 {
                    eprintln!(
                        "behavioral-receipt: {} EQUIVALENT over {calls} derived calls",
                        plan.module_path
                    );
                } else {
                    eprintln!(
                        "behavioral-receipt: {} EQUIVALENT over {calls} derived calls, with \
                         {nondeterministic_calls} EXCLUDED as nondeterministically rendered — \
                         {}. Those calls were NOT compared, so this green does not cover them",
                        plan.module_path,
                        nondeterministic_functions.join(", ")
                    );
                    nondeterministic_calls_total += nondeterministic_calls;
                    nondeterministic_modules += 1;
                }
            }
            ReceiptVerdict::NondeterministicRendering {
                unstable_calls,
                functions,
            } => {
                eprintln!(
                    "behavioral-receipt: {} NONDETERMINISTIC-RENDERING — all {unstable_calls} \
                     derived call(s) render unstably, so nothing could be compared: {}. This is \
                     a property of what the mirror RETURNS, not a defect in this diff, and it is \
                     not scored as a divergence",
                    plan.module_path,
                    functions.join(", ")
                );
                nondeterministic_calls_total += unstable_calls;
                nondeterministic_modules += 1;
            }
            ReceiptVerdict::Divergent {
                calls,
                first_difference,
            } => {
                eprintln!(
                    "behavioral-receipt: {} DIVERGENT over {calls} derived calls — {first_difference}",
                    plan.module_path
                );
                all_equivalent = false;
            }
            ReceiptVerdict::Refused { reason } => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — {reason}",
                    plan.module_path
                );
                all_equivalent = false;
            }
        }
    }
    // THE DENOMINATOR, EVERY RUN (operator ruling, 2026-08-20). A green here means the DERIVED
    // CALLS in the selected modules agreed -- never that a module is behaviourally equivalent.
    // Printing a bare PASS is how, inside a week, someone reads this as promotion evidence.
    for line in &denominators {
        eprintln!("{line}");
    }
    // EVERY RUN, INCLUDING ZERO -- a counter that appears only when nonzero teaches a reader that
    // its absence means "not measured", and the two then look alike in a log tail.
    //
    // THE FLOOR IS IN THE LINE, NOT IN A NOTE BESIDE IT. The line outlives the note: someone will
    // trend this number, watch it sit at N, and conclude the class is nearly closed. It is a floor
    // because the probe proves instability by DISAGREEMENT between two runs, and two randomized
    // renderings can coincide -- an uncaught call is scored as an ordinary comparison and, if it
    // then differs across the seed/candidate pair, inflates the DIVERGENT count instead.
    //
    // DISSOLUTION: this goes to zero when emission is deterministic (a `BTreeMap` container
    // template rather than `HashMap`), NOT when the probe gets better at spotting the residue.
    // A shrinking count from a sharper probe would be the metric improving while the defect stays.
    eprintln!(
        "behavioral-receipt: nondeterministic_rendering={nondeterministic_calls_total} call(s) \
         across {nondeterministic_modules} module(s) — FLOOR, not a total: instability is proved \
         by two runs disagreeing, so a call whose randomized rendering happened to agree twice is \
         not counted here and is compared as if it were deterministic. Not failures; each is a \
         subject this fragment cannot ask about until emission is deterministic"
    );
    Ok(ReceiptPlanOutcome::Ran {
        agreed: all_equivalent,
    })
}

#[cfg(test)]
mod driver_transcript_tests {
    use super::*;

    fn lines(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    /// The property the exclusion rests on: two runs of ONE binary that disagree at an index
    /// prove that index is not a function of the program alone.
    #[test]
    fn a_line_that_differs_between_two_runs_is_unstable() {
        let t = DriverTranscript::of(
            lines(&["stable() = 1", "m() = {a, b}", "also_stable() = 2"]),
            lines(&["stable() = 1", "m() = {b, a}", "also_stable() = 2"]),
        );
        assert_eq!(t.unstable, [1].into_iter().collect());
        // The transcript itself is the FIRST run, unchanged -- the probe observes, it does not
        // rewrite what gets compared.
        assert_eq!(
            t.lines,
            lines(&["stable() = 1", "m() = {a, b}", "also_stable() = 2"])
        );
    }

    /// THE FALSE-POSITIVE CONTROL, at unit grain. A deterministic corpus must yield NOTHING
    /// unstable, or every module would silently stop being compared while still printing a green.
    #[test]
    fn two_identical_runs_mark_nothing_unstable() {
        let t = DriverTranscript::of(
            lines(&["a() = 1", "b() = 2"]),
            lines(&["a() = 1", "b() = 2"]),
        );
        assert!(t.unstable.is_empty());
    }

    /// A length difference is instability that belongs to no single index. Marking every line is
    /// the fail-closed reading; comparing the common prefix would silently compare a shifted pair.
    #[test]
    fn a_length_difference_marks_every_line() {
        let t = DriverTranscript::of(lines(&["a() = 1"]), lines(&["a() = 1", "b() = 2"]));
        assert_eq!(t.unstable, [0, 1].into_iter().collect());
    }

    /// THE RESIDUE, ASSERTED SO IT IS NOT MISTAKEN FOR A CLOSED CLASS. Two randomized renderings
    /// can coincide; when they do, the probe cannot see it and the call is compared as if it were
    /// deterministic. This test PINS that limitation rather than hiding it -- if someone later
    /// makes the probe complete, this test fails and forces the FLOOR wording to be revisited.
    #[test]
    fn a_nondeterministic_call_that_agreed_twice_is_not_caught() {
        let t = DriverTranscript::of(lines(&["m() = {a, b}"]), lines(&["m() = {a, b}"]));
        assert!(t.unstable.is_empty());
    }
}

#[cfg(test)]
mod function_grain_admission_tests {
    use super::*;

    fn plan(derivable: Vec<(&str, usize)>, refused: Vec<(&str, RefusalCause)>) -> ModuleCorpusPlan {
        let declared = derivable.len() + refused.len();
        ModuleCorpusPlan {
            module_path: "test.module".to_string(),
            derivable: derivable
                .into_iter()
                .map(|(name, calls)| {
                    (
                        name.to_string(),
                        EnumeratedDomain::Exhaustive { cardinality: calls },
                        (0..calls).map(|i| vec![format!("{i}i64")]).collect(),
                    )
                })
                .collect(),
            refused: refused
                .into_iter()
                .map(|(name, cause)| (name.to_string(), cause))
                .collect(),
            declared_fn_lines: declared,
            parsed_signatures: declared,
        }
    }

    fn unbounded() -> RefusalCause {
        RefusalCause::UnboundedString {
            ty: "String".to_string(),
        }
    }

    /// BOTH ARMS OF THE ONE DEFECT, IN ONE TEST, and it goes green only when BOTH are closed.
    ///
    /// The specimen is gunbc#8704, which tripped both in a single run:
    /// `v2.compiler.self_host.stage0_crate_layout` (a wet-actuator mirror the emit never writes)
    /// and `gunbc.stage0_crate_layout_generated` (zero declared functions). A fix closing one
    /// half leaves a live red reachable by the very change that motivated it, so the assertions
    /// are stated together rather than in two tests that could be satisfied separately.
    #[test]
    fn a_zero_function_authority_is_excluded_as_such_and_not_as_a_derivation_deficit() {
        let mut p = plan(vec![], vec![]);
        p.declared_fn_lines = 0;
        p.parsed_signatures = 0;
        let coverage = function_grain_coverage(&p);
        match plan_grain_selection(&p, coverage).expect("a module with no functions is a fact") {
            Some(ReceiptExclusion::NoFunctionDeclared { module_path }) => {
                assert_eq!(module_path, "test.module")
            }
            other => panic!(
                "an authority declaring no function has no behaviour that could diverge, so it \
                 must be excluded AS SUCH -- reporting it as `none of its 0 declared functions \
                 yields a call` sends a reader to close a derivation with no subject: {other:?}"
            ),
        }
    }

    /// THE PLANTED CONTROL FOR AN EMPTY POPULATION, and it is planted precisely BECAUSE the
    /// population is empty. No authority in the corpus today declares zero functions AND owns a
    /// generated artifact, so nothing live exercises this branch -- which means main and this
    /// branch disagree about it with no measurement in either direction, and the first real member
    /// would decide the verdict by accident. This test is the statement of intent, executing:
    ///
    ///   such a module is SELECTED, not excluded, because its subject is the artifact's BYTES.
    ///
    /// The identity check in the differential loop calls no function, so "declares no function"
    /// says nothing about whether it can be checked -- and that check is the only drift observer
    /// those artifacts have had since the floor cut removed the generated-artifact drift gates
    /// (DESIGN.md names them first in what the cut left unguarded). Excluding here would delete
    /// it. Per DESIGN.md 4b(4) this control stays enrolled once the population is non-empty; it is
    /// the evidence that the higher rung is real, not scaffolding for its absence.
    #[test]
    fn a_zero_function_authority_owning_a_generated_artifact_is_selected_not_excluded() {
        let excluded = ReceiptExclusion::NoFunctionDeclared {
            module_path: "test.module".to_string(),
        };
        let survives = exclusion_survives_generated_artifact_population(
            excluded,
            &GeneratedArtifactPathBody::Produced("generated bytes".to_string()),
        );
        assert!(
            survives.is_none(),
            "an artifact-owning authority has a subject -- its bytes -- and excluding it would \
             delete the only drift observer those artifacts have"
        );
    }

    /// The same for a generator that REFUSED: the path still belongs to the artifact population,
    /// and the loop is where that refusal is reported. Swallowing it here would turn a generator
    /// failure into a quiet exclusion -- a refusal downgraded to a skip.
    #[test]
    fn a_generator_refusal_does_not_become_an_exclusion() {
        let excluded = ReceiptExclusion::NoFunctionHasACorpus {
            module_path: "test.module".to_string(),
            uncovered: vec![("alpha".to_string(), unbounded())],
        };
        assert!(exclusion_survives_generated_artifact_population(
            excluded,
            &GeneratedArtifactPathBody::Refused("generator said no".to_string()),
        )
        .is_none());
    }

    /// THE OTHER SIDE OF THE SAME CONTROL, without which the two above could be satisfied by never
    /// excluding anything: an ordinary module mirror answers `NotGenerated`, and there the
    /// exclusion IS the fact and must survive.
    #[test]
    fn an_ordinary_mirror_keeps_its_exclusion() {
        let excluded = ReceiptExclusion::NoFunctionDeclared {
            module_path: "test.module".to_string(),
        };
        match exclusion_survives_generated_artifact_population(
            excluded,
            &GeneratedArtifactPathBody::NotGenerated,
        ) {
            Some(ReceiptExclusion::NoFunctionDeclared { module_path }) => {
                assert_eq!(module_path, "test.module")
            }
            other => panic!("a module mirror with no function has no subject at all: {other:?}"),
        }
    }

    /// THE FALSE-POSITIVE CONTROL for the arm above, read beside it: a module that DOES declare
    /// functions, none of them derivable, must still land on the derivation-deficit arm. Without
    /// this, a fix could satisfy the test above by sending every uncovered module to the new arm
    /// and the deficit population would silently go to zero.
    #[test]
    fn a_module_whose_functions_all_refuse_stays_a_derivation_deficit() {
        let p = plan(vec![], vec![("alpha", unbounded())]);
        let coverage = function_grain_coverage(&p);
        match plan_grain_selection(&p, coverage).expect("non-derivability is a fact, not ignorance")
        {
            Some(ReceiptExclusion::NoFunctionHasACorpus { uncovered, .. }) => {
                assert_eq!(uncovered.len(), 1)
            }
            other => panic!("a declared-but-underivable function is a rankable deficit: {other:?}"),
        }
    }

    /// A module with one derivable function is SELECTED -- the arm that must not be swallowed by
    /// either exclusion above.
    #[test]
    fn a_module_with_a_derivable_function_is_selected() {
        let p = plan(vec![("alpha", 2)], vec![("beta", unbounded())]);
        let coverage = function_grain_coverage(&p);
        assert!(plan_grain_selection(&p, coverage)
            .expect("a derivable function is a subject")
            .is_none());
    }

    /// THE READER-BLINDNESS ARM. `fn ` lines declared and no signature parsed is a disagreement
    /// between two readers of one fact; answering it with an exclusion would publish this
    /// fragment's own failure as a property of the authority, so it refuses.
    #[test]
    fn a_parse_that_sees_none_of_the_declared_functions_refuses_rather_than_excluding() {
        let mut p = plan(vec![], vec![]);
        p.declared_fn_lines = 7;
        p.parsed_signatures = 0;
        let coverage = function_grain_coverage(&p);
        let refusal = plan_grain_selection(&p, coverage)
            .expect_err("two readers disagreeing is ignorance, not a fact about the module");
        assert!(
            refusal.contains("readers disagree") && refusal.contains("test.module"),
            "the refusal must name what it refused and why: {refusal}"
        );
    }

    /// THE DISCRIMINATING RED for the whole change. A module whose every declared function refuses
    /// derivation used to reach the differential and come back `Refused`, which made the diff that
    /// touched it hard-fail required CI. It must now fail admission instead -- and name every
    /// function, because a module-level "nothing derived" is not something an author can act on.
    #[test]
    fn a_module_with_no_derivable_function_is_excluded_not_refused() {
        let p = plan(vec![], vec![("alpha", unbounded()), ("beta", unbounded())]);
        let coverage = AdmittedPlan::of(&p).err().expect(
            "a module yielding no call must not be admissible to a differential that would \
             compare a program against itself over an empty transcript",
        );
        let named: Vec<&str> = coverage.uncovered.iter().map(|(f, _)| f.as_str()).collect();
        assert_eq!(named, vec!["alpha", "beta"]);
        assert_eq!(coverage.calls(), 0);
    }

    /// THE POSITIVE CONTROL, and the one that makes the red above load-bearing rather than a
    /// mechanism that refuses everything. One derivable function is enough to admit, and the
    /// refused sibling stays counted as uncovered rather than disappearing into the pass.
    #[test]
    fn one_function_with_a_call_admits_and_the_rest_stay_counted() {
        let p = plan(vec![("alpha", 3)], vec![("beta", unbounded())]);
        let admitted = AdmittedPlan::of(&p).expect("one function with calls must admit");
        assert_eq!(admitted.coverage.covered, vec![("alpha".to_string(), 3)]);
        assert_eq!(admitted.coverage.calls(), 3);
        assert_eq!(
            admitted
                .coverage
                .uncovered
                .iter()
                .map(|(f, _)| f.as_str())
                .collect::<Vec<_>>(),
            vec!["beta"],
            "a function nothing ran must remain visible in the denominator of a passing module"
        );
    }

    /// THE SUBTLER HALF: a function that derived a domain containing nothing sits in `derivable`
    /// and contributes no call. Counting it as covered is a zero that reads as success. It is
    /// uncovered, with its OWN cause -- separate from the never-derivable ones, whose remedy is a
    /// different piece of work.
    #[test]
    fn empty_derived_domain_is_uncovered_not_covered() {
        let p = plan(vec![("alpha", 0)], vec![("beta", unbounded())]);
        assert_eq!(
            p.derivable.len(),
            1,
            "precondition: the plan does record this function as derivable, which is exactly why \
             counting that vector would over-report coverage"
        );
        let coverage = AdmittedPlan::of(&p)
            .err()
            .expect("a derivable function with an empty domain yields no call, so nothing admits");
        assert_eq!(
            coverage.uncovered,
            vec![
                ("alpha".to_string(), RefusalCause::EmptyDerivedDomain),
                ("beta".to_string(), unbounded()),
            ],
            "the two causes must stay distinguishable: one needs an enumerator fixed, the other \
             needs a type grounded"
        );
    }

    /// A module carrying an empty-domain function BESIDE a real one still admits, and the empty
    /// one is not silently promoted into the covered count by the module having passed.
    #[test]
    fn an_empty_domain_function_does_not_ride_a_covered_sibling() {
        let p = plan(vec![("alpha", 2), ("empty", 0)], vec![]);
        let admitted = AdmittedPlan::of(&p).expect("alpha yields calls, so the module admits");
        assert_eq!(admitted.coverage.covered, vec![("alpha".to_string(), 2)]);
        assert_eq!(
            admitted.coverage.uncovered,
            vec![("empty".to_string(), RefusalCause::EmptyDerivedDomain)]
        );
    }
}
