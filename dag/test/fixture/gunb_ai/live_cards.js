/* ════════════════════════════════════════════════════════════════════
   gunb.ai · story-deck renderer
   ────────────────────────────────────────────────────────────────────
   One Users system. Three isolated views. Same underlying source
   model; each card renders only the slice it argues from.

     01  projection closure       — one system, four target artifacts
     02  affected-set propagation — Timestamp edit ripples through
     03  complexity budget        — O(n) declared, O(n²) observed,
                                    suggested replacement shape

   Each card is independent visually (no accreting layers). Card 03
   absorbs the diagnostic — the suggested fix is just another row of
   its receipt, not a separate panel.

   HTML owns text. SVG owns structure (and small node labels for
   identifiability). Cards are validated for ref→graph correspondence,
   role consistency, no line-number semantics, and shared systemId.

   Navigation: prev/next buttons or ← / → keys.
   No build step. Plain script tag, IIFE, vanilla DOM.
   ══════════════════════════════════════════════════════════════════ */

(function () {
  "use strict";

  /* ── Roles → CSS class ──────────────────────────────────────── */
  const Roles = {
    stable:     "map-stable",
    focus:      "map-focus",
    derived:    "map-derived",
    transitive: "map-transitive",
    artifact:   "map-artifact",
    removed:    "map-removed",
    boundary:   "map-boundary",
    context:    "map-context",
    skipped:    "map-skipped"
  };
  const roleClass = r => Roles[r] || Roles.context;

  /* ── Code spec helpers ─────────────────────────────────────── */
  const tx  = v => ({ kind: "text", value: v });
  const kw  = v => ({ kind: "kw",   value: v });
  const ty  = v => ({ kind: "ty",   value: v });
  const com = v => ({ kind: "com",  value: v });
  const lit = v => ({ kind: "lit",  value: v });
  const ref = (id, text, role) => ({ kind: "ref", id, text: text || id, role: role || null });
  const mark = (text, role) => ({ kind: "mark", text, role });
  const ln       = (n, parts) => ({ n, parts });
  const blank    = (n)        => ({ n, parts: [] });
  const diffRm   = (n, parts) => ({ n, parts, diff: "rm" });
  const diffAdd  = (n, parts) => ({ n, parts, diff: "add" });

  /* ── Graph spec helpers ────────────────────────────────────── */
  const node = (id, label, col, row, role) =>
    ({ kind: "node", id, label, col, row, role: role || "context" });
  const artifact = (id, label, col, row) =>
    ({ kind: "artifact", id, label, col, row, role: "artifact", generated: true });
  const edge = (from, to, label, role) =>
    ({ from, to, label: label || "", role: role || "derived" });

  /* ── Layout (column grid with demand-driven gaps) ──────────── */
  const COL_GAP_MIN   = 56;
  const Y_GAP         = 46;
  const BASE_X        = 18;
  const BASE_Y        = 22;
  const NODE_H        = 28;
  const PAD           = 18;
  const ELBOW_OFFSET  = 18;
  const LABEL_FONT_PX = 5;
  const LABEL_PAD     = 8;

  function labelWidth(label) { return Math.max(60, label.length * 7.2 + 22); }
  function edgeLabelWidth(label) { return label ? label.length * LABEL_FONT_PX + 4 : 0; }
  function columnXs(nodes, edges) {
    const widest = {};
    for (const n of nodes) {
      const w = labelWidth(n.label);
      if (!(n.col in widest) || w > widest[n.col]) widest[n.col] = w;
    }
    const byId = {};
    for (const n of nodes) byId[n.id] = n;
    const gapDemand = {};
    for (const e of edges) {
      if (!e.label) continue;
      const a = byId[e.from];
      const b = byId[e.to];
      if (!a || !b || a.col === b.col) continue;
      const startCol = Math.min(a.col, b.col);
      const need = ELBOW_OFFSET + edgeLabelWidth(e.label) + LABEL_PAD * 2;
      if (!(startCol in gapDemand) || need > gapDemand[startCol]) gapDemand[startCol] = need;
    }
    const cols = Object.keys(widest).map(Number).sort((a, b) => a - b);
    const x = {};
    let cx = BASE_X;
    for (let i = 0; i < cols.length; i++) {
      const c = cols[i];
      x[c] = cx;
      cx += widest[c] + Math.max(COL_GAP_MIN, gapDemand[c] || 0);
    }
    return x;
  }
  function layoutNodes(nodes, edges) {
    const x = columnXs(nodes, edges);
    return nodes.map(n => Object.assign({}, n, {
      x: x[n.col],
      y: BASE_Y + n.row * Y_GAP,
      w: labelWidth(n.label),
      h: NODE_H
    }));
  }
  function graphBounds(nodes) {
    const maxX = Math.max.apply(null, nodes.map(n => n.x + n.w));
    const maxY = Math.max.apply(null, nodes.map(n => n.y + n.h));
    return { width: maxX + PAD, height: maxY + PAD };
  }
  function edgeGeom(a, b) {
    const sx = a.x + a.w;
    const sy = a.y + a.h / 2;
    const tx = b.x;
    const ty = b.y + b.h / 2;
    const sameRow = Math.abs(sy - ty) < 1;
    const baseMid = sx + ELBOW_OFFSET;
    const mid = Math.min(baseMid, tx - 8);
    return { sx, sy, tx, ty, mid, sameRow };
  }
  function edgePath(a, b) {
    const g = edgeGeom(a, b);
    if (g.sameRow) return "M " + g.sx + " " + g.sy + " L " + g.tx + " " + g.ty;
    return "M " + g.sx + " " + g.sy + " H " + g.mid + " V " + g.ty + " H " + g.tx;
  }

  /* ── Escape ────────────────────────────────────────────────── */
  function esc(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  /* ════════════════════════════════════════════════════════════════
     VALIDATORS — fail-closed
     ══════════════════════════════════════════════════════════════ */

  const LINE_NUM_RE = /\bL\d+\b/;

  // Card-shape-aware accessors. A lane-tree card carries its graph
  // structure in card.lanes (cells × columns); a relation card uses
  // card.graph (nodes + edges); a cost-area card has neither.
  function cardCells(card) {
    if (card.shape === "lane-tree") {
      const out = [];
      for (const row of card.lanes.rows) {
        for (const cell of row.cells) {
          if (cell) out.push(cell);
        }
      }
      return out;
    }
    if (card.graph) return card.graph.nodes;
    return [];
  }
  function cardNodeIds(card) {
    return new Set(cardCells(card).map(c => c.id));
  }

  function validateRefsAnchored(card) {
    const ids = cardNodeIds(card);
    for (const line of card.code) {
      for (const p of line.parts) {
        if (p.kind === "ref" && !ids.has(p.id)) {
          throw new Error(card.id + ": code ref '" + p.id + "' has no graph node");
        }
      }
    }
    if (card.graph) {
      for (const e of card.graph.edges) {
        if (!ids.has(e.from)) throw new Error(card.id + ": edge from '" + e.from + "' missing");
        if (!ids.has(e.to))   throw new Error(card.id + ": edge to '"   + e.to   + "' missing");
      }
    }
  }
  function validateRoleConsistency(card) {
    const byId = {};
    for (const c of cardCells(card)) byId[c.id] = c;
    for (const line of card.code) {
      for (const p of line.parts) {
        if (p.kind !== "ref" || !p.role) continue;
        const g = byId[p.id];
        if (g && g.role && p.role !== g.role) {
          throw new Error(card.id + ": role mismatch for '" + p.id +
                          "': code=" + p.role + ", graph=" + g.role);
        }
      }
    }
  }
  function validateNoUnanchoredActiveNodes(card) {
    const codeRefs = new Set();
    for (const line of card.code) {
      for (const p of line.parts) if (p.kind === "ref") codeRefs.add(p.id);
    }
    for (const c of cardCells(card)) {
      if (c.generated || c.kind === "artifact" || c.role === "context") continue;
      if (!codeRefs.has(c.id)) {
        throw new Error(card.id + ": active graph node '" + c.id + "' has no code anchor");
      }
    }
  }
  function validateNoLineNumbersInGraph(card) {
    for (const c of cardCells(card)) {
      const label = c.label || c.id;
      if (LINE_NUM_RE.test(label)) throw new Error(card.id + ": graph label '" + label + "' uses L## semantics");
    }
    if (card.graph) {
      for (const e of card.graph.edges) {
        if (e.label && LINE_NUM_RE.test(e.label)) throw new Error(card.id + ": edge label '" + e.label + "' uses L## semantics");
      }
    }
    if (card.shape === "lane-tree") {
      for (const row of card.lanes.rows) {
        for (const cell of row.cells) {
          if (cell && cell.via && LINE_NUM_RE.test(cell.via)) {
            throw new Error(card.id + ": lane via label uses L## semantics");
          }
        }
      }
    }
  }
  function validateReceiptNoLineNumbers(card) {
    for (const row of card.receipt) {
      const txt = typeof row === "string" ? row : (row.value || "") + (row.code ? (Array.isArray(row.code) ? row.code.join(" ") : row.code) : "");
      if (LINE_NUM_RE.test(txt)) throw new Error(card.id + ": receipt uses L## shorthand");
    }
  }
  function validateCard(card) {
    validateRefsAnchored(card);
    validateRoleConsistency(card);
    validateNoUnanchoredActiveNodes(card);
    validateNoLineNumbersInGraph(card);
    validateReceiptNoLineNumbers(card);
  }
  function validateContinuity(cards) {
    if (!cards.length) return;
    const base = cards[0].systemId;
    for (const c of cards) {
      if (c.systemId !== base && !c.standalone) {
        throw new Error(c.id + ": breaks continuity; expected systemId='" + base + "'");
      }
    }
  }

  /* ════════════════════════════════════════════════════════════════
     CODE RENDERER
     ══════════════════════════════════════════════════════════════ */

  function partHtml(p, refRole) {
    if (p.kind === "text") return esc(p.value);
    if (p.kind === "kw")   return '<span class="ck-kw">'  + esc(p.value) + '</span>';
    if (p.kind === "ty")   return '<span class="ck-ty">'  + esc(p.value) + '</span>';
    if (p.kind === "lit")  return '<span class="ck-lit">' + esc(p.value) + '</span>';
    if (p.kind === "com")  return '<span class="ck-com">' + esc(p.value) + '</span>';
    if (p.kind === "mark") return '<span class="' + roleClass(p.role) + '">' + esc(p.text) + '</span>';
    if (p.kind === "ref") {
      const role = p.role || refRole(p.id);
      const cls  = role ? roleClass(role) : "";
      return '<span class="ck-ref ' + cls + '" data-id="' + esc(p.id) + '">' + esc(p.text) + '</span>';
    }
    return "";
  }
  function lineHtml(line, refRole) {
    const lnNo = line.n != null
      ? '<span class="ck-ln">' + String(line.n).padStart(2, "0") + '</span> '
      : '<span class="ck-ln">  </span> ';
    const parts = line.parts.map(p => partHtml(p, refRole)).join("");
    if (line.diff === "rm")  return '<div class="ck-line ck-diff-rm">'  + lnNo + '<span class="ck-diff-sign">-</span> ' + parts + '</div>';
    if (line.diff === "add") return '<div class="ck-line ck-diff-add">' + lnNo + '<span class="ck-diff-sign">+</span> ' + parts + '</div>';
    return '<div class="ck-line">' + lnNo + '  ' + (parts || '&nbsp;') + '</div>';
  }
  function renderCode(card) {
    const roleById = {};
    for (const c of cardCells(card)) roleById[c.id] = c.role;
    const refRole = id => roleById[id];
    const head = '<div class="card-code-head">'
               + '<span class="ck-dot">●</span> '
               + '<span class="ck-file">' + esc(card.codeFile || (card.id + ".dag")) + '</span>'
               + '</div>';
    const body = '<div class="card-code-body">'
               + card.code.map(l => lineHtml(l, refRole)).join("")
               + '</div>';
    return '<div class="card-code">' + head + body + '</div>';
  }

  /* ════════════════════════════════════════════════════════════════
     GRAPH RENDERERS — relation graph (cards 01, 02) and cost-area (03)
     ══════════════════════════════════════════════════════════════ */

  function renderRelationGraph(card) {
    const nodes = layoutNodes(card.graph.nodes, card.graph.edges);
    const byId  = {};
    for (const n of nodes) byId[n.id] = n;
    const bounds = graphBounds(nodes);

    // Two-axis edge staggering so multi-incoming convergence and
    // multi-outgoing fan-outs don't pile arrows on top of each other:
    //   - incoming offset: when N arrows converge on one target, they
    //     enter the left edge at different y points so their final
    //     horizontal segments don't superimpose.
    //   - outgoing elbow fan: when N arrows leave one source, they
    //     turn at slightly different elbow x columns so the verticals
    //     don't share a single bus line.
    const incomingCount = {};
    const outgoingCount = {};
    for (const e of card.graph.edges) {
      incomingCount[e.to] = (incomingCount[e.to] || 0) + 1;
      outgoingCount[e.from] = (outgoingCount[e.from] || 0) + 1;
    }
    const incomingIdx = new Map();
    const outgoingIdx = new Map();
    {
      const inSeen = {};
      const outSeen = {};
      for (const e of card.graph.edges) {
        inSeen[e.to]    = inSeen[e.to]    || 0;
        outSeen[e.from] = outSeen[e.from] || 0;
        incomingIdx.set(e, inSeen[e.to]++);
        outgoingIdx.set(e, outSeen[e.from]++);
      }
    }
    function tyFor(e) {
      const b = byId[e.to];
      const total = incomingCount[e.to];
      if (total <= 1) return b.y + b.h / 2;
      const spread = Math.min(b.h * 0.55, 12);
      const step = spread / (total - 1);
      return b.y + b.h / 2 - spread / 2 + incomingIdx.get(e) * step;
    }
    function midFor(e, sx, tx) {
      const total = outgoingCount[e.from];
      let baseMid = sx + ELBOW_OFFSET;
      if (total > 1) {
        // Each outgoing edge gets its own elbow column. Step is small
        // (4 px) so the fan still reads as siblings of one source
        // rather than independent arrows.
        const step = 5;
        baseMid = sx + ELBOW_OFFSET + outgoingIdx.get(e) * step;
      }
      return Math.min(baseMid, tx - 8);
    }
    function geomFor(e) {
      const a = byId[e.from];
      const b = byId[e.to];
      const sx = a.x + a.w;
      const sy = a.y + a.h / 2;
      const tx = b.x;
      const ty = tyFor(e);
      const sameRow = Math.abs(sy - ty) < 1;
      const mid = midFor(e, sx, tx);
      return { sx, sy, tx, ty, mid, sameRow };
    }
    function pathFor(e) {
      const g = geomFor(e);
      if (g.sameRow) return "M " + g.sx + " " + g.sy + " L " + g.tx + " " + g.ty;
      return "M " + g.sx + " " + g.sy + " H " + g.mid + " V " + g.ty + " H " + g.tx;
    }

    const edgesSvg = card.graph.edges.map(e => {
      const cls = roleClass(e.role);
      return '<path class="graph-edge ' + cls + '" d="' + pathFor(e)
           + '" marker-end="url(#card-arrow)" />';
    }).join("");

    const edgeLabels = card.graph.edges.filter(e => e.label).map(e => {
      const g = geomFor(e);
      const lx = g.sameRow ? (g.sx + g.tx) / 2 : (g.mid + g.tx) / 2;
      const ly = (g.sameRow ? g.sy : g.ty) - 4;
      const cls = roleClass(e.role);
      return '<text class="graph-edge-label ' + cls + '" x="' + lx
           + '" y="' + ly + '" text-anchor="middle">' + esc(e.label) + '</text>';
    }).join("");

    const nodesSvg = nodes.map(n => {
      const cls = roleClass(n.role);
      if (n.kind === "artifact") {
        const cut = 8;
        const d = "M " + n.x + " " + n.y
                + " H " + (n.x + n.w - cut)
                + " L " + (n.x + n.w) + " " + (n.y + cut)
                + " V " + (n.y + n.h)
                + " H " + n.x + " Z";
        return '<g class="graph-node ' + cls + '">'
             + '<path d="' + d + '" />'
             + '<text x="' + (n.x + n.w / 2) + '" y="' + (n.y + n.h / 2 + 4)
             + '" text-anchor="middle">' + esc(n.label) + '</text></g>';
      }
      return '<g class="graph-node ' + cls + '">'
           + '<rect x="' + n.x + '" y="' + n.y + '" width="' + n.w
           + '" height="' + n.h + '" rx="4" />'
           + '<text x="' + (n.x + n.w / 2) + '" y="' + (n.y + n.h / 2 + 4)
           + '" text-anchor="middle">' + esc(n.label) + '</text></g>';
    }).join("");

    const defs = '<defs>'
               + '<marker id="card-arrow" viewBox="0 0 10 10" refX="9" refY="5"'
               + ' markerWidth="6" markerHeight="6" orient="auto">'
               + '<path d="M0,0 L10,5 L0,10 Z" fill="context-stroke"/>'
               + '</marker></defs>';

    return '<div class="card-graph">'
         + '<svg viewBox="0 0 ' + bounds.width + ' ' + bounds.height
         + '" preserveAspectRatio="xMidYMid meet" aria-label="' + esc(card.name || "") + '">'
         + defs + edgesSvg + edgeLabels + nodesSvg
         + '</svg></div>';
  }

  // Cost-area: two panels bottom-aligned. Each cell is colored per-row:
  // the bottom `budgetRows` rows (counting up from the bottom) are
  // budget (moss/green); rows above are overage (clay/red). For the
  // declared panel, the whole panel is budget (1 row). For observed,
  // the bottom row is budget and the rest is overage — the visual
  // claim "observed extends UP past the budget ceiling."
  function renderCostArea(card) {
    const CELL = 11;
    const GAP  = 2;
    const PANEL_GAP = 56;
    const LABEL_H   = 26;   // room for label + sublabel below each panel
    const PAD       = 14;
    function gridDims(p) {
      return {
        w: p.cols * CELL + (p.cols - 1) * GAP,
        h: p.rows * CELL + (p.rows - 1) * GAP
      };
    }
    const panels = card.panels.map(p => Object.assign({}, p, gridDims(p)));
    const totalW = panels.reduce((s, p) => s + p.w, 0) + (panels.length - 1) * PANEL_GAP;
    const maxH   = Math.max.apply(null, panels.map(p => p.h));
    const W = totalW + PAD * 2;
    const H = maxH + LABEL_H + PAD * 2;

    const bottomY = PAD + maxH;   // shared bottom edge of all panel grids

    let cx = PAD;
    const panelsSvg = panels.map(p => {
      const px = cx;
      const py = bottomY - p.h;   // bottom-align
      cx += p.w + PANEL_GAP;

      let cells = "";
      const budgetRows = p.budgetRows || 0;
      for (let r = 0; r < p.rows; r++) {
        const fromBottom = p.rows - 1 - r;
        const cellRole = fromBottom < budgetRows ? "budget" : "overage";
        for (let c = 0; c < p.cols; c++) {
          const x = px + c * (CELL + GAP);
          const y = py + r * (CELL + GAP);
          cells += '<rect class="cost-cell-' + cellRole + '" x="' + x
                 + '" y="' + y + '" width="' + CELL + '" height="' + CELL + '" />';
        }
      }

      const labelX = px + p.w / 2;
      const labelY    = bottomY + 12;
      const sublabelY = bottomY + 22;
      return '<g class="cost-area">'
           + cells
           + '<text class="cost-area-label" x="' + labelX + '" y="' + labelY
           + '" text-anchor="middle">' + esc(p.label) + '</text>'
           + (p.sublabel
             ? '<text class="cost-area-sublabel" x="' + labelX + '" y="' + sublabelY
                + '" text-anchor="middle">' + esc(p.sublabel) + '</text>'
             : '')
           + '</g>';
    }).join("");

    return '<div class="card-graph">'
         + '<svg viewBox="0 0 ' + W + ' ' + H
         + '" preserveAspectRatio="xMidYMid meet" aria-label="' + esc(card.name || "") + '">'
         + panelsSvg
         + '</svg></div>';
  }

  /* Lane-tree: a story renderer for affected-set cards.
     Fixed columns × fixed rows. Edges only connect adjacent columns
     within the same row, so the diagram CANNOT cross — overlap is
     structurally impossible, not "prevented by elbow staggering."

     Spec shape:
       card.shape = "lane-tree"
       card.lanes = {
         columns: ["fact", "field", "operation", "artifact"],
         rows: [{ id, cells: [cell | null, ...] }, ...]
       }
       cell = { id, label, role, kind?, via? }
         - via labels the OUTGOING edge from this cell to the next
         - kind: "artifact" renders the cut-corner pentagon */
  function renderLaneTree(card) {
    const NODE_H = 28;
    const ROW_GAP = 50;
    const COL_GAP_MIN = 36;
    const BASE_Y = 22;
    const BASE_X = 18;
    const PAD = 18;

    const lanes = card.lanes;
    const cols = lanes.columns;
    const rows = lanes.rows;

    // Per-column width = max cell label width across rows
    const colWidth = cols.map((_, ci) => {
      let max = 60;
      for (const row of rows) {
        const cell = row.cells[ci];
        if (!cell) continue;
        const w = labelWidth(cell.label || cell.id);
        if (w > max) max = w;
      }
      return max;
    });

    // Per-gap demand: max via label crossing this boundary + clearance
    const gapDemand = [];
    for (let ci = 0; ci < cols.length - 1; ci++) {
      let need = 0;
      for (const row of rows) {
        const left = row.cells[ci];
        if (!left || !left.via) continue;
        const next = row.cells[ci + 1];
        if (!next) continue;
        const w = edgeLabelWidth(left.via) + 20;
        if (w > need) need = w;
      }
      gapDemand.push(Math.max(COL_GAP_MIN, need));
    }

    // Column x positions
    const colX = [];
    let cx = BASE_X;
    for (let i = 0; i < cols.length; i++) {
      colX.push(cx);
      cx += colWidth[i];
      if (i < cols.length - 1) cx += gapDemand[i];
    }
    const totalW = cx + PAD;
    const totalH = BASE_Y + rows.length * ROW_GAP + PAD;

    let nodesHtml = "";
    let edgesHtml = "";
    let labelsHtml = "";

    for (let ri = 0; ri < rows.length; ri++) {
      const row = rows[ri];
      const y = BASE_Y + ri * ROW_GAP;
      const cy = y + NODE_H / 2;

      let prevX = null;
      let prevW = null;
      let prevCell = null;

      for (let ci = 0; ci < row.cells.length; ci++) {
        const cell = row.cells[ci];
        if (!cell) { prevX = null; prevCell = null; continue; }

        const xx = colX[ci];
        const ww = colWidth[ci];
        const cls = roleClass(cell.role);

        // Render box (rect or cut-corner artifact)
        if (cell.kind === "artifact") {
          const cut = 8;
          const d = "M " + xx + " " + y
                  + " H " + (xx + ww - cut)
                  + " L " + (xx + ww) + " " + (y + cut)
                  + " V " + (y + NODE_H)
                  + " H " + xx + " Z";
          nodesHtml += '<g class="graph-node ' + cls + '">'
                    + '<path d="' + d + '" />'
                    + '<line x1="' + (xx + ww - cut) + '" y1="' + y
                    + '" x2="' + (xx + ww - cut) + '" y2="' + (y + cut) + '" />'
                    + '<line x1="' + (xx + ww - cut) + '" y1="' + (y + cut)
                    + '" x2="' + (xx + ww) + '" y2="' + (y + cut) + '" />'
                    + '<text x="' + (xx + ww / 2) + '" y="' + (cy + 4)
                    + '" text-anchor="middle">' + esc(cell.label || cell.id) + '</text>'
                    + '</g>';
        } else {
          nodesHtml += '<g class="graph-node ' + cls + '">'
                    + '<rect x="' + xx + '" y="' + y + '" width="' + ww
                    + '" height="' + NODE_H + '" rx="4" />'
                    + '<text x="' + (xx + ww / 2) + '" y="' + (cy + 4)
                    + '" text-anchor="middle">' + esc(cell.label || cell.id) + '</text>'
                    + '</g>';
        }

        // Edge from prevCell to this cell (straight horizontal — same row).
        // Pull endpoint back from target's left edge so the arrowhead has
        // visual separation from the box stroke. Without this gap the arrow
        // tip merges into the target's stroke and reads as a missing arrow.
        if (prevX !== null) {
          const ex1 = prevX + prevW;
          const ex2 = xx - 3;
          // Edge color follows the TARGET's role (the downstream node)
          edgesHtml += '<line class="graph-edge ' + cls + '" x1="' + ex1
                    + '" y1="' + cy + '" x2="' + ex2 + '" y2="' + cy
                    + '" marker-end="url(#card-arrow)" />';

          // Via label sits on the edge, above the line.
          // The via lives on the PREV cell (it annotates the outgoing edge).
          if (prevCell && prevCell.via) {
            const midX = (ex1 + ex2) / 2;
            labelsHtml += '<text class="graph-edge-label ' + cls + '" x="' + midX
                       + '" y="' + (cy - 6) + '" text-anchor="middle">'
                       + esc(prevCell.via) + '</text>';
          }
        }

        prevX = xx;
        prevW = ww;
        prevCell = cell;
      }
    }

    const defs = '<defs>'
               + '<marker id="card-arrow" viewBox="0 0 10 10" refX="9" refY="5"'
               + ' markerWidth="6" markerHeight="6" orient="auto">'
               + '<path d="M0,0 L10,5 L0,10 Z" fill="context-stroke"/>'
               + '</marker></defs>';

    return '<div class="card-graph">'
         + '<svg viewBox="0 0 ' + totalW + ' ' + totalH
         + '" preserveAspectRatio="xMidYMid meet" aria-label="' + esc(card.name || "") + '">'
         + defs + edgesHtml + labelsHtml + nodesHtml
         + '</svg></div>';
  }

  function renderGraph(card) {
    if (card.shape === "cost-area") return renderCostArea(card);
    if (card.shape === "lane-tree") return renderLaneTree(card);
    return renderRelationGraph(card);
  }

  /* ════════════════════════════════════════════════════════════════
     RECEIPT — structured array of {label, value} or {label, code}
     ══════════════════════════════════════════════════════════════ */

  function receiptValueHtml(text) {
    return String(text).replace(/\{(\w+):([^}]*)\}/g, function (_, role, t) {
      return '<span class="' + roleClass(role) + '">' + esc(t) + '</span>';
    });
  }
  function renderReceiptRow(row) {
    if (typeof row === "string") {
      // free-form line (fallback)
      return '<div class="card-receipt-row"><div class="rv">' + receiptValueHtml(row) + '</div></div>';
    }
    let html = '<div class="card-receipt-row">';
    html += '<div class="rk">' + esc(row.label) + '</div>';
    if (row.code) {
      const code = Array.isArray(row.code) ? row.code.join("\n") : row.code;
      html += '<pre class="card-receipt-code">' + esc(code) + '</pre>';
    } else {
      html += '<div class="rv">' + receiptValueHtml(row.value || "") + '</div>';
    }
    html += '</div>';
    return html;
  }
  function renderReceipt(card) {
    if (!card.receipt || !card.receipt.length) return "";
    return '<div class="card-receipt">'
         + card.receipt.map(renderReceiptRow).join("")
         + '</div>';
  }

  /* ════════════════════════════════════════════════════════════════
     CARD + DECK RENDERERS
     ══════════════════════════════════════════════════════════════ */

  function renderCard(card) {
    validateCard(card);
    return '<article class="deck-card" data-card-id="' + esc(card.id) + '">'
         + '<div class="deck-card-head">'
         + '<span class="deck-card-num">'  + esc(card.num)  + '</span>'
         + '<span class="deck-card-name">' + esc(card.name) + '</span>'
         + '</div>'
         + '<div class="deck-card-body">'
         + renderCode(card)
         + renderGraph(card)
         + '</div>'
         + renderReceipt(card)
         + '</article>';
  }

  function renderDeck(deck) {
    validateContinuity(deck.cards);
    const ticks = deck.cards.map(() => '<span class="deck-tick"></span>').join("");
    return '<section class="deck" tabindex="0" aria-label="' + esc(deck.name || "") + '">'
         + '<div class="deck-head">'
         + '<span class="deck-name">' + esc(deck.name || "") + '</span>'
         + '<div class="deck-ticks">' + ticks + '</div>'
         + '<span class="deck-pos">'
         + '<span class="deck-current">01</span> / '
         + '<span class="deck-total">' + String(deck.cards.length).padStart(2, "0") + '</span>'
         + '</span></div>'
         + '<div class="deck-stage"><div class="card-track">'
         + deck.cards.map(renderCard).join("")
         + '</div></div>'
         + '<div class="deck-nav">'
         + '<button class="deck-btn" type="button" data-dir="prev">← prev</button>'
         + '<span style="flex:1"></span>'
         + '<button class="deck-btn" type="button" data-dir="next">next →</button>'
         + '</div></section>';
  }

  /* ── Navigation ────────────────────────────────────────────── */
  function initDeck(deck) {
    const track = deck.querySelector(".card-track");
    const cards = deck.querySelectorAll(".deck-card");
    const ticks = deck.querySelectorAll(".deck-tick");
    const current = deck.querySelector(".deck-current");
    const prev = deck.querySelector('[data-dir="prev"]');
    const next = deck.querySelector('[data-dir="next"]');
    let idx = 0;
    function go(newIdx) {
      idx = Math.max(0, Math.min(cards.length - 1, newIdx));
      track.style.transform = "translateX(-" + (idx * 100) + "%)";
      ticks.forEach((t, i) => t.classList.toggle("active", i <= idx));
      if (current) current.textContent = String(idx + 1).padStart(2, "0");
      if (prev) prev.disabled = idx === 0;
      if (next) next.disabled = idx === cards.length - 1;
    }
    if (prev) prev.addEventListener("click", () => go(idx - 1));
    if (next) next.addEventListener("click", () => go(idx + 1));
    deck.addEventListener("keydown", e => {
      if (e.key === "ArrowLeft")  { go(idx - 1); e.preventDefault(); }
      if (e.key === "ArrowRight") { go(idx + 1); e.preventDefault(); }
    });
    go(0);
  }

  /* ════════════════════════════════════════════════════════════════
     USERS_SYSTEM — the shared source model. All cards derive from this.
     ══════════════════════════════════════════════════════════════ */

  const USERS_SYSTEM = {
    id: "users-system",
    // One service with three operations, each using a different
    // field of User. This is the "single tree" structure the
    // operator asked for: the three coloring categories (direct,
    // transitive, unrelated) are siblings under the same root.
    // The three operations project into three DIFFERENT backends —
    // a Rust library, a TypeScript library, and a SQL schema — to
    // show that one description emits a polyglot artifact set with
    // zero hand-written translation between them.
    artifacts: {
      profile_rs: { label: "profile.rs",  from: "UsersProfile" },
      email_rs:   { label: "email.ts",    from: "UsersEmail"   },
      token_rs:   { label: "token.sql",   from: "UsersToken"   }
    }
  };

  // Layout for card 01 (projection view) — simple User-rooted tree.
  // Card 01 is "one description · three operations · three artifacts":
  // teach the SHAPE of the system. format_joined and field-level
  // dependencies belong to card 02's lane-tree view, not here.
  //
  //   User → Users.profile → profile.rs  (Rust library)
  //        → Users.email   → email.ts    (TypeScript library)
  //        → Users.token   → token.sql   (SQL schema)
  function projectionLayoutNodes(system) {
    return [
      node("User",         "User",          0, 1),
      node("UsersProfile", "Users.profile", 1, 0),
      node("UsersEmail",   "Users.email",   1, 1),
      node("UsersToken",   "Users.token",   1, 2),
      artifact("profile_rs", system.artifacts.profile_rs.label, 2, 0),
      artifact("email_rs",   system.artifacts.email_rs.label,   2, 1),
      artifact("token_rs",   system.artifacts.token_rs.label,   2, 2)
    ];
  }
  function projectionLayoutEdges() {
    return [
      edge("User", "UsersProfile", "profile"),
      edge("User", "UsersEmail",   "email"),
      edge("User", "UsersToken",   "token"),
      edge("UsersProfile", "profile_rs", ""),
      edge("UsersEmail",   "email_rs",   ""),
      edge("UsersToken",   "token_rs",   "")
    ];
  }
  function applyNodeRoles(nodes, roleMap) {
    return nodes.map(n => {
      if (n.kind === "artifact") return n;
      return Object.assign({}, n, { role: roleMap[n.id] || n.role || "context" });
    });
  }
  function applyEdgeRoles(edges, roleMap, defaultRole) {
    return edges.map(e => {
      const key = e.from + "→" + e.to;
      return Object.assign({}, e, { role: roleMap[key] || defaultRole || "derived" });
    });
  }

  /* ════════════════════════════════════════════════════════════════
     CARD 01 · projection closure
     ══════════════════════════════════════════════════════════════ */

  function projectionClosureCard(system) {
    return {
      id: "card-01",
      num: "01",
      name: "one description · three operations · three artifacts",
      systemId: system.id,
      codeFile: "examples/users.dag",
      code: [
        ln(1,  [kw("type"), tx(" "), ref("User", "User", "stable"), tx(" {")]),
        ln(2,  [tx("  id: String")]),
        ln(3,  [tx("  email: String")]),
        ln(4,  [tx("  name: String")]),
        ln(5,  [tx("  created_at: "), ty("Timestamp")]),
        ln(6,  [tx("}")]),
        blank(7),
        ln(8,  [kw("type"), tx(" Profile {")]),
        ln(9,  [tx("  name: String")]),
        ln(10, [tx("  joined: String")]),
        ln(11, [tx("}")]),
        blank(12),
        ln(13, [kw("service"), tx(" Users {")]),
        ln(14, [tx("  "), kw("operation"), tx(" "), ref("UsersProfile", "Profile", "derived"), tx(" {")]),
        ln(15, [tx("    input  { id: String }")]),
        ln(16, [tx("    output { profile: Profile }")]),
        ln(17, [tx("  }")]),
        ln(18, [tx("  "), kw("operation"), tx(" "), ref("UsersEmail", "Email", "derived"), tx(" {")]),
        ln(19, [tx("    input  { id: String }")]),
        ln(20, [tx("    output { email: String }")]),
        ln(21, [tx("  }")]),
        ln(22, [tx("  "), kw("operation"), tx(" "), ref("UsersToken", "Token", "derived"), tx(" {")]),
        ln(23, [tx("    input  { id: String }")]),
        ln(24, [tx("    output { token: String }")]),
        ln(25, [tx("  }")]),
        ln(26, [tx("}")]),
        blank(27),
        ln(28, [com("// gunbc compile --target rust|ts|sql")])
      ],
      graph: {
        nodes: applyNodeRoles(projectionLayoutNodes(system), {
          User:         "stable",
          UsersProfile: "derived",
          UsersEmail:   "derived",
          UsersToken:   "derived"
        }),
        edges: applyEdgeRoles(projectionLayoutEdges(), {}, "derived")
      },
      receipt: [
        { label: "structure",    value: "{stable:one description} · {derived:three operations}" },
        { label: "emissions",    value: "{artifact:profile.rs} · {artifact:email.ts} · {artifact:token.sql}" },
        { label: "backends",     value: "{derived:Rust · TypeScript · SQL}" },
        { label: "translations", value: "{derived:zero hand-written}" }
      ]
    };
  }

  /* ════════════════════════════════════════════════════════════════
     CARD 02 · affected-set propagation — same layout, different roles
     ══════════════════════════════════════════════════════════════ */

  function affectedSetCard(system) {
    return {
      id: "card-02",
      num: "02",
      name: "edit Timestamp · one branch re-derived, two unrelated",
      systemId: system.id,
      codeFile: "examples/users.dag",
      shape: "lane-tree",
      code: [
        diffRm(1,  [kw("type"), tx(" "), ref("Timestamp", "Timestamp", "focus"), tx(" = UnixMillis")]),
        diffAdd(1, [kw("type"), tx(" "), ref("Timestamp", "Timestamp", "focus"), tx(" = IsoDateTime")]),
        blank(2),
        ln(3,  [kw("type"), tx(" User {")]),
        ln(4,  [tx("  id: String")]),
        ln(5,  [tx("  email: String")]),
        ln(6,  [tx("  name: String")]),
        ln(7,  [tx("  "), ref("user_created_at", "created_at", "derived"), tx(": "), ref("Timestamp")]),
        ln(8,  [tx("}")]),
        blank(9),
        ln(10, [kw("type"), tx(" Profile {")]),
        ln(11, [tx("  name: String")]),
        ln(12, [tx("  joined: String")]),
        ln(13, [tx("}")]),
        blank(14),
        ln(15, [kw("fn"), tx(" format_joined(t: "), ref("Timestamp"), tx(") -> String {")]),
        ln(16, [tx("  iso8601(t: t)")]),
        ln(17, [tx("}")]),
        blank(18),
        ln(19, [kw("fn"), tx(" build_profile(u: "), ty("User"), tx(") -> Profile {")]),
        ln(20, [tx("  Profile {")]),
        ln(21, [tx("    name: u.name,")]),
        ln(22, [tx("    joined: format_joined(t: u."),
                ref("user_created_at", "created_at", "derived"), tx(")")]),
        ln(23, [tx("  }")]),
        ln(24, [tx("}")]),
        blank(25),
        ln(26, [kw("service"), tx(" Users {")]),
        ln(27, [tx("  "), kw("operation"), tx(" "), ref("UsersProfile", "Profile", "transitive"), tx(" {")]),
        ln(28, [tx("    input  { id: String }")]),
        ln(29, [tx("    output { profile: Profile }")]),
        ln(30, [tx("  }")]),
        ln(31, [tx("}")])
      ],
      // Lane tree: 4 columns × 3 rows. Each row is a branch; edges
      // only connect adjacent cells in the same row. Overlap is
      // structurally impossible.
      lanes: {
        columns: ["fact", "field", "operation", "artifact"],
        rows: [
          {
            id: "profile",
            cells: [
              { id: "Timestamp",       label: "Timestamp",        role: "focus" },
              { id: "user_created_at", label: "User.created_at",  role: "derived",
                via: "format_joined(...)" },
              { id: "UsersProfile",    label: "Users.profile",    role: "transitive" },
              { id: "profile_rs",      label: "profile.rs",       role: "artifact",
                kind: "artifact" }
            ]
          },
          {
            id: "email",
            cells: [
              null,
              { id: "user_email",      label: "User.email",       role: "context" },
              { id: "UsersEmail",      label: "Users.email",      role: "context" },
              { id: "email_rs",        label: "email.ts",         role: "context",
                kind: "artifact" }
            ]
          },
          {
            id: "token",
            cells: [
              null,
              { id: "user_id",         label: "User.id",          role: "context" },
              { id: "UsersToken",      label: "Users.token",      role: "context" },
              { id: "token_rs",        label: "token.sql",        role: "context",
                kind: "artifact" }
            ]
          }
        ]
      },
      receipt: [
        { label: "changed",    value: "{focus:Timestamp representation}" },
        { label: "direct",     value: "{derived:User.created_at}" },
        { label: "transitive", value: "{transitive:Profile.joined} · {transitive:Users.profile}" },
        { label: "re-derived", value: "{artifact:profile.rs}" },
        { label: "unrelated",  value: "{context:User.email · User.id · Users.email · Users.token · email.ts · token.sql}" },
        { label: "hand-edits", value: "{derived:zero}" }
      ]
    };
  }

  /* ════════════════════════════════════════════════════════════════
     CARD 03 · complexity budget violation — diagnostic absorbed
     ══════════════════════════════════════════════════════════════ */

  function complexityViolationCard(system) {
    return {
      id: "card-03",
      num: "03",
      name: "complexity budget · O(n) declared, O(n²) observed",
      systemId: system.id,
      codeFile: "examples/users.dag",
      shape: "cost-area",
      code: [
        ln(1, [kw("fn"), tx(" "), ref("unique_users", "unique_users", "focus"),
               tx("(users: List<"), ref("User", "User", "context"), tx(">) -> List<User> {")]),
        ln(2, [tx("  users |> "), ref("filter_call", "filter", "derived"), tx("(u =>")]),
        diffAdd(3, [tx("    "), ref("count_call", "count", "boundary"),
                    tx("(users, v => v.id == u.id) == 1)")]),
        ln(4, [tx("}")]),
        blank(5),
        ln(6, [com("// complexity budget: O(n)")])
      ],
      panels: [
        { id: "declared", label: "declared", sublabel: "O(n)",  cols: 6, rows: 1, budgetRows: 1 },
        { id: "observed", label: "observed", sublabel: "O(n²)", cols: 6, rows: 6, budgetRows: 1 }
      ],
      graph: {
        nodes: [
          node("unique_users", "unique_users", 0, 0, "focus"),
          node("filter_call",  "filter",       0, 0, "derived"),
          node("count_call",   "count",        0, 0, "boundary"),
          node("User",         "User",         0, 0, "context")
        ],
        edges: []
      },
      receipt: [
        { label: "declared",          value: "{derived:unique_users ≤ O(n) budget}" },
        { label: "observed",          value: "{boundary:filter(users) × count(users) = O(n²)}" },
        { label: "reached through",   value: "{boundary:filter(users) → count(users)}" },
        { label: "suggested",         value: "{derived:index users by id once, filter buckets of size 1}" },
        { label: "replacement shape", code: [
          "let by_id = group_by(users, key: u => u.id)",
          "by_id |> values |> filter(bucket => length(bucket) == 1)"
        ]},
        { label: "result",            value: "{boundary:no artifact emitted}" }
      ]
    };
  }

  /* ════════════════════════════════════════════════════════════════
     BOOTSTRAP
     ══════════════════════════════════════════════════════════════ */

  const DECK = {
    name: "users.dag · one system · three views",
    cards: [
      projectionClosureCard(USERS_SYSTEM),
      affectedSetCard(USERS_SYSTEM),
      complexityViolationCard(USERS_SYSTEM)
    ]
  };

  function init() {
    const slot = document.querySelector('[data-deck="api.dag"]')
              || document.querySelector('[data-deck="users.dag"]')
              || document.querySelector('[data-deck]');
    if (!slot) return;
    slot.outerHTML = renderDeck(DECK);
    const deck = document.querySelector('.deck');
    if (deck) initDeck(deck);
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
