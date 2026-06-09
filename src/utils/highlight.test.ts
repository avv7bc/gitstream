import { describe, it, expect } from "vitest";
import { highlight } from "./highlight";

describe("highlight", () => {
  it("escapes HTML when query is empty", () => {
    expect(highlight("<b>x</b>", "")).toBe("&lt;b&gt;x&lt;/b&gt;");
  });

  it("returns empty string for null/undefined", () => {
    expect(highlight(null, "a")).toBe("");
    expect(highlight(undefined, "a")).toBe("");
  });

  it("coerces numbers to string", () => {
    expect(highlight(42, "")).toBe("42");
  });

  it("wraps matches in <mark>", () => {
    expect(highlight("foobar", "oo")).toBe('f<mark class="hl">oo</mark>bar');
  });

  it("is case-insensitive", () => {
    expect(highlight("FooBar", "foo")).toBe('<mark class="hl">Foo</mark>Bar');
  });

  it("highlights all occurrences", () => {
    expect(highlight("aXaXa", "a")).toBe(
      '<mark class="hl">a</mark>X<mark class="hl">a</mark>X<mark class="hl">a</mark>',
    );
  });

  it("treats query as literal, not regex", () => {
    // '.' must not match any char — only a literal dot.
    expect(highlight("abc", ".")).toBe("abc");
    expect(highlight("a.c", ".")).toBe('a<mark class="hl">.</mark>c');
  });

  it("does not emit unescaped markup from text (XSS-safe for v-html)", () => {
    const out = highlight('<img src=x onerror="alert(1)">', "img");
    expect(out).not.toContain("<img");
    expect(out).toContain("&lt;");
    expect(out).toContain('<mark class="hl">img</mark>');
  });

  it("matches against escaped text, so HTML-special queries work", () => {
    // The text becomes "a &lt; b"; querying "<" (escaped to "&lt;") must match.
    expect(highlight("a < b", "<")).toBe('a <mark class="hl">&lt;</mark> b');
  });

  it("ignores leading/trailing whitespace in query", () => {
    expect(highlight("foobar", "  oo  ")).toBe('f<mark class="hl">oo</mark>bar');
  });
});
