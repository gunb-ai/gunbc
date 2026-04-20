### Layering

**Foundation (`std/`):** Shared facts only. Standards, specifications,
mathematical definitions. No policy, no preference. This is already
strong: `logic → bit → integer → float → string → unicode → filesystem`.

**External dependencies (`extdeps/`):** Spec-grounded models of real
systems. Each type comes from actual API documentation. Shared concepts
across providers (like `Role` in LLM APIs) are valid when both providers
independently document the same concept. Reference the documentation.

**Application layer:** Policy, calibration, team decisions. Legitimate
but clearly separated from factual layers. Deferred until the foundation
is solid.

---

