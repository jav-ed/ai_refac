// Control file — no dependency on date_helpers. Must not be touched by the move.
import type { User } from '../types/user';

export class UserModel {
  constructor(private data: User) {}

  displayName(): string {
    return this.data.name;
  }
}
