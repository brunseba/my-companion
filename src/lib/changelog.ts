import raw from "../../CHANGELOG.md?raw";

export interface Release {
  version: string;
  date: string;
  body: string;
}

// commit-and-tag-version links a release heading to its previous tag once one
// exists ("## [1.2.3](compare-url) (date)"), and leaves it plain otherwise
// ("## 1.2.3 (date)", e.g. --first-release). Unwrap the link first so a
// single simple pattern below can match either shape.
const VERSION_LINK = /\[(\d+\.\d+\.\d+(?:-[\w.]+)?)\]\([^)]*\)/g;

// `commit-and-tag-version` only ever writes this heading shape for a real
// tagged release. Anything else in the file (prose, a manually-added
// "Unreleased" section, etc.) is excluded by simply not matching this pattern
// - so what we show is tag history, not commit history.
const RELEASE_HEADING = /^## (\d+\.\d+\.\d+(?:-[\w.]+)?) \((\d{4}-\d{2}-\d{2})\)$/gm;

export function parseReleases(rawMarkdown: string): Release[] {
  const markdown = rawMarkdown.replaceAll(VERSION_LINK, "$1");
  const matches = [...markdown.matchAll(RELEASE_HEADING)];
  return matches.map((match, index) => {
    const start = match.index! + match[0].length;
    const end = index + 1 < matches.length ? matches[index + 1].index! : markdown.length;
    return {
      version: match[1],
      date: match[2],
      body: markdown.slice(start, end).trim(),
    };
  });
}

function escapeHtml(text: string): string {
  return text.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

/** Turns a release's body (a small, known subset of Markdown) into safe HTML. */
export function renderReleaseBody(body: string): string {
  const html: string[] = [];
  let inList = false;
  const closeList = () => {
    if (inList) {
      html.push("</ul>");
      inList = false;
    }
  };

  for (const rawLine of body.split("\n")) {
    const line = rawLine.trim();
    if (!line) continue;
    if (line.startsWith("### ")) {
      closeList();
      html.push(`<h4>${escapeHtml(line.slice(4))}</h4>`);
    } else if (line.startsWith("* ")) {
      if (!inList) {
        html.push("<ul>");
        inList = true;
      }
      html.push(`<li>${escapeHtml(line.slice(2))}</li>`);
    } else {
      closeList();
      html.push(`<p>${escapeHtml(line)}</p>`);
    }
  }
  closeList();
  return html.join("");
}

// Newest-first, matching how commit-and-tag-version writes the file.
export const releases: Release[] = parseReleases(raw);
