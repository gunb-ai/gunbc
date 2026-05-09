# R3 Novel-Finding Worker Brief — F4 `PartitionResult` bypassed by anonymous return type

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068) lane scope.
**Authority parent**: gpt-5-5-pro reflective analysis Finding 4; PM dispatch at gunbc#846 c#4413701937.
**Priority**: LOW-MEDIUM — Class G small-scope cleanup; thesis-faithfulness doc/consumer drift.

---

## §0. Problem statement

`dsl/std/filesystem.dag:51` declares:
```
type PartitionResult { ... }
```

But the consumer at `:83` uses an anonymous return record:
```
fn partition_entries(entries: List<FileEntry>) -> { readable: List<FileEntry>, skipped: List<FileEntry> } { ... }
```

The named type `PartitionResult` exists but is bypassed — the anonymous return shape duplicates the declared type's structural shape without naming it. Class G small duplicate-authority cleanup.

## §1. Required outcome

`partition_entries` returns `PartitionResult` directly; anonymous duplication eliminated.

## §2. Fix options

**Option A**: Update `partition_entries` signature to `-> PartitionResult`; verify consumers don't break; delete anonymous shape duplication.

(Single option; trivial scope.)

## §3. Files

- `dsl/std/filesystem.dag` (signature change)
- consumers of `partition_entries` (typecheck)

## §4. Cross-cutting constraints

- Verify shape exact-match between `PartitionResult` declaration and current anonymous return before substituting.
- Cross-references Class G row 14 in sweep doc.

## §5. Receipt

- `partition_entries` returns `PartitionResult` named type.
- Anonymous return type deleted.
- Consumers verified or updated.
- Sweep-doc Class G row 14 updated.

---

**End of brief.**
