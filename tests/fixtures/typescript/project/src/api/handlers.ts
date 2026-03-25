// Cross-layer named import — verifies multi-file, multi-depth update coverage.
import { formatDate } from '../utils/date_helpers';
import { summarise } from '../services/task_service';
import type { Task } from '../types/task';

export function renderTask(task: Task): string {
  return `${summarise(task)} — rendered at ${formatDate(Date.now())}`;
}
