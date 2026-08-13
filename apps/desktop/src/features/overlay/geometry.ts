import type { Point } from "../../lib/contracts";

export const toPercent = (point: Point): { left: string; top: string } => ({
  left: `${Math.min(1, Math.max(0, point.x)) * 100}%`,
  top: `${Math.min(1, Math.max(0, point.y)) * 100}%`,
});

export const line = (from: Point, to: Point) => ({
  x1: `${from.x * 100}%`,
  y1: `${from.y * 100}%`,
  x2: `${to.x * 100}%`,
  y2: `${to.y * 100}%`,
});
