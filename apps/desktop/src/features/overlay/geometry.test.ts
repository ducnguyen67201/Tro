import { describe, expect, it } from "vitest";
import { toPercent } from "./geometry";

describe("overlay geometry", () => {
  it("clamps normalized coordinates", () => {
    expect(toPercent({ x: -1, y: 2 })).toEqual({ left: "0%", top: "100%" });
  });
});
