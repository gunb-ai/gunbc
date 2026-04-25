// Pilot-scope probes. Code in this module is explicitly *not* part of the
// production v3 compiler pipeline — each submodule is a bounded experiment
// validating a design hypothesis, scoped per its worker brief, and either
// promotes into the main compiler (with its own dissolution lane) or
// escalates findings back to its manager.
//
// Add new probes only when a worker brief explicitly authorizes
// pilot-scoped code. Pilot modules must:
//   - be named after the brief (e.g., grounding_pilot)
//   - carry a header pointing to the brief and its escalation path
//   - never modify production substrate or pipeline stages
//   - be deletable as a unit when the probe completes
//
// Active probes:
//   grounding_pilot -- T-Ground-Pilot worker brief; validates that
//     algebra-homomorphism inhabitance search reproduces today's
//     name-keyed table-lookup routing on a 10-element Rust pilot set.

pub mod grounding_pilot;
