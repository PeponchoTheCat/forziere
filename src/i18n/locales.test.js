import { describe, expect, it } from "vitest";
import en from "./locales/en.json";
import it_ from "./locales/it.json";

const flatten = (obj, prefix = "") =>
  Object.entries(obj).flatMap(([k, v]) =>
    typeof v === "object" ? flatten(v, `${prefix}${k}.`) : [[`${prefix}${k}`, v]],
  );
const placeholders = (text) => [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

describe("locales", () => {
  it("English and Italian have exactly the same keys", () => {
    // Arrange / Act
    const enKeys = flatten(en).map(([k]) => k).sort();
    const itKeys = flatten(it_).map(([k]) => k).sort();

    // Assert
    expect(itKeys).toEqual(enKeys);
  });

  it("every key uses the same placeholders in both languages", () => {
    // Arrange
    const itMap = Object.fromEntries(flatten(it_));

    // Act
    const mismatches = flatten(en).filter(([k, v]) => placeholders(v).join() !== placeholders(itMap[k]).join());

    // Assert
    expect(mismatches.map(([k]) => k)).toEqual([]);
  });

  it("no translation is empty", () => {
    // Arrange / Act
    const empty = [...flatten(en), ...flatten(it_)].filter(([, v]) => !v.trim());

    // Assert
    expect(empty).toEqual([]);
  });
});
