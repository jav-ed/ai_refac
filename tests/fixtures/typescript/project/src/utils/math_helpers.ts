// Control file — no dependency on date_helpers. Must not be touched by the move.

export function clamp(n: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, n));
}

export function sum(values: number[]): number {
  return values.reduce((acc, v) => acc + v, 0);
}
