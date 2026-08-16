import { useRef, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useAuth } from "../lib/auth";
import { PRIORITY_LABEL, STATUS_LABEL, TASK_STATUSES, type Priority, type ProjectMember, type Tag, type TaskStatus } from "../lib/types";
import { Avatar, Button, Drawer, Select, Spinner, TagChip, Textarea } from "./ui";

function formatBytes(n: number) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

function formatDateTime(iso: string) {
  return new Date(iso).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

export default function TaskDrawer({
  taskId,
  canEdit,
  canManage,
  members,
  membersById,
  tags,
  onClose,
  onChanged,
}: {
  taskId: string;
  canEdit: boolean;
  canManage: boolean;
  members: ProjectMember[];
  membersById: Map<string, string>;
  tags: Tag[];
  onClose: () => void;
  onChanged: () => void;
}) {
  const { user } = useAuth();
  const queryClient = useQueryClient();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [titleDraft, setTitleDraft] = useState<string | null>(null);
  const [descriptionDraft, setDescriptionDraft] = useState<string | null>(null);
  const [newComment, setNewComment] = useState("");
  const [editingCommentId, setEditingCommentId] = useState<string | null>(null);
  const [editingCommentBody, setEditingCommentBody] = useState("");

  const taskQuery = useQuery({ queryKey: ["task", taskId], queryFn: () => api.tasks.get(taskId) });
  const task = taskQuery.data;

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ["task", taskId] });
    onChanged();
  };

  const update = async (patch: Parameters<typeof api.tasks.update>[1]) => {
    await api.tasks.update(taskId, patch);
    invalidate();
  };

  const authorName = (id: string) => membersById.get(id) ?? "Unknown";

  return (
    <Drawer onClose={onClose}>
      {!task ? (
        <div className="flex h-full items-center justify-center">
          <Spinner className="text-accent" />
        </div>
      ) : (
        <div className="flex h-full flex-col">
          <div className="flex items-center justify-between border-b border-border px-5 py-3">
            <span className="font-mono text-xs text-text-faint">#{task.task_number}</span>
            <div className="flex items-center gap-2">
              {canManage && (
                <Button
                  variant="ghost"
                  onClick={async () => {
                    if (!confirm("Delete this task? This cannot be undone.")) return;
                    await api.tasks.remove(taskId);
                    onChanged();
                    onClose();
                  }}
                >
                  Delete
                </Button>
              )}
              <button onClick={onClose} className="text-text-faint hover:text-text" aria-label="Close">
                ×
              </button>
            </div>
          </div>

          <div className="flex-1 overflow-y-auto px-5 py-4">
            <textarea
              className="w-full resize-none rounded-md border border-transparent bg-transparent text-lg font-semibold text-text focus:border-border-strong focus:bg-surface-2 focus:px-2 focus:py-1"
              rows={1}
              value={titleDraft ?? task.title}
              disabled={!canEdit}
              onChange={(e) => setTitleDraft(e.target.value)}
              onBlur={async () => {
                if (titleDraft !== null && titleDraft.trim() && titleDraft !== task.title) {
                  await update({ title: titleDraft.trim() });
                }
                setTitleDraft(null);
              }}
            />

            <div className="mt-4 grid grid-cols-2 gap-3">
              <Field label="Status">
                <Select
                  className="w-full"
                  value={task.status}
                  disabled={!canEdit}
                  onChange={(e) => update({ status: e.target.value as TaskStatus })}
                >
                  {TASK_STATUSES.map((s) => (
                    <option key={s} value={s}>
                      {STATUS_LABEL[s]}
                    </option>
                  ))}
                </Select>
              </Field>
              <Field label="Priority">
                <Select
                  className="w-full"
                  value={task.priority}
                  disabled={!canEdit}
                  onChange={(e) => update({ priority: e.target.value as Priority })}
                >
                  {Object.entries(PRIORITY_LABEL).map(([v, l]) => (
                    <option key={v} value={v}>
                      {l}
                    </option>
                  ))}
                </Select>
              </Field>
              <Field label="Assignee">
                <Select
                  className="w-full"
                  value={task.assignee_id ?? ""}
                  disabled={!canEdit}
                  onChange={(e) => update({ assignee_id: e.target.value || null })}
                >
                  <option value="">Unassigned</option>
                  {members.map((m) => (
                    <option key={m.user_id} value={m.user_id}>
                      {m.display_name}
                    </option>
                  ))}
                </Select>
              </Field>
              <Field label="Due date">
                <input
                  type="date"
                  className="w-full rounded-md border border-border-strong bg-surface-2 px-2 py-1.5 text-sm text-text disabled:opacity-60"
                  value={task.due_date ?? ""}
                  disabled={!canEdit}
                  onChange={(e) => update({ due_date: e.target.value || null })}
                />
              </Field>
            </div>

            <div className="mt-4">
              <p className="mb-1 text-xs font-medium tracking-wide text-text-dim uppercase">
                Reporter
              </p>
              <div className="flex items-center gap-2">
                <Avatar name={authorName(task.reporter_id)} size={20} />
                <span className="text-sm text-text-dim">{authorName(task.reporter_id)}</span>
              </div>
            </div>

            <div className="mt-4">
              <p className="mb-1 text-xs font-medium tracking-wide text-text-dim uppercase">
                Description
              </p>
              <Textarea
                rows={4}
                placeholder={canEdit ? "Add a description…" : "No description"}
                value={descriptionDraft ?? task.description ?? ""}
                disabled={!canEdit}
                onChange={(e) => setDescriptionDraft(e.target.value)}
                onBlur={async () => {
                  if (descriptionDraft !== null && descriptionDraft !== (task.description ?? "")) {
                    await update({ description: descriptionDraft || null });
                  }
                  setDescriptionDraft(null);
                }}
              />
            </div>

            <div className="mt-4">
              <p className="mb-1.5 text-xs font-medium tracking-wide text-text-dim uppercase">Tags</p>
              <div className="flex flex-wrap gap-1.5">
                {tags.map((tag) => {
                  const active = task.tags.some((t) => t.id === tag.id);
                  return (
                    <button
                      key={tag.id}
                      disabled={!canEdit}
                      onClick={async () => {
                        const next = active
                          ? task.tags.filter((t) => t.id !== tag.id).map((t) => t.id)
                          : [...task.tags.map((t) => t.id), tag.id];
                        await api.tasks.setTags(taskId, next);
                        invalidate();
                      }}
                      className="disabled:cursor-default"
                      style={{ opacity: active ? 1 : 0.4 }}
                    >
                      <TagChip name={tag.name} color={tag.color} />
                    </button>
                  );
                })}
                {tags.length === 0 && <span className="text-xs text-text-faint">No tags in this project yet</span>}
              </div>
            </div>

            <div className="mt-6">
              <p className="mb-1.5 text-xs font-medium tracking-wide text-text-dim uppercase">
                Attachments
              </p>
              <ul className="space-y-1.5">
                {task.attachments.map((a) => (
                  <li
                    key={a.id}
                    className="flex items-center justify-between rounded-md border border-border bg-surface-2 px-2.5 py-1.5 text-sm"
                  >
                    <button
                      className="truncate text-left text-text hover:text-accent"
                      onClick={() => api.attachments.download(taskId, a.id, a.file_name)}
                    >
                      {a.file_name}
                    </button>
                    <div className="flex items-center gap-2 text-xs text-text-faint">
                      <span>{formatBytes(a.size_bytes)}</span>
                      {(a.uploaded_by === user?.id || canManage) && (
                        <button
                          className="hover:text-status-blocked"
                          onClick={async () => {
                            await api.attachments.remove(taskId, a.id);
                            invalidate();
                          }}
                        >
                          Remove
                        </button>
                      )}
                    </div>
                  </li>
                ))}
              </ul>
              {canEdit && (
                <>
                  <input
                    ref={fileInputRef}
                    type="file"
                    className="hidden"
                    onChange={async (e) => {
                      const file = e.target.files?.[0];
                      if (!file) return;
                      await api.attachments.upload(taskId, file);
                      e.target.value = "";
                      invalidate();
                    }}
                  />
                  <Button
                    variant="secondary"
                    className="mt-2"
                    onClick={() => fileInputRef.current?.click()}
                  >
                    Attach file
                  </Button>
                </>
              )}
            </div>

            <div className="mt-6">
              <p className="mb-1.5 text-xs font-medium tracking-wide text-text-dim uppercase">
                Comments
              </p>
              <ul className="space-y-3">
                {task.comments.map((c) => (
                  <li key={c.id}>
                    <div className="flex items-center gap-2">
                      <Avatar name={authorName(c.author_id)} size={20} />
                      <span className="text-sm font-medium text-text">{authorName(c.author_id)}</span>
                      <span className="text-xs text-text-faint">{formatDateTime(c.created_at)}</span>
                    </div>
                    {editingCommentId === c.id ? (
                      <div className="mt-1 ml-7 space-y-1.5">
                        <Textarea
                          rows={2}
                          value={editingCommentBody}
                          onChange={(e) => setEditingCommentBody(e.target.value)}
                        />
                        <div className="flex gap-2">
                          <Button
                            onClick={async () => {
                              await api.comments.update(taskId, c.id, editingCommentBody.trim());
                              setEditingCommentId(null);
                              invalidate();
                            }}
                          >
                            Save
                          </Button>
                          <Button variant="ghost" onClick={() => setEditingCommentId(null)}>
                            Cancel
                          </Button>
                        </div>
                      </div>
                    ) : (
                      <p className="mt-1 ml-7 text-sm whitespace-pre-wrap text-text-dim">{c.body}</p>
                    )}
                    {(c.author_id === user?.id || canManage) && editingCommentId !== c.id && (
                      <div className="ml-7 mt-1 flex gap-3 text-xs text-text-faint">
                        {c.author_id === user?.id && (
                          <button
                            className="hover:text-text"
                            onClick={() => {
                              setEditingCommentId(c.id);
                              setEditingCommentBody(c.body);
                            }}
                          >
                            Edit
                          </button>
                        )}
                        <button
                          className="hover:text-status-blocked"
                          onClick={async () => {
                            await api.comments.remove(taskId, c.id);
                            invalidate();
                          }}
                        >
                          Delete
                        </button>
                      </div>
                    )}
                  </li>
                ))}
              </ul>

              <div className="mt-3 space-y-1.5">
                <Textarea
                  rows={2}
                  placeholder="Write a comment…"
                  value={newComment}
                  onChange={(e) => setNewComment(e.target.value)}
                />
                <Button
                  disabled={!newComment.trim()}
                  onClick={async () => {
                    await api.comments.create(taskId, newComment.trim());
                    setNewComment("");
                    invalidate();
                  }}
                >
                  Comment
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </Drawer>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <p className="mb-1 text-xs font-medium tracking-wide text-text-dim uppercase">{label}</p>
      {children}
    </div>
  );
}
