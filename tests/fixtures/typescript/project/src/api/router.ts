// Control file — imports only from the services barrel, not directly from date_helpers.
// Must not be touched by the move.
import { summarise } from '../services';

export function route(taskJson: string): string {
  const task = JSON.parse(taskJson);
  return summarise(task);
}
