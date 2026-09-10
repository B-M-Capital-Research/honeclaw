// Tables on the exported share card.
//
// The card is a 420px portrait image. A markdown table with more than three
// or four columns cannot live inside it as a table: each column gets under
// 50px and every cell shatters into one character per line, so the shared
// answer reads as noise exactly where the numbers are. Instead of squeezing,
// this module reshapes each table by its silhouette, before the HTML reaches
// the DOM:
//
//   keep        it fits at its natural width — leave it alone;
//   transpose   wide but short (two or three data rows) — rotate it so the
//               few entities become columns and the many measures become
//               rows, which is what a phone reader wants anyway;
//   list        wide and tall — one small block per row, the first cell as
//               its title and the other cells as "header: value" lines.
//
// Width is estimated from the text, not measured: CJK glyphs count as one em,
// Latin and digits as roughly half. That keeps the decision deterministic,
// testable, and identical between the live preview and the rasterised PNG.

export type ShareTableLayout = "keep" | "transpose" | "list";

export type ShareTableShape = {
  header: string[];
  rows: string[][];
};

/** Horizontal room the card gives a table: body padding is 20px each side. */
export const SHARE_TABLE_INNER_WIDTH_PX = 420 - 40;
/** The card scales table type to 0.9 of the message size (chat-share.css). */
export const SHARE_TABLE_FONT_SCALE = 0.9;
/** th/td padding-right, in em at table size. */
const CELL_PAD_EM = 0.6;
/** A prose column is allowed to wrap; this is the width it wraps at. */
const WRAP_CAP_EM = 12;
/** Beyond this many data rows a rotated table would be just as wide. */
const TRANSPOSE_MAX_ROWS = 3;

export function shareTableCapacityEm(messageFontSize: number) {
  return SHARE_TABLE_INNER_WIDTH_PX / (SHARE_TABLE_FONT_SCALE * messageFontSize);
}

function isWideCodePoint(code: number) {
  return (
    (code >= 0x1100 && code <= 0x115f) ||
    (code >= 0x2e80 && code <= 0xa4cf) ||
    (code >= 0xac00 && code <= 0xd7a3) ||
    (code >= 0xf900 && code <= 0xfaff) ||
    (code >= 0xfe30 && code <= 0xfe4f) ||
    (code >= 0xff00 && code <= 0xff60) ||
    (code >= 0xffe0 && code <= 0xffe6) ||
    (code >= 0x1f300 && code <= 0x1faff) ||
    (code >= 0x20000 && code <= 0x3fffd)
  );
}

/** Rough advance width of a run of text, in em of the table face. */
export function estimateTextEm(text: string) {
  let width = 0;
  for (const ch of text) {
    const code = ch.codePointAt(0) ?? 0;
    if (ch === " ") width += 0.3;
    else if (isWideCodePoint(code)) width += 1;
    else if (ch >= "0" && ch <= "9") width += 0.58;
    else if (ch >= "A" && ch <= "Z") width += 0.68;
    else width += 0.52;
  }
  return width;
}

/** Widest unbreakable run: CJK breaks anywhere, Latin only at spaces. */
function longestTokenEm(text: string) {
  let longest = 0;
  let run = 0;
  for (const ch of text) {
    const code = ch.codePointAt(0) ?? 0;
    if (ch === " " || isWideCodePoint(code)) {
      longest = Math.max(longest, run);
      run = ch === " " ? 0 : 1;
      continue;
    }
    run += estimateTextEm(ch);
  }
  return Math.max(longest, run);
}

/** Room a column needs once its prose is allowed to wrap at WRAP_CAP_EM. */
function columnWidthEm(cells: string[]) {
  let widest = 0;
  let unbreakable = 0;
  for (const cell of cells) {
    widest = Math.max(widest, estimateTextEm(cell));
    unbreakable = Math.max(unbreakable, longestTokenEm(cell));
  }
  return Math.max(Math.min(widest, WRAP_CAP_EM), unbreakable) + CELL_PAD_EM;
}

function columns(shape: ShareTableShape) {
  return Math.max(shape.header.length, ...shape.rows.map((row) => row.length));
}

export function estimateTableWidthEm(shape: ShareTableShape) {
  const count = columns(shape);
  let total = 0;
  for (let index = 0; index < count; index += 1) {
    total += columnWidthEm([
      shape.header[index] ?? "",
      ...shape.rows.map((row) => row[index] ?? ""),
    ]);
  }
  return total;
}

export function transposeTable(shape: ShareTableShape): ShareTableShape {
  const count = columns(shape);
  const header = [shape.header[0] ?? "", ...shape.rows.map((row) => row[0] ?? "")];
  const rows: string[][] = [];
  for (let index = 1; index < count; index += 1) {
    rows.push([
      shape.header[index] ?? "",
      ...shape.rows.map((row) => row[index] ?? ""),
    ]);
  }
  return { header, rows };
}

export function decideShareTableLayout(
  shape: ShareTableShape,
  capacityEm: number,
): ShareTableLayout {
  const count = columns(shape);
  // One or two columns wrap gracefully whatever they hold.
  if (count <= 2) return "keep";
  if (estimateTableWidthEm(shape) <= capacityEm) return "keep";
  if (
    shape.rows.length > 0 &&
    shape.rows.length <= TRANSPOSE_MAX_ROWS &&
    shape.rows.length + 1 < count &&
    estimateTableWidthEm(transposeTable(shape)) <= capacityEm
  ) {
    return "transpose";
  }
  return "list";
}

// ── DOM reshaping ──

type CellNodes = { text: string; nodes: Node[] };

function readCells(row: Element): CellNodes[] {
  return Array.from(row.children)
    .filter((cell) => cell.tagName === "TH" || cell.tagName === "TD")
    .map((cell) => ({
      text: (cell.textContent ?? "").trim(),
      nodes: Array.from(cell.childNodes),
    }));
}

function readTable(table: HTMLTableElement) {
  const allRows = Array.from(table.querySelectorAll("tr"));
  const headerRow = allRows.find((row) => row.closest("thead")) ?? allRows[0];
  const header = headerRow ? readCells(headerRow) : [];
  const rows = allRows.filter((row) => row !== headerRow).map(readCells);
  return { header, rows };
}

function shapeOf(header: CellNodes[], rows: CellNodes[][]): ShareTableShape {
  return {
    header: header.map((cell) => cell.text),
    rows: rows.map((row) => row.map((cell) => cell.text)),
  };
}

/**
 * Move a cell's approved nodes into a new frame. Header cells are reused by
 * every block of a list, so those are cloned; appending would silently move
 * the label out of the previous block.
 */
function fill(target: Element, cell: CellNodes | undefined, clone = false) {
  if (!cell) return;
  for (const node of cell.nodes) {
    target.appendChild(clone ? node.cloneNode(true) : node);
  }
}

function buildTransposed(
  doc: Document,
  header: CellNodes[],
  rows: CellNodes[][],
) {
  const count = Math.max(header.length, ...rows.map((row) => row.length));
  const table = doc.createElement("table");
  table.setAttribute("data-hone-share-table", "transposed");
  const thead = doc.createElement("thead");
  const headRow = doc.createElement("tr");
  const corner = doc.createElement("th");
  fill(corner, header[0]);
  headRow.appendChild(corner);
  for (const row of rows) {
    const th = doc.createElement("th");
    fill(th, row[0]);
    headRow.appendChild(th);
  }
  thead.appendChild(headRow);
  table.appendChild(thead);
  const tbody = doc.createElement("tbody");
  for (let index = 1; index < count; index += 1) {
    const tr = doc.createElement("tr");
    const th = doc.createElement("th");
    th.setAttribute("scope", "row");
    fill(th, header[index]);
    tr.appendChild(th);
    for (const row of rows) {
      const td = doc.createElement("td");
      fill(td, row[index]);
      tr.appendChild(td);
    }
    tbody.appendChild(tr);
  }
  table.appendChild(tbody);
  return table;
}

function buildList(doc: Document, header: CellNodes[], rows: CellNodes[][]) {
  const count = Math.max(header.length, ...rows.map((row) => row.length));
  // Key column: as wide as the widest header, within a sane band, so values
  // line up down one block without a long header pushing them off the card.
  const keyEm = Math.min(
    9,
    Math.max(
      4,
      ...header.slice(1).map((cell) => estimateTextEm(cell.text) + 0.4),
    ),
  );
  const list = doc.createElement("div");
  list.className = "hf-share-tlist";
  list.setAttribute("data-hone-share-table", "list");
  rows.forEach((row, rowIndex) => {
    const item = doc.createElement("div");
    item.className = "hf-share-tlist__item";
    const title = doc.createElement("div");
    title.className = "hf-share-tlist__title";
    if (row[0] && row[0].text) {
      fill(title, row[0]);
    } else {
      title.textContent = `#${rowIndex + 1}`;
    }
    item.appendChild(title);
    for (let index = 1; index < count; index += 1) {
      const line = doc.createElement("div");
      line.className = "hf-share-tlist__row";
      const key = doc.createElement("span");
      key.className = "hf-share-tlist__k";
      key.style.width = `${keyEm.toFixed(2)}em`;
      fill(key, header[index], true);
      const value = doc.createElement("span");
      value.className = "hf-share-tlist__v";
      fill(value, row[index]);
      if (!value.hasChildNodes()) value.textContent = "—";
      line.appendChild(key);
      line.appendChild(value);
      item.appendChild(line);
    }
    list.appendChild(item);
  });
  return list;
}

/**
 * Reshape every table in sanitised markdown HTML for the card's width. Runs
 * after DOMPurify: it only moves the nodes the sanitiser already approved
 * into new table / div frames, never parses new markup from text.
 */
export function reflowShareTables(html: string, capacityEm: number) {
  if (!html.includes("<table")) return html;
  const doc = new DOMParser().parseFromString(html, "text/html");
  for (const table of Array.from(doc.querySelectorAll("table"))) {
    const { header, rows } = readTable(table);
    if (header.length === 0 || rows.length === 0) continue;
    const layout = decideShareTableLayout(shapeOf(header, rows), capacityEm);
    if (layout === "keep") continue;
    const replacement =
      layout === "transpose"
        ? buildTransposed(doc, header, rows)
        : buildList(doc, header, rows);
    table.replaceWith(replacement);
  }
  return doc.body.innerHTML;
}
