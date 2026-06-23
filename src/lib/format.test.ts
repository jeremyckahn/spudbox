import { describe, expect, it } from "vitest";
import { formatDuration } from "./format";

describe("formatDuration", () => {
  it("formats whole minutes with zero seconds", () => {
    expect(formatDuration(0)).toBe("0:00");
    expect(formatDuration(60_000)).toBe("1:00");
  });

  it("pads single-digit seconds", () => {
    expect(formatDuration(65_000)).toBe("1:05");
  });

  it("does not pad minutes", () => {
    expect(formatDuration(3_661_000)).toBe("61:01");
  });

  it("rounds to the nearest second", () => {
    expect(formatDuration(59_600)).toBe("1:00");
    expect(formatDuration(59_400)).toBe("0:59");
  });
});
