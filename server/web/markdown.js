// Renders the role conversation's replies (markdown) as HTML, safely: the
// text is HTML-escaped first, so only the markup this file emits can
// appear. Covers what the model writes: paragraphs, headings, bullet and
// numbered lists, fenced code, **bold**, *italic* and `code`. Anything else
// stays as plain text.
(function () {
  function esc(s) {
    return s.replace(/[&<>"]/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
  }

  // Inline markup on already-escaped text.
  function inline(s) {
    return s
      .replace(/`([^`]+)`/g, "<code>$1</code>")
      .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
      .replace(/(^|[^*\w])\*([^*\s][^*]*)\*(?!\w)/g, "$1<em>$2</em>");
  }

  function renderMarkdown(src) {
    const lines = esc(src).split("\n");
    const out = [];
    let para = [];
    let list = null; // { tag, items }
    const flushPara = () => {
      if (para.length) out.push("<p>" + para.map(inline).join("<br>") + "</p>");
      para = [];
    };
    const flushList = () => {
      if (list) out.push(`<${list.tag}>` + list.items.map(i => "<li>" + inline(i) + "</li>").join("") + `</${list.tag}>`);
      list = null;
    };
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (/^\s*```/.test(line)) {
        flushPara(); flushList();
        const code = [];
        for (i++; i < lines.length && !/^\s*```/.test(lines[i]); i++) code.push(lines[i]);
        out.push("<pre><code>" + code.join("\n") + "</code></pre>");
        continue;
      }
      const h = line.match(/^(#{1,6})\s+(.*)$/);
      const ul = line.match(/^\s*[-*]\s+(.*)$/);
      const ol = line.match(/^\s*\d+[.)]\s+(.*)$/);
      if (h) {
        flushPara(); flushList();
        const n = Math.min(h[1].length + 2, 6); // keep reply headings below the page's
        out.push(`<h${n}>${inline(h[2])}</h${n}>`);
      } else if (ul || ol) {
        flushPara();
        const tag = ul ? "ul" : "ol";
        if (!list || list.tag !== tag) { flushList(); list = { tag, items: [] }; }
        list.items.push((ul || ol)[1]);
      } else if (!line.trim()) {
        flushPara(); flushList();
      } else if (list && /^\s{2,}\S/.test(line)) {
        list.items[list.items.length - 1] += " " + line.trim(); // a wrapped list item
      } else {
        flushList();
        para.push(line);
      }
    }
    flushPara(); flushList();
    return out.join("");
  }

  window.renderMarkdown = renderMarkdown;
})();
