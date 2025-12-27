use std::path::Path;
use super::utils::create_file;

pub fn generate(root: &Path) -> std::io::Result<()> {
    let base = root.join("typescript");
    println!("Generating Complex TypeScript project (Task Manager Domain)...");

    // 1. Config
    create_file(&base, "tsconfig.json", r#"{
    "compilerOptions": {
        "target": "es2016",
        "module": "commonjs",
        "esModuleInterop": true,
        "strict": true,
        "skipLibCheck": true,
        "baseUrl": "./",
        "paths": { "@/*": ["src/*"] }
    }
}"#)?;

    create_file(&base, "package.json", r#"{
    "name": "complex-task-manager",
    "version": "2.0.0",
    "main": "src/index.ts"
}"#)?;

    // 2. Constants & Types (File 1 & 2)
    create_file(&base, "src/constants.ts", r#"
export const MAX_PRIORITY = 5;
export const MIN_PRIORITY = 1;
export const DEFAULT_PAGE_SIZE = 20;
export const API_ENDPOINT = "https://api.example.com/v1";
"#)?;

    create_file(&base, "src/types/index.ts", r#"
export enum TaskStatus {
    TODO = "TODO",
    IN_PROGRESS = "IN_PROGRESS",
    DONE = "DONE"
}

export interface User {
    id: string;
    username: string;
    email: string;
    isActive: boolean;
}

export interface Task {
    id: string;
    title: string;
    description: string;
    status: TaskStatus;
    assigneeId?: string;
    priority: number;
    createdAt: Date;
}
"#)?;

    // 3. Utilities (File 3)
    create_file(&base, "src/utils/date_helpers.ts", r#"
export function formatDate(date: Date): string {
    return date.toISOString().split('T')[0];
}

export function isOverdue(date: Date): boolean {
    const now = new Date();
    return date.getTime() < now.getTime();
}
"#)?;

    // 4. Models/Classes (File 4 & 5)
    create_file(&base, "src/models/TaskManager.ts", r#"
import { Task, TaskStatus } from "../types";
import { MAX_PRIORITY } from "../constants";

export class TaskManager {
    private tasks: Task[] = [];

    addTask(task: Task): void {
        if (task.priority > MAX_PRIORITY) {
            throw new Error("Invalid priority");
        }
        this.tasks.push(task);
    }

    getTasksByStatus(status: TaskStatus): Task[] {
        return this.tasks.filter(t => t.status === status);
    }
}
"#)?;

    create_file(&base, "src/models/NotificationService.ts", r#"
import { User } from "../types";

export class NotificationService {
    sendEmail(user: User, subject: string, body: string): void {
        if (!user.email.includes("@")) {
            console.error("Invalid email for user " + user.username);
            return;
        }
        console.log(`Sending email to ${user.email}: ${subject}`);
    }
}
"#)?;

    // 5. App Logic (File 6)
    create_file(&base, "src/app.ts", r#"
import { TaskManager } from "./models/TaskManager";
import { NotificationService } from "./models/NotificationService";
import { Task, TaskStatus, User } from "./types";
import { MAX_PRIORITY } from "./constants";

export class App {
    private taskManager = new TaskManager();
    private startMessage: string = "Starting Task App...";

    init() {
        console.log(this.startMessage);
        
        const superUser: User = {
            id: "u1",
            username: "admin",
            email: "admin@company.com",
            isActive: true
        };

        const initialTask: Task = {
            id: "t1",
            title: "Init Project",
            description: "Setup the infrastructure",
            status: TaskStatus.TODO,
            priority: MAX_PRIORITY,
            createdAt: new Date()
        };

        this.taskManager.addTask(initialTask);
    }
}
"#)?;

    // 6. Entry Point (File 7)
    create_file(&base, "src/index.ts", r#"
import { App } from "./app";

const app = new App();
app.init();
"#)?;

    Ok(())
}
