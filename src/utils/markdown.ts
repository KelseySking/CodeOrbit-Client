import { Marked } from "marked";

const md = new Marked({
  gfm: true,
  breaks: true,
});

// CodeOrbit chat is semi-trusted (own machine) but still strip raw HTML tags.
md.use({
  renderer: {
    html({ text }) {
      return escapeHtml(text);
    },
  },
});

export function renderMarkdown(source: string): string {
  const text = source ?? "";
  if (!text.trim()) return "";
  try {
    return md.parse(text, { async: false }) as string;
  } catch {
    return `<p>${escapeHtml(text)}</p>`;
  }
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
