import { describe, expect, it } from "bun:test";

import { setLocale, type Locale } from "./i18n";
import { CONTENT } from "./public-content";

function collectShape(value: unknown, path = "CONTENT"): string[] {
  if (Array.isArray(value)) {
    const objectItems = value.every(isRecordLike);
    return [
      `${path}:array${objectItems ? `:${value.length}` : ""}`,
      ...(objectItems
        ? value.flatMap((item, index) => collectShape(item, `${path}[${index}]`))
        : []),
    ];
  }
  if (isRecordLike(value)) {
    const record = value as Record<string, unknown>;
    const keys = Object.keys(record).sort();
    return [
      `${path}:object:${keys.join("|")}`,
      ...keys.flatMap((key) => collectShape(record[key], `${path}.${key}`)),
    ];
  }
  return [`${path}:leaf`];
}

function isRecordLike(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function contentShapeFor(locale: Locale): string[] {
  setLocale(locale);
  return collectShape(CONTENT);
}

describe("public content key parity", () => {
  it("keeps zh and en runtime content trees in the same shape", () => {
    const zhShape = contentShapeFor("zh");
    const enShape = contentShapeFor("en");

    expect(enShape).toEqual(zhShape);
  });
});
