export type InstanceRole = "user" | "superadmin";
export type OrgRole = "member" | "admin" | "owner";
export type ProjectRole = "viewer" | "contributor" | "lead";
export type TaskStatus = "pending" | "in_progress" | "blocked" | "completed";
export type Priority = "low" | "medium" | "high" | "urgent";

export const TASK_STATUSES: TaskStatus[] = [
  "pending",
  "in_progress",
  "blocked",
  "completed",
];

export const STATUS_LABEL: Record<TaskStatus, string> = {
  pending: "Pending",
  in_progress: "In Progress",
  blocked: "Blocked",
  completed: "Completed",
};

export const PRIORITY_LABEL: Record<Priority, string> = {
  low: "Low",
  medium: "Medium",
  high: "High",
  urgent: "Urgent",
};

export interface User {
  id: string;
  email: string;
  display_name: string;
  instance_role: InstanceRole;
}

export interface Org {
  id: string;
  name: string;
  slug: string;
  created_at: string;
  updated_at: string;
}

export interface OrgMember {
  user_id: string;
  email: string;
  display_name: string;
  role: OrgRole;
}

export interface Invite {
  id: string;
  org_id: string;
  email: string;
  role: OrgRole;
  expires_at: string;
  accepted_at: string | null;
  created_at: string;
}

export interface InviteCreated extends Invite {
  token: string;
}

export interface Project {
  id: string;
  org_id: string;
  key: string;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface ProjectMember {
  user_id: string;
  email: string;
  display_name: string;
  role: ProjectRole;
}

export interface Tag {
  id: string;
  project_id: string;
  name: string;
  color: string;
}

export interface Task {
  id: string;
  org_id: string;
  project_id: string;
  task_number: number;
  title: string;
  description: string | null;
  status: TaskStatus;
  priority: Priority;
  assignee_id: string | null;
  reporter_id: string;
  due_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface Comment {
  id: string;
  task_id: string;
  author_id: string;
  body: string;
  created_at: string;
  updated_at: string;
}

export interface Attachment {
  id: string;
  task_id: string;
  uploaded_by: string;
  file_name: string;
  content_type: string;
  size_bytes: number;
  created_at: string;
}

export interface TaskDetail extends Task {
  tags: Tag[];
  comments: Comment[];
  attachments: Attachment[];
}
