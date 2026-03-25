// Exercises: named import + static dynamic import() of date_helpers.
import { formatDate, isOverdue } from '../utils/date_helpers';
import type { Task } from '../types/task';

export function summarise(task: Task): string {
  const status = isOverdue(task.dueAt) ? 'OVERDUE' : 'ok';
  return `${task.title} [${formatDate(task.dueAt)}] ${status}`;
}

export async function loadDateHelpers() {
  // Static-string dynamic import — ts-morph updates this path.
  const mod = await import('../utils/date_helpers');
  return mod.formatDate;
}
