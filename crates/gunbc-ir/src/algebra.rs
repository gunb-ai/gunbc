//! Algebraic primitives for the IR.
//!
//! These types provide closed-form structures for validation, guards, and constraints.
//! They're "for us" — authoring and validation conveniences that don't affect
//! the core execution model (which only sees nodes, ports, and edges).

use std::fmt;

/// Set specification — explicit membership semantics.
///
/// This eliminates the ambiguity where an empty collection could mean
/// "nothing" or "everything" depending on context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetSpec<T> {
    /// The empty set — contains nothing.
    Empty,
    /// The universal set — contains everything.
    Universal,
    /// A specific set of elements.
    These(Vec<T>),
}

impl<T: PartialEq> SetSpec<T> {
    /// Check if an item is a member of this set.
    pub fn contains(&self, item: &T) -> bool {
        match self {
            SetSpec::Empty => false,
            SetSpec::Universal => true,
            SetSpec::These(items) => items.contains(item),
        }
    }

    /// Check if this set is empty (contains no elements).
    pub fn is_empty(&self) -> bool {
        match self {
            SetSpec::Empty => true,
            SetSpec::Universal => false,
            SetSpec::These(items) => items.is_empty(),
        }
    }

    /// Check if this set is universal (contains all elements).
    pub fn is_universal(&self) -> bool {
        matches!(self, SetSpec::Universal)
    }
}

impl<T: PartialEq + Clone> SetSpec<T> {
    /// Set union: self ∪ other
    pub fn union(&self, other: &Self) -> Self {
        match (self, other) {
            // ∅ ∪ S = S
            (SetSpec::Empty, s) | (s, SetSpec::Empty) => s.clone(),
            // U ∪ S = U
            (SetSpec::Universal, _) | (_, SetSpec::Universal) => SetSpec::Universal,
            // {a,b} ∪ {c,d} = {a,b,c,d} (deduplicated)
            (SetSpec::These(a), SetSpec::These(b)) => {
                let mut result = a.clone();
                for item in b {
                    if !result.contains(item) {
                        result.push(item.clone());
                    }
                }
                SetSpec::These(result)
            }
        }
    }

    /// Set intersection: self ∩ other
    pub fn intersect(&self, other: &Self) -> Self {
        match (self, other) {
            // ∅ ∩ S = ∅
            (SetSpec::Empty, _) | (_, SetSpec::Empty) => SetSpec::Empty,
            // U ∩ S = S
            (SetSpec::Universal, s) | (s, SetSpec::Universal) => s.clone(),
            // {a,b} ∩ {b,c} = {b}
            (SetSpec::These(a), SetSpec::These(b)) => {
                let result: Vec<T> = a.iter().filter(|x| b.contains(x)).cloned().collect();
                if result.is_empty() {
                    SetSpec::Empty
                } else {
                    SetSpec::These(result)
                }
            }
        }
    }

    /// Check if self ⊆ other
    pub fn is_subset_of(&self, other: &Self) -> bool {
        match (self, other) {
            // ∅ ⊆ S (for all S)
            (SetSpec::Empty, _) => true,
            // S ⊆ U (for all S)
            (_, SetSpec::Universal) => true,
            // U ⊆ S only if S = U
            (SetSpec::Universal, _) => false,
            // {a,b} ⊆ ∅ only if {a,b} is empty
            (SetSpec::These(items), SetSpec::Empty) => items.is_empty(),
            // {a,b} ⊆ {a,b,c} if all elements of left are in right
            (SetSpec::These(a), SetSpec::These(b)) => a.iter().all(|x| b.contains(x)),
        }
    }
}

impl<T> Default for SetSpec<T> {
    fn default() -> Self {
        SetSpec::Empty
    }
}

/// Runtime value that can flow through the graph.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Unit value — used for signals/triggers with no data.
    Unit,
    /// Boolean value.
    Bool(bool),
    /// Integer value (64-bit signed).
    Int(i64),
    /// Floating point value (64-bit).
    Float(f64),
    /// String value.
    String(String),
    /// Bytes value.
    Bytes(Vec<u8>),
    /// List of values.
    List(Vec<Value>),
    /// Set of values.
    Set(SetSpec<Box<Value>>),
    /// The "skipped" sentinel — propagates when a guard fails.
    Skipped,
}

impl Value {
    /// Check if this value is the Skipped sentinel.
    pub fn is_skipped(&self) -> bool {
        matches!(self, Value::Skipped)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bytes(b) => write!(f, "<{} bytes>", b.len()),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Set(set) => write!(f, "{:?}", set),
            Value::Skipped => write!(f, "<skipped>"),
        }
    }
}

/// Predicate — a boolean function on values.
///
/// Used for guards on ports and conditional logic.
#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    /// Always true.
    True,
    /// Always false.
    False,
    /// Value equals this literal.
    Eq(Value),
    /// Value does not equal this literal.
    NotEq(Value),
    /// Value is a member of this set.
    In(SetSpec<Value>),
    /// Value is not a member of this set.
    NotIn(SetSpec<Value>),
    /// Logical AND of two predicates.
    And(Box<Predicate>, Box<Predicate>),
    /// Logical OR of two predicates.
    Or(Box<Predicate>, Box<Predicate>),
    /// Logical NOT of a predicate.
    Not(Box<Predicate>),
    /// Value is the Skipped sentinel.
    IsSkipped,
    /// Value is not the Skipped sentinel.
    NotSkipped,
}

impl Predicate {
    /// Evaluate this predicate against a value.
    pub fn evaluate(&self, value: &Value) -> bool {
        match self {
            Predicate::True => true,
            Predicate::False => false,
            Predicate::Eq(expected) => value == expected,
            Predicate::NotEq(expected) => value != expected,
            Predicate::In(set) => set.contains(value),
            Predicate::NotIn(set) => !set.contains(value),
            Predicate::And(a, b) => a.evaluate(value) && b.evaluate(value),
            Predicate::Or(a, b) => a.evaluate(value) || b.evaluate(value),
            Predicate::Not(p) => !p.evaluate(value),
            Predicate::IsSkipped => value.is_skipped(),
            Predicate::NotSkipped => !value.is_skipped(),
        }
    }

    /// Combine two predicates with AND.
    pub fn and(self, other: Self) -> Self {
        match (&self, &other) {
            (Predicate::True, _) => other,
            (_, Predicate::True) => self,
            (Predicate::False, _) | (_, Predicate::False) => Predicate::False,
            _ => Predicate::And(Box::new(self), Box::new(other)),
        }
    }

    /// Combine two predicates with OR.
    pub fn or(self, other: Self) -> Self {
        match (&self, &other) {
            (Predicate::False, _) => other,
            (_, Predicate::False) => self,
            (Predicate::True, _) | (_, Predicate::True) => Predicate::True,
            _ => Predicate::Or(Box::new(self), Box::new(other)),
        }
    }

    /// Negate a predicate.
    pub fn not(self) -> Self {
        match self {
            Predicate::True => Predicate::False,
            Predicate::False => Predicate::True,
            Predicate::Not(inner) => *inner,
            other => Predicate::Not(Box::new(other)),
        }
    }
}

impl Default for Predicate {
    fn default() -> Self {
        Predicate::True
    }
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Predicate::True => write!(f, "true"),
            Predicate::False => write!(f, "false"),
            Predicate::Eq(v) => write!(f, "== {}", v),
            Predicate::NotEq(v) => write!(f, "!= {}", v),
            Predicate::In(set) => write!(f, "in {:?}", set),
            Predicate::NotIn(set) => write!(f, "not in {:?}", set),
            Predicate::And(a, b) => write!(f, "({} && {})", a, b),
            Predicate::Or(a, b) => write!(f, "({} || {})", a, b),
            Predicate::Not(p) => write!(f, "!{}", p),
            Predicate::IsSkipped => write!(f, "is_skipped"),
            Predicate::NotSkipped => write!(f, "not_skipped"),
        }
    }
}

// ========== Resource Claims ==========

/// Identifier for a resource that can be claimed.
///
/// Resources are abstract — they could represent:
/// - A lock on a file path
/// - A lease on a database connection
/// - A budget allocation (money, API credits)
/// - A rate limit slot
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(pub String);

impl ResourceId {
    pub fn new(id: impl Into<String>) -> Self {
        ResourceId(id.into())
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Amount of a resource being claimed.
///
/// This is a flexible unit that means different things for different exclusion modes:
/// - For locks: typically 1 (present/absent)
/// - For leases: duration in milliseconds
/// - For budgets: quantity (money, credits, bytes)
/// - For rate limits: number of operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Amount(pub f64);

impl Amount {
    pub fn one() -> Self {
        Amount(1.0)
    }

    pub fn zero() -> Self {
        Amount(0.0)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}

impl Default for Amount {
    fn default() -> Self {
        Amount::one()
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// How a resource is exclusively controlled.
///
/// This is a unified model for different kinds of resource exclusion:
/// - Locks (exclusive/shared access)
/// - Leases (time-bounded locks)
/// - Budgets (consumable allocations)
/// - Rate limits (frequency constraints)
#[derive(Debug, Clone, PartialEq)]
pub enum ExclusionMode {
    /// Exclusive access — no other claims can coexist.
    /// Used for write locks, critical sections.
    Exclusive,

    /// Shared access — multiple readers can coexist, but not with exclusive.
    /// Used for read locks.
    Shared,

    /// Time-bounded exclusive access.
    /// Amount specifies duration in milliseconds.
    Lease {
        /// Maximum time to hold the resource.
        timeout_ms: u64,
    },

    /// Consumable allocation — amount is deducted from a pool.
    /// Used for budgets, credits, quotas.
    Budget,

    /// Frequency constraint — limits operations per time window.
    /// Used for API rate limits, throttling.
    RateLimit {
        /// Time window in milliseconds.
        window_ms: u64,
        /// Maximum operations per window.
        max_ops: u64,
    },
}

impl Default for ExclusionMode {
    fn default() -> Self {
        ExclusionMode::Exclusive
    }
}

impl fmt::Display for ExclusionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExclusionMode::Exclusive => write!(f, "exclusive"),
            ExclusionMode::Shared => write!(f, "shared"),
            ExclusionMode::Lease { timeout_ms } => write!(f, "lease({}ms)", timeout_ms),
            ExclusionMode::Budget => write!(f, "budget"),
            ExclusionMode::RateLimit { window_ms, max_ops } => {
                write!(f, "rate_limit({}/{} ms)", max_ops, window_ms)
            }
        }
    }
}

/// A claim on a resource.
///
/// This is the unified primitive for all resource management:
/// - Locks: `ResourceClaim { id: "file.txt", amount: 1, mode: Exclusive }`
/// - Leases: `ResourceClaim { id: "db_conn", amount: 1, mode: Lease { timeout_ms: 5000 } }`
/// - Budgets: `ResourceClaim { id: "api_credits", amount: 100, mode: Budget }`
/// - Rate limits: `ResourceClaim { id: "api/v1", amount: 1, mode: RateLimit { window_ms: 1000, max_ops: 10 } }`
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceClaim {
    /// The resource being claimed.
    pub resource: ResourceId,
    /// How much of the resource is needed.
    pub amount: Amount,
    /// How the resource is exclusively controlled.
    pub mode: ExclusionMode,
}

impl ResourceClaim {
    /// Create a new resource claim.
    pub fn new(resource: impl Into<String>, amount: Amount, mode: ExclusionMode) -> Self {
        ResourceClaim {
            resource: ResourceId::new(resource),
            amount,
            mode,
        }
    }

    /// Create an exclusive lock claim.
    pub fn exclusive(resource: impl Into<String>) -> Self {
        Self::new(resource, Amount::one(), ExclusionMode::Exclusive)
    }

    /// Create a shared lock claim.
    pub fn shared(resource: impl Into<String>) -> Self {
        Self::new(resource, Amount::one(), ExclusionMode::Shared)
    }

    /// Create a lease claim.
    pub fn lease(resource: impl Into<String>, timeout_ms: u64) -> Self {
        Self::new(resource, Amount::one(), ExclusionMode::Lease { timeout_ms })
    }

    /// Create a budget claim.
    pub fn budget(resource: impl Into<String>, amount: f64) -> Self {
        Self::new(resource, Amount(amount), ExclusionMode::Budget)
    }

    /// Create a rate limit claim.
    pub fn rate_limit(resource: impl Into<String>, window_ms: u64, max_ops: u64) -> Self {
        Self::new(
            resource,
            Amount::one(),
            ExclusionMode::RateLimit { window_ms, max_ops },
        )
    }

    /// Check if two claims conflict (cannot coexist).
    ///
    /// Conflict rules:
    /// - Different resources never conflict
    /// - Exclusive conflicts with everything (including other exclusive)
    /// - Shared only conflicts with exclusive
    /// - Lease conflicts like exclusive during its duration
    /// - Budget conflicts if combined amount exceeds available
    /// - RateLimit conflicts if combined ops exceed max_ops
    pub fn conflicts_with(&self, other: &ResourceClaim) -> bool {
        // Different resources never conflict
        if self.resource != other.resource {
            return false;
        }

        // Same resource — check modes
        match (&self.mode, &other.mode) {
            // Exclusive conflicts with everything
            (ExclusionMode::Exclusive, _) | (_, ExclusionMode::Exclusive) => true,

            // Shared only conflicts with exclusive (handled above)
            (ExclusionMode::Shared, ExclusionMode::Shared) => false,

            // Lease conflicts like exclusive
            (ExclusionMode::Lease { .. }, _) | (_, ExclusionMode::Lease { .. }) => true,

            // Budget and rate limit need external state to determine conflict
            // For now, we assume they can coexist (scheduler handles actual limits)
            (ExclusionMode::Budget, ExclusionMode::Budget) => false,
            (ExclusionMode::RateLimit { .. }, ExclusionMode::RateLimit { .. }) => false,

            // Mixed modes that share a resource are conservatively conflicting
            _ => true,
        }
    }
}

impl fmt::Display for ResourceClaim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}, {}]", self.resource, self.amount, self.mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== SetSpec tests ==========

    #[test]
    fn setspec_empty_contains_nothing() {
        let set: SetSpec<i32> = SetSpec::Empty;
        assert!(!set.contains(&1));
        assert!(!set.contains(&0));
        assert!(!set.contains(&-999));
    }

    #[test]
    fn setspec_universal_contains_everything() {
        let set: SetSpec<i32> = SetSpec::Universal;
        assert!(set.contains(&1));
        assert!(set.contains(&0));
        assert!(set.contains(&-999));
    }

    #[test]
    fn setspec_these_contains_specified() {
        let set = SetSpec::These(vec![1, 2, 3]);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
        assert!(!set.contains(&4));
        assert!(!set.contains(&0));
    }

    #[test]
    fn setspec_union_identity() {
        let s = SetSpec::These(vec![1, 2]);
        // ∅ ∪ S = S
        assert_eq!(SetSpec::Empty.union(&s), s);
        assert_eq!(s.union(&SetSpec::Empty), s);
    }

    #[test]
    fn setspec_union_universal() {
        let s = SetSpec::These(vec![1, 2]);
        // U ∪ S = U
        assert_eq!(SetSpec::Universal.union(&s), SetSpec::Universal);
        assert_eq!(s.union(&SetSpec::Universal), SetSpec::Universal);
    }

    #[test]
    fn setspec_union_these() {
        let a = SetSpec::These(vec![1, 2]);
        let b = SetSpec::These(vec![2, 3]);
        let result = a.union(&b);
        if let SetSpec::These(items) = result {
            assert!(items.contains(&1));
            assert!(items.contains(&2));
            assert!(items.contains(&3));
            assert_eq!(items.len(), 3); // no duplicates
        } else {
            panic!("Expected These");
        }
    }

    #[test]
    fn setspec_intersect_empty() {
        let s = SetSpec::These(vec![1, 2]);
        // ∅ ∩ S = ∅
        assert_eq!(SetSpec::Empty.intersect(&s), SetSpec::Empty);
        assert_eq!(s.intersect(&SetSpec::Empty), SetSpec::Empty);
    }

    #[test]
    fn setspec_intersect_universal() {
        let s = SetSpec::These(vec![1, 2]);
        // U ∩ S = S
        assert_eq!(SetSpec::Universal.intersect(&s), s);
        assert_eq!(s.intersect(&SetSpec::Universal), s);
    }

    #[test]
    fn setspec_intersect_these() {
        let a = SetSpec::These(vec![1, 2, 3]);
        let b = SetSpec::These(vec![2, 3, 4]);
        let result = a.intersect(&b);
        if let SetSpec::These(items) = result {
            assert!(items.contains(&2));
            assert!(items.contains(&3));
            assert!(!items.contains(&1));
            assert!(!items.contains(&4));
        } else {
            panic!("Expected These");
        }
    }

    #[test]
    fn setspec_intersect_disjoint() {
        let a = SetSpec::These(vec![1, 2]);
        let b = SetSpec::These(vec![3, 4]);
        assert_eq!(a.intersect(&b), SetSpec::Empty);
    }

    #[test]
    fn setspec_subset_empty() {
        // ∅ ⊆ S (for all S)
        assert!(SetSpec::<i32>::Empty.is_subset_of(&SetSpec::Empty));
        assert!(SetSpec::<i32>::Empty.is_subset_of(&SetSpec::Universal));
        assert!(SetSpec::<i32>::Empty.is_subset_of(&SetSpec::These(vec![1])));
    }

    #[test]
    fn setspec_subset_universal() {
        // S ⊆ U (for all S)
        assert!(SetSpec::<i32>::Empty.is_subset_of(&SetSpec::Universal));
        assert!(SetSpec::<i32>::Universal.is_subset_of(&SetSpec::Universal));
        assert!(SetSpec::These(vec![1, 2, 3]).is_subset_of(&SetSpec::Universal));
        // U ⊆ S only if S = U
        assert!(!SetSpec::<i32>::Universal.is_subset_of(&SetSpec::Empty));
        assert!(!SetSpec::<i32>::Universal.is_subset_of(&SetSpec::These(vec![1])));
    }

    #[test]
    fn setspec_subset_these() {
        let small = SetSpec::These(vec![1, 2]);
        let large = SetSpec::These(vec![1, 2, 3]);
        assert!(small.is_subset_of(&large));
        assert!(!large.is_subset_of(&small));
        assert!(small.is_subset_of(&small)); // reflexive
    }

    // ========== Value tests ==========

    #[test]
    fn value_skipped_detection() {
        assert!(Value::Skipped.is_skipped());
        assert!(!Value::Unit.is_skipped());
        assert!(!Value::Bool(true).is_skipped());
        assert!(!Value::Int(42).is_skipped());
    }

    #[test]
    fn value_display() {
        assert_eq!(format!("{}", Value::Unit), "()");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Int(42)), "42");
        assert_eq!(format!("{}", Value::String("hello".into())), "\"hello\"");
        assert_eq!(format!("{}", Value::Skipped), "<skipped>");
    }

    // ========== Predicate tests ==========

    #[test]
    fn predicate_true_false() {
        let v = Value::Int(42);
        assert!(Predicate::True.evaluate(&v));
        assert!(!Predicate::False.evaluate(&v));
    }

    #[test]
    fn predicate_eq() {
        let v = Value::Int(42);
        assert!(Predicate::Eq(Value::Int(42)).evaluate(&v));
        assert!(!Predicate::Eq(Value::Int(0)).evaluate(&v));
    }

    #[test]
    fn predicate_not_eq() {
        let v = Value::Int(42);
        assert!(!Predicate::NotEq(Value::Int(42)).evaluate(&v));
        assert!(Predicate::NotEq(Value::Int(0)).evaluate(&v));
    }

    #[test]
    fn predicate_in_set() {
        let v = Value::Int(2);
        let set = SetSpec::These(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert!(Predicate::In(set.clone()).evaluate(&v));
        assert!(!Predicate::In(set).evaluate(&Value::Int(4)));
    }

    #[test]
    fn predicate_not_in_set() {
        let v = Value::Int(4);
        let set = SetSpec::These(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert!(Predicate::NotIn(set.clone()).evaluate(&v));
        assert!(!Predicate::NotIn(set).evaluate(&Value::Int(2)));
    }

    #[test]
    fn predicate_and() {
        let v = Value::Int(5);
        let gt_zero = Predicate::NotEq(Value::Int(0));
        let not_ten = Predicate::NotEq(Value::Int(10));
        let combined = gt_zero.and(not_ten);
        assert!(combined.evaluate(&v));
        assert!(!Predicate::True.and(Predicate::False).evaluate(&v));
    }

    #[test]
    fn predicate_or() {
        let v = Value::Int(5);
        let is_five = Predicate::Eq(Value::Int(5));
        let is_ten = Predicate::Eq(Value::Int(10));
        assert!(is_five.clone().or(is_ten.clone()).evaluate(&v));
        assert!(!Predicate::False.or(Predicate::False).evaluate(&v));
    }

    #[test]
    fn predicate_not() {
        let v = Value::Int(5);
        assert!(!Predicate::True.not().evaluate(&v));
        assert!(Predicate::False.not().evaluate(&v));
        assert!(!Predicate::Eq(Value::Int(5)).not().evaluate(&v));
    }

    #[test]
    fn predicate_is_skipped() {
        assert!(Predicate::IsSkipped.evaluate(&Value::Skipped));
        assert!(!Predicate::IsSkipped.evaluate(&Value::Int(42)));
        assert!(!Predicate::NotSkipped.evaluate(&Value::Skipped));
        assert!(Predicate::NotSkipped.evaluate(&Value::Int(42)));
    }

    #[test]
    fn predicate_simplification() {
        // True AND x = x
        assert_eq!(
            Predicate::True.and(Predicate::Eq(Value::Int(1))),
            Predicate::Eq(Value::Int(1))
        );
        // False AND x = False
        assert_eq!(
            Predicate::False.and(Predicate::Eq(Value::Int(1))),
            Predicate::False
        );
        // False OR x = x
        assert_eq!(
            Predicate::False.or(Predicate::Eq(Value::Int(1))),
            Predicate::Eq(Value::Int(1))
        );
        // True OR x = True
        assert_eq!(
            Predicate::True.or(Predicate::Eq(Value::Int(1))),
            Predicate::True
        );
        // NOT NOT x = x
        assert_eq!(Predicate::True.not().not(), Predicate::True);
    }

    // ========== ResourceClaim tests ==========

    #[test]
    fn resource_claim_exclusive_conflicts_with_exclusive() {
        let a = ResourceClaim::exclusive("file.txt");
        let b = ResourceClaim::exclusive("file.txt");
        assert!(a.conflicts_with(&b));
    }

    #[test]
    fn resource_claim_exclusive_conflicts_with_shared() {
        let a = ResourceClaim::exclusive("file.txt");
        let b = ResourceClaim::shared("file.txt");
        assert!(a.conflicts_with(&b));
        assert!(b.conflicts_with(&a));
    }

    #[test]
    fn resource_claim_shared_does_not_conflict_with_shared() {
        let a = ResourceClaim::shared("file.txt");
        let b = ResourceClaim::shared("file.txt");
        assert!(!a.conflicts_with(&b));
    }

    #[test]
    fn resource_claim_different_resources_never_conflict() {
        let a = ResourceClaim::exclusive("file1.txt");
        let b = ResourceClaim::exclusive("file2.txt");
        assert!(!a.conflicts_with(&b));
    }

    #[test]
    fn resource_claim_lease_conflicts_like_exclusive() {
        let a = ResourceClaim::lease("db_conn", 5000);
        let b = ResourceClaim::exclusive("db_conn");
        assert!(a.conflicts_with(&b));
    }

    #[test]
    fn resource_claim_budget_does_not_conflict_with_budget() {
        let a = ResourceClaim::budget("api_credits", 100.0);
        let b = ResourceClaim::budget("api_credits", 50.0);
        // Budgets don't conflict at the claim level — scheduler handles limits
        assert!(!a.conflicts_with(&b));
    }

    #[test]
    fn resource_claim_rate_limit_does_not_conflict_with_rate_limit() {
        let a = ResourceClaim::rate_limit("api/v1", 1000, 10);
        let b = ResourceClaim::rate_limit("api/v1", 1000, 10);
        // Rate limits don't conflict at the claim level — scheduler handles limits
        assert!(!a.conflicts_with(&b));
    }

    #[test]
    fn resource_claim_display() {
        let claim = ResourceClaim::exclusive("file.txt");
        assert_eq!(format!("{}", claim), "file.txt[1, exclusive]");

        let lease = ResourceClaim::lease("db", 5000);
        assert_eq!(format!("{}", lease), "db[1, lease(5000ms)]");

        let budget = ResourceClaim::budget("credits", 100.0);
        assert_eq!(format!("{}", budget), "credits[100, budget]");

        let rate = ResourceClaim::rate_limit("api", 1000, 10);
        assert_eq!(format!("{}", rate), "api[1, rate_limit(10/1000 ms)]");
    }

    #[test]
    fn amount_zero_and_one() {
        assert!(Amount::zero().is_zero());
        assert!(!Amount::one().is_zero());
        assert_eq!(Amount::one().0, 1.0);
        assert_eq!(Amount::zero().0, 0.0);
    }
}
