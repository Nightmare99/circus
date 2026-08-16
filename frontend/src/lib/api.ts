import type {
  Attachment,
  Comment,
  Invite,
  InviteCreated,
  Org,
  OrgMember,
  OrgRole,
  Priority,
  Project,
  ProjectMember,
  ProjectRole,
  Tag,
  Task,
  TaskDetail,
  TaskStatus,
  User,
} from "./types";

let accessToken: string | null = null;
let refreshPromise: Promise<string | null> | null = null;

export function setAccessToken(token: string | null) {
  accessToken = token;
}

export function getAccessToken() {
  return accessToken;
}

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function doRefresh(): Promise<string | null> {
  try {
    const res = await fetch("/api/auth/refresh", {
      method: "POST",
      credentials: "include",
    });
    if (!res.ok) {
      accessToken = null;
      return null;
    }
    const data = await res.json();
    accessToken = data.access_token;
    return accessToken;
  } catch {
    accessToken = null;
    return null;
  }
}

export async function apiFetch<T>(
  path: string,
  init: RequestInit = {},
  retry = true,
): Promise<T> {
  const headers = new Headers(init.headers);
  if (accessToken) headers.set("Authorization", `Bearer ${accessToken}`);
  if (init.body && !(init.body instanceof FormData) && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }
  const res = await fetch(`/api${path}`, {
    ...init,
    headers,
    credentials: "include",
  });
  if (res.status === 401 && retry) {
    if (!refreshPromise) {
      refreshPromise = doRefresh().finally(() => {
        refreshPromise = null;
      });
    }
    const newToken = await refreshPromise;
    if (newToken) return apiFetch<T>(path, init, false);
  }
  if (!res.ok) {
    let message = res.statusText;
    try {
      const body = await res.json();
      message = body.error ?? message;
    } catch {
      // response had no JSON body
    }
    throw new ApiError(res.status, message);
  }
  if (res.status === 204) return undefined as T;
  const contentType = res.headers.get("content-type") || "";
  if (contentType.includes("application/json")) return res.json();
  return undefined as T;
}

const json = (body: unknown) => JSON.stringify(body);

export const api = {
  auth: {
    async register(email: string, password: string, displayName: string) {
      const data = await apiFetch<{ access_token: string; user: User }>(
        "/auth/register",
        { method: "POST", body: json({ email, password, display_name: displayName }) },
      );
      setAccessToken(data.access_token);
      return data.user;
    },
    async login(email: string, password: string) {
      const data = await apiFetch<{ access_token: string; user: User }>(
        "/auth/login",
        { method: "POST", body: json({ email, password }) },
      );
      setAccessToken(data.access_token);
      return data.user;
    },
    async refresh() {
      const token = await doRefresh();
      return token !== null;
    },
    async logout() {
      await apiFetch("/auth/logout", { method: "POST" });
      setAccessToken(null);
    },
    me: () => apiFetch<User>("/auth/me"),
  },

  orgs: {
    list: () => apiFetch<Org[]>("/orgs"),
    create: (name: string) => apiFetch<Org>("/orgs", { method: "POST", body: json({ name }) }),
    get: (orgId: string) => apiFetch<Org>(`/orgs/${orgId}`),
    members: (orgId: string) => apiFetch<OrgMember[]>(`/orgs/${orgId}/members`),
    updateMemberRole: (orgId: string, userId: string, role: OrgRole) =>
      apiFetch<OrgMember>(`/orgs/${orgId}/members/${userId}`, {
        method: "PATCH",
        body: json({ role }),
      }),
    removeMember: (orgId: string, userId: string) =>
      apiFetch<void>(`/orgs/${orgId}/members/${userId}`, { method: "DELETE" }),
    invites: (orgId: string) => apiFetch<Invite[]>(`/orgs/${orgId}/invites`),
    createInvite: (orgId: string, email: string, role: OrgRole) =>
      apiFetch<InviteCreated>(`/orgs/${orgId}/invites`, {
        method: "POST",
        body: json({ email, role }),
      }),
    revokeInvite: (orgId: string, inviteId: string) =>
      apiFetch<void>(`/orgs/${orgId}/invites/${inviteId}`, { method: "DELETE" }),
    acceptInvite: (token: string) =>
      apiFetch<Org>("/invites/accept", { method: "POST", body: json({ token }) }),
  },

  projects: {
    list: (orgId: string) => apiFetch<Project[]>(`/orgs/${orgId}/projects`),
    create: (orgId: string, key: string, name: string, description: string) =>
      apiFetch<Project>(`/orgs/${orgId}/projects`, {
        method: "POST",
        body: json({ key, name, description: description || null }),
      }),
    get: (projectId: string) => apiFetch<Project>(`/projects/${projectId}`),
    remove: (projectId: string) => apiFetch<void>(`/projects/${projectId}`, { method: "DELETE" }),
    members: (projectId: string) => apiFetch<ProjectMember[]>(`/projects/${projectId}/members`),
    addMember: (projectId: string, userId: string, role: ProjectRole) =>
      apiFetch<ProjectMember>(`/projects/${projectId}/members`, {
        method: "POST",
        body: json({ user_id: userId, role }),
      }),
    removeMember: (projectId: string, userId: string) =>
      apiFetch<void>(`/projects/${projectId}/members/${userId}`, { method: "DELETE" }),
    tags: (projectId: string) => apiFetch<Tag[]>(`/projects/${projectId}/tags`),
    createTag: (projectId: string, name: string, color: string) =>
      apiFetch<Tag>(`/projects/${projectId}/tags`, {
        method: "POST",
        body: json({ name, color }),
      }),
    deleteTag: (projectId: string, tagId: string) =>
      apiFetch<void>(`/projects/${projectId}/tags/${tagId}`, { method: "DELETE" }),
  },

  tasks: {
    list: (
      projectId: string,
      params: { status?: TaskStatus; assignee_id?: string; tag_id?: string; search?: string } = {},
    ) => {
      const qs = new URLSearchParams(
        Object.entries(params).filter(([, v]) => v) as [string, string][],
      ).toString();
      return apiFetch<Task[]>(`/projects/${projectId}/tasks${qs ? `?${qs}` : ""}`);
    },
    create: (
      projectId: string,
      input: {
        title: string;
        description?: string;
        priority?: Priority;
        assignee_id?: string | null;
        due_date?: string | null;
      },
    ) => apiFetch<Task>(`/projects/${projectId}/tasks`, { method: "POST", body: json(input) }),
    get: (taskId: string) => apiFetch<TaskDetail>(`/tasks/${taskId}`),
    update: (
      taskId: string,
      patch: Partial<{
        title: string;
        description: string | null;
        status: TaskStatus;
        priority: Priority;
        assignee_id: string | null;
        due_date: string | null;
      }>,
    ) => apiFetch<Task>(`/tasks/${taskId}`, { method: "PATCH", body: json(patch) }),
    remove: (taskId: string) => apiFetch<void>(`/tasks/${taskId}`, { method: "DELETE" }),
    setTags: (taskId: string, tagIds: string[]) =>
      apiFetch<Tag[]>(`/tasks/${taskId}/tags`, {
        method: "PUT",
        body: json({ tag_ids: tagIds }),
      }),
  },

  comments: {
    list: (taskId: string) => apiFetch<Comment[]>(`/tasks/${taskId}/comments`),
    create: (taskId: string, body: string) =>
      apiFetch<Comment>(`/tasks/${taskId}/comments`, { method: "POST", body: json({ body }) }),
    update: (taskId: string, commentId: string, body: string) =>
      apiFetch<Comment>(`/tasks/${taskId}/comments/${commentId}`, {
        method: "PATCH",
        body: json({ body }),
      }),
    remove: (taskId: string, commentId: string) =>
      apiFetch<void>(`/tasks/${taskId}/comments/${commentId}`, { method: "DELETE" }),
  },

  attachments: {
    list: (taskId: string) => apiFetch<Attachment[]>(`/tasks/${taskId}/attachments`),
    upload: (taskId: string, file: File) => {
      const form = new FormData();
      form.append("file", file);
      return apiFetch<Attachment>(`/tasks/${taskId}/attachments`, {
        method: "POST",
        body: form,
      });
    },
    remove: (taskId: string, attachmentId: string) =>
      apiFetch<void>(`/tasks/${taskId}/attachments/${attachmentId}`, { method: "DELETE" }),
    async download(taskId: string, attachmentId: string, fileName: string) {
      const headers = new Headers();
      if (accessToken) headers.set("Authorization", `Bearer ${accessToken}`);
      const res = await fetch(`/api/tasks/${taskId}/attachments/${attachmentId}`, {
        headers,
        credentials: "include",
      });
      if (!res.ok) throw new ApiError(res.status, "download failed");
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = fileName;
      a.click();
      URL.revokeObjectURL(url);
    },
  },

  admin: {
    orgs: () => apiFetch<Org[]>("/admin/orgs"),
    users: () => apiFetch<User[]>("/admin/users"),
    updateUserRole: (userId: string, instanceRole: "user" | "superadmin") =>
      apiFetch<User>(`/admin/users/${userId}`, {
        method: "PATCH",
        body: json({ instance_role: instanceRole }),
      }),
  },
};
