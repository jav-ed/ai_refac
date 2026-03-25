// Exercises: namespace import (import * as) and import type of a symbol from date_helpers.
import type { Task } from '../types/task';
import type { DateString } from '../utils/date_helpers';
import * as DateHelpers from '../utils/date_helpers';

export class TaskModel {
  constructor(private data: Task) {}

  isOverdue(): boolean {
    return DateHelpers.isOverdue(this.data.dueAt);
  }

  formattedDue(): DateString {
    return DateHelpers.formatDate(this.data.dueAt);
  }
}
