// Second file with a named import — verifies the tool updates ALL files, not just the first.
import { daysBetween } from '../utils/date_helpers';
import type { User } from '../types/user';

export function accountAge(user: User & { createdAt: number }): number {
  return daysBetween(user.createdAt, Date.now());
}
