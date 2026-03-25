// Same-directory sibling — relative named import of date_helpers.
import { formatDate } from './date_helpers';

export function labelDate(ts: number): string {
  return `[${formatDate(ts)}]`;
}

export function truncate(s: string, max: number): string {
  return s.length > max ? s.slice(0, max) + '…' : s;
}
