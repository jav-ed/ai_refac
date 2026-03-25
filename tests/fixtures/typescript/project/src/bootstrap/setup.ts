// Side-effect import — exercises bare import with no bindings.
import '../utils/date_helpers';

export function initEnv(): void {
  process.env.TZ = process.env.TZ ?? 'UTC';
}
