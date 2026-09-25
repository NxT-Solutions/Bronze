import { describe, expect, it } from "vitest";
import {
  FUZZY_EXACT,
  FUZZY_PREFIX,
  FUZZY_SUBSEQUENCE,
  FUZZY_SUBSTRING,
  FUZZY_TYPO,
  fuzzyFilter,
  fuzzyMatchScore,
} from "./fuzzy-match";

describe("fuzzyMatchScore", () => {
  it("ranks exact, prefix, substring, subsequence, then a small typo", () => {
    expect(fuzzyMatchScore("Bronze", "", "en")).toBe(0);
    expect(fuzzyMatchScore("Bronze", "   ", "en")).toBe(0);
    expect(fuzzyMatchScore("Bronze", "Bronze", "en")).toBe(FUZZY_EXACT);
    expect(fuzzyMatchScore("Bronze", "BRONZE", "en")).toBe(FUZZY_EXACT);
    expect(fuzzyMatchScore("Bronze", "Bron", "en")).toBe(FUZZY_PREFIX + 998);
    expect(fuzzyMatchScore("xxBronze", "bronze", "en")).toBe(
      FUZZY_SUBSTRING + 998,
    );
    expect(fuzzyMatchScore("Bronze", "brn", "en")).toBe(
      FUZZY_SUBSEQUENCE + 998,
    );
    expect(fuzzyMatchScore("Bronze", "Bronxe", "en")).toBe(FUZZY_TYPO + 1000);
    expect(fuzzyMatchScore("Bronze", "Bronez", "en")).toBe(FUZZY_TYPO + 1000);
    expect(fuzzyMatchScore("Bronze", "zzzz", "en")).toBeNull();
  });

  it("does not fold diacritics", () => {
    expect(fuzzyMatchScore("Café token", "cafe", "en")).toBeNull();
    expect(fuzzyMatchScore("Café token", "CAFÉ", "en")).toBe(
      FUZZY_PREFIX + 994,
    );
  });

  it("folds Turkish I with the UI locale", () => {
    expect(fuzzyMatchScore("Istanbul", "ıstanbul", "tr")).toBe(FUZZY_EXACT);
    expect(fuzzyMatchScore("Istanbul", "istanbul", "tr")).toBeNull();
    expect(fuzzyMatchScore("Istanbul", "istanbul", "en")).toBe(FUZZY_EXACT);
    expect(fuzzyMatchScore("İstanbul", "istanbul", "tr")).toBe(FUZZY_EXACT);
    expect(fuzzyMatchScore("İstanbul", "istanbul", "en")).not.toBe(FUZZY_EXACT);
    expect(fuzzyMatchScore("Istanbul", "ıstanbul", "tr-TR")).toBe(FUZZY_EXACT);
  });

  it("keeps the original order for an empty query and drops a miss", () => {
    const items = ["My Bronze", "Bronze", "Notes"];
    expect(fuzzyFilter(items, "  ", "en", (item) => item)).toEqual(items);
    expect(fuzzyFilter(items, "zzzz", "en", (item) => item)).toEqual([]);
    expect(fuzzyFilter(items, "BRONZE", "en", (item) => item)).toEqual([
      "Bronze",
      "My Bronze",
    ]);
    expect(fuzzyFilter(items, "brn", "en", (item) => item)).toEqual([
      "Bronze",
      "My Bronze",
    ]);
  });
});
