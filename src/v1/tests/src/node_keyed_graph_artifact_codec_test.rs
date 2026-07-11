//! Sharing witnesses for the NodeKeyedGraphArtifact codec kernel (the interned
//! content-keyed node table in `resolved_graph_cache.rs`; modeled authority
//! `v2.workflow.realization_runner`). The two agreed witnesses, by execution:
//! decode rebuilds structural sharing (Rc pointer-equal at both parents, with
//! a deliberately unshared tree-decode as the RED control), and reencode is
//! byte-identical. Plus the interning, refusal, and row-facts/size receipts.
//!
//! SEED-RETAINED with its kernel (DESIGN §7): counted by the disposition row
//! `node_keyed_graph_codec_seed_disposition` in `v2.workflow.realization_runner`.
//! These witnesses observe Rc pointer identity — a host-realization fact — so
//! they live in the seed test surface; they migrate with the kernel when its
//! dissolve-on (binary-medium emission rows) fires.

use std::rc::Rc;

use v1_compiler::resolved_graph_cache::{
    node_keyed_graph_decode, node_keyed_graph_encode, node_keyed_graph_row_facts,
    NodeKeyedGraphEncode, NodeKeyedGraphRowFacts,
};
use v1_compiler::v1_rt;

struct FixtureNode {
    label: String,
    children: Vec<Rc<FixtureNode>>,
}

impl NodeKeyedGraphEncode for FixtureNode {
    fn local_payload_bytes(&self) -> Vec<u8> {
        self.label.as_bytes().to_vec()
    }

    fn graph_children(&self) -> Vec<Rc<Self>> {
        self.children.clone()
    }

    fn rebuild(local_payload: &[u8], children: Vec<Rc<Self>>) -> Result<Self, String> {
        Ok(FixtureNode {
            label: String::from_utf8(local_payload.to_vec())
                .map_err(|e| format!("fixture label not utf8: {e}"))?,
            children,
        })
    }
}

/// root -> {a, b}; a -> c; b -> c, with `c` one shared `Rc` at both parents.
fn diamond() -> Rc<FixtureNode> {
    let c = Rc::new(FixtureNode {
        label: "c-shared-leaf".to_string(),
        children: vec![],
    });
    let a = Rc::new(FixtureNode {
        label: "a".to_string(),
        children: vec![c.clone()],
    });
    let b = Rc::new(FixtureNode {
        label: "b-mid".to_string(),
        children: vec![c],
    });
    Rc::new(FixtureNode {
        label: "root-node".to_string(),
        children: vec![a, b],
    })
}

fn store_entry_key(name: &str) -> String {
    v1_rt::atom_identity_hash(name.to_string())
}

fn encode_diamond() -> Vec<u8> {
    node_keyed_graph_encode(&[(store_entry_key("diamond-entry"), diamond())])
        .expect("diamond encodes")
}

/// The RED-control decode: rebuild every child ref as a FRESH `Rc` (a tree
/// decode with no hash-consing) — exactly the un-sharing failure class the
/// codec ruling forbids. The ptr-eq witness must fail against this.
fn decode_unshared_from(rows: &[NodeKeyedGraphRowFacts], root: &str) -> Rc<FixtureNode> {
    let row = rows
        .iter()
        .find(|r| r.content_hash == root)
        .expect("row present");
    let children = row
        .child_refs
        .iter()
        .map(|child| decode_unshared_from(rows, child))
        .collect();
    Rc::new(FixtureNode::rebuild(&row.payload, children).expect("fixture rebuilds"))
}

fn decoded_diamond_root(bytes: &[u8]) -> Rc<FixtureNode> {
    let decoded = node_keyed_graph_decode::<FixtureNode>(bytes).expect("diamond decodes");
    assert_eq!(decoded.entries.len(), 1);
    assert_eq!(decoded.entries[0].0, store_entry_key("diamond-entry"));
    decoded.entries[0].1.clone()
}

#[test]
fn shared_subtree_interned_once() {
    let rows = node_keyed_graph_row_facts(&encode_diamond()).expect("row facts parse");
    // 4 distinct nodes, not the 5 a tree walk would emit (c once, not twice).
    assert_eq!(rows.len(), 4);
}

#[test]
fn decode_rebuilds_sharing_with_unshared_red_control() {
    let bytes = encode_diamond();
    let root = decoded_diamond_root(&bytes);
    let a = &root.children[0];
    let b = &root.children[1];
    assert!(
        Rc::ptr_eq(&a.children[0], &b.children[0]),
        "hash-consed decode must rebuild the shared subtree as ONE Rc"
    );

    let rows = node_keyed_graph_row_facts(&bytes).expect("row facts parse");
    let root_hash = rows
        .last()
        .expect("root row emitted last")
        .content_hash
        .clone();
    let unshared = decode_unshared_from(&rows, &root_hash);
    assert_eq!(unshared.children[0].children[0].label, "c-shared-leaf");
    assert!(
        !Rc::ptr_eq(
            &unshared.children[0].children[0],
            &unshared.children[1].children[0]
        ),
        "RED control: the deliberately unshared tree decode must FAIL the ptr-eq witness \
         (otherwise the witness discriminates nothing)"
    );
}

#[test]
fn reencode_is_byte_identical() {
    let bytes = encode_diamond();
    let root = decoded_diamond_root(&bytes);
    let reencoded = node_keyed_graph_encode(&[(store_entry_key("diamond-entry"), root)])
        .expect("decoded diamond re-encodes");
    assert_eq!(reencoded, bytes);
}

#[test]
fn equal_content_distinct_allocations_intern_to_one_row() {
    // Two SEPARATE Rc allocations with identical content: interning is by
    // content hash, so one row lands — and the decode then shares them.
    let leaf_one = Rc::new(FixtureNode {
        label: "same-content".to_string(),
        children: vec![],
    });
    let leaf_two = Rc::new(FixtureNode {
        label: "same-content".to_string(),
        children: vec![],
    });
    assert!(!Rc::ptr_eq(&leaf_one, &leaf_two));
    let root = Rc::new(FixtureNode {
        label: "root".to_string(),
        children: vec![leaf_one, leaf_two],
    });
    let bytes =
        node_keyed_graph_encode(&[(store_entry_key("interned-entry"), root)]).expect("encodes");
    let rows = node_keyed_graph_row_facts(&bytes).expect("row facts parse");
    assert_eq!(rows.len(), 2);
    let decoded = node_keyed_graph_decode::<FixtureNode>(&bytes).expect("decodes");
    let decoded_root = &decoded.entries[0].1;
    assert!(Rc::ptr_eq(
        &decoded_root.children[0],
        &decoded_root.children[1]
    ));
}

#[test]
fn row_local_lengths_sum_matches_total_fold_semantics() {
    let rows = node_keyed_graph_row_facts(&encode_diamond()).expect("row facts parse");
    let total: usize = rows.iter().map(|r| r.payload.len()).sum();
    // Sum of row-local encoded lengths, each interned row exactly once — the
    // same reading the .dag total/transitive folds pin on the diamond fixture.
    let expected = "c-shared-leaf".len() + "a".len() + "b-mid".len() + "root-node".len();
    assert_eq!(total, expected);
}

fn decode_err(bytes: &[u8]) -> String {
    match node_keyed_graph_decode::<FixtureNode>(bytes) {
        Err(e) => e,
        Ok(_) => panic!("decode unexpectedly succeeded"),
    }
}

#[test]
fn corrupt_and_truncated_artifacts_refuse() {
    let bytes = encode_diamond();

    let mut bad_magic = bytes.clone();
    bad_magic[0] ^= 0xFF;
    assert!(decode_err(&bad_magic).contains("bad magic"));

    let truncated = &bytes[..bytes.len() - 3];
    assert!(decode_err(truncated).contains("truncated"));

    // Flip one payload byte: the stored row hash no longer matches the
    // recomputed content hash — content verified on read, refuse not serve.
    let rows = node_keyed_graph_row_facts(&bytes).expect("row facts parse");
    let first_payload = rows[0].payload.clone();
    let payload_pos = bytes
        .windows(first_payload.len())
        .position(|w| w == first_payload.as_slice())
        .expect("payload bytes located");
    let mut corrupted = bytes.clone();
    corrupted[payload_pos] ^= 0xFF;
    assert!(decode_err(&corrupted).contains("content hash mismatch"));
}

#[test]
fn non_digest_store_key_refuses_at_encode() {
    let err = node_keyed_graph_encode(&[("not-a-hash".to_string(), diamond())]).unwrap_err();
    assert!(err.contains("not a 16-char hex hash"));
}
