// MOVE TARGET: src/utils/date_helpers.ts -> src/lib/date_helpers.ts

export type DateString = string;

export function formatDate(ts: number): DateString {
  return new Date(ts).toISOString().split('T')[0];
}

export function isOverdue(ts: number): boolean {
  return ts < Date.now();
}

export function daysBetween(a: number, b: number): number {
  return Math.floor(Math.abs(b - a) / 86_400_000);
}
