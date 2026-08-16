import { useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useDraggable,
  useDroppable,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragStartEvent,
} from "@dnd-kit/core";
import { api } from "../lib/api";
import { useAuth } from "../lib/auth";
import { useProjectLiveUpdates } from "../lib/useProjectLiveUpdates";
import { STATUS_LABEL, TASK_STATUSES, type Task, type TaskStatus } from "../lib/types";
import { Avatar, Button, Input, PriorityDot, Select, StatusDot } from "../components/ui";
import TaskDrawer from "../components/TaskDrawer";
import CreateTaskModal from "../components/CreateTaskModal";
import ManageTagsModal from "../components/ManageTagsModal";

export default function ProjectBoardPage() {
  const { orgId, projectId } = useParams();
  const { user } = useAuth();
  const queryClient = useQueryClient();
  useProjectLiveUpdates(projectId);

  const [search, setSearch] = useState("");
  const [assigneeFilter, setAssigneeFilter] = useState("");
  const [tagFilter, setTagFilter] = useState("");
  const [openTaskId, setOpenTaskId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [managingTags, setManagingTags] = useState(false);
  const [activeTask, setActiveTask] = useState<Task | null>(null);

  const projectQuery = useQuery({
    queryKey: ["project", projectId],
    queryFn: () => api.projects.get(projectId!),
    enabled: !!projectId,
  });
  const orgMembersQuery = useQuery({
    queryKey: ["org-members", orgId],
    queryFn: () => api.orgs.members(orgId!),
    enabled: !!orgId,
  });
  const projectMembersQuery = useQuery({
    queryKey: ["project-members", projectId],
    queryFn: () => api.projects.members(projectId!),
    enabled: !!projectId,
  });
  const tagsQuery = useQuery({
    queryKey: ["tags", projectId],
    queryFn: () => api.projects.tags(projectId!),
    enabled: !!projectId,
  });
  const tasksQuery = useQuery({
    queryKey: ["tasks", projectId, search, assigneeFilter, tagFilter],
    queryFn: () =>
      api.tasks.list(projectId!, {
        search: search || undefined,
        assignee_id: assigneeFilter || undefined,
        tag_id: tagFilter || undefined,
      }),
    enabled: !!projectId,
  });

  const myRole = useMemo(() => {
    if (!user) return null;
    if (user.instance_role === "superadmin") return "lead" as const;
    const orgRole = orgMembersQuery.data?.find((m) => m.user_id === user.id)?.role;
    if (orgRole === "owner" || orgRole === "admin") return "lead" as const;
    return projectMembersQuery.data?.find((m) => m.user_id === user.id)?.role ?? null;
  }, [user, orgMembersQuery.data, projectMembersQuery.data]);

  const canEdit = myRole === "contributor" || myRole === "lead";

  const membersById = useMemo(() => {
    const map = new Map<string, string>();
    for (const m of projectMembersQuery.data ?? []) map.set(m.user_id, m.display_name);
    for (const m of orgMembersQuery.data ?? []) if (!map.has(m.user_id)) map.set(m.user_id, m.display_name);
    return map;
  }, [projectMembersQuery.data, orgMembersQuery.data]);

  const tasksByStatus = useMemo(() => {
    const grouped: Record<TaskStatus, Task[]> = {
      pending: [],
      in_progress: [],
      blocked: [],
      completed: [],
    };
    for (const t of tasksQuery.data ?? []) grouped[t.status].push(t);
    return grouped;
  }, [tasksQuery.data]);

  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }));

  const onDragStart = (event: DragStartEvent) => {
    const task = (tasksQuery.data ?? []).find((t) => t.id === event.active.id);
    setActiveTask(task ?? null);
  };

  const onDragEnd = async (event: DragEndEvent) => {
    setActiveTask(null);
    const { active, over } = event;
    if (!over) return;
    const taskId = active.id as string;
    const newStatus = over.id as TaskStatus;
    const task = (tasksQuery.data ?? []).find((t) => t.id === taskId);
    if (!task || task.status === newStatus || !canEdit) return;

    const key = ["tasks", projectId, search, assigneeFilter, tagFilter];
    queryClient.setQueryData<Task[]>(key, (old) =>
      old?.map((t) => (t.id === taskId ? { ...t, status: newStatus } : t)),
    );
    try {
      await api.tasks.update(taskId, { status: newStatus });
    } finally {
      queryClient.invalidateQueries({ queryKey: ["tasks", projectId] });
    }
  };

  if (!projectId || !orgId) return null;

  return (
    <div className="flex h-full flex-col">
      <div className="flex shrink-0 items-center justify-between gap-3 border-b border-border px-6 py-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="rounded bg-surface-3 px-1.5 py-0.5 font-mono text-xs text-text-dim">
              {projectQuery.data?.key}
            </span>
            <h1 className="text-base font-semibold">{projectQuery.data?.name}</h1>
          </div>
        </div>
        {canEdit && <Button onClick={() => setCreating(true)}>New task</Button>}
      </div>

      <div className="flex shrink-0 flex-wrap items-center gap-2 border-b border-border px-6 py-3">
        <Input
          placeholder="Search tasks…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="max-w-xs"
        />
        <Select value={assigneeFilter} onChange={(e) => setAssigneeFilter(e.target.value)}>
          <option value="">Anyone</option>
          {projectMembersQuery.data?.map((m) => (
            <option key={m.user_id} value={m.user_id}>
              {m.display_name}
            </option>
          ))}
        </Select>
        <Select value={tagFilter} onChange={(e) => setTagFilter(e.target.value)}>
          <option value="">Any tag</option>
          {tagsQuery.data?.map((t) => (
            <option key={t.id} value={t.id}>
              {t.name}
            </option>
          ))}
        </Select>
        {(search || assigneeFilter || tagFilter) && (
          <Button
            variant="ghost"
            onClick={() => {
              setSearch("");
              setAssigneeFilter("");
              setTagFilter("");
            }}
          >
            Clear
          </Button>
        )}
        {canEdit && (
          <Button variant="ghost" className="ml-auto" onClick={() => setManagingTags(true)}>
            Manage tags
          </Button>
        )}
      </div>

      <DndContext sensors={sensors} onDragStart={onDragStart} onDragEnd={onDragEnd}>
        <div className="flex flex-1 gap-4 overflow-x-auto p-6">
          {TASK_STATUSES.map((status) => (
            <Column
              key={status}
              status={status}
              tasks={tasksByStatus[status]}
              membersById={membersById}
              onOpenTask={setOpenTaskId}
            />
          ))}
        </div>
        <DragOverlay>
          {activeTask && (
            <TaskCard task={activeTask} membersById={membersById} onOpen={() => {}} dragging />
          )}
        </DragOverlay>
      </DndContext>

      {openTaskId && (
        <TaskDrawer
          taskId={openTaskId}
          canEdit={canEdit}
          canManage={myRole === "lead"}
          members={projectMembersQuery.data ?? []}
          membersById={membersById}
          tags={tagsQuery.data ?? []}
          onClose={() => setOpenTaskId(null)}
          onChanged={() => queryClient.invalidateQueries({ queryKey: ["tasks", projectId] })}
        />
      )}

      {creating && (
        <CreateTaskModal
          projectId={projectId}
          members={projectMembersQuery.data ?? []}
          onClose={() => setCreating(false)}
          onCreated={() => {
            setCreating(false);
            queryClient.invalidateQueries({ queryKey: ["tasks", projectId] });
          }}
        />
      )}

      {managingTags && (
        <ManageTagsModal
          projectId={projectId}
          tags={tagsQuery.data ?? []}
          onClose={() => setManagingTags(false)}
          onChanged={() => queryClient.invalidateQueries({ queryKey: ["tags", projectId] })}
        />
      )}
    </div>
  );
}

function Column({
  status,
  tasks,
  membersById,
  onOpenTask,
}: {
  status: TaskStatus;
  tasks: Task[];
  membersById: Map<string, string>;
  onOpenTask: (id: string) => void;
}) {
  const { setNodeRef, isOver } = useDroppable({ id: status });
  return (
    <div
      ref={setNodeRef}
      className={`flex w-72 shrink-0 flex-col rounded-lg border ${isOver ? "border-accent" : "border-border"} bg-surface/50`}
    >
      <div className="flex items-center gap-2 border-b border-border px-3 py-2.5">
        <StatusDot status={status} />
        <span className="text-sm font-medium text-text">{STATUS_LABEL[status]}</span>
        <span className="ml-auto text-xs text-text-faint">{tasks.length}</span>
      </div>
      <div className="flex-1 space-y-2 overflow-y-auto p-2">
        {tasks.map((task) => (
          <TaskCard key={task.id} task={task} membersById={membersById} onOpen={onOpenTask} />
        ))}
      </div>
    </div>
  );
}

function TaskCard({
  task,
  membersById,
  onOpen,
  dragging = false,
}: {
  task: Task;
  membersById: Map<string, string>;
  onOpen: (id: string) => void;
  dragging?: boolean;
}) {
  const { attributes, listeners, setNodeRef, transform } = useDraggable({ id: task.id });
  const style = transform
    ? { transform: `translate3d(${transform.x}px, ${transform.y}px, 0)` }
    : undefined;
  const assigneeName = task.assignee_id ? membersById.get(task.assignee_id) : undefined;
  const overdue =
    task.due_date && task.status !== "completed" && new Date(task.due_date) < new Date();

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...listeners}
      {...attributes}
      onClick={() => !dragging && onOpen(task.id)}
      className={`cursor-pointer rounded-md border border-border bg-surface p-3 shadow-sm hover:border-border-strong ${dragging ? "opacity-90 shadow-lg" : ""}`}
    >
      <div className="mb-1.5 flex items-center gap-1.5">
        <PriorityDot priority={task.priority} />
        <span className="font-mono text-xs text-text-faint">#{task.task_number}</span>
      </div>
      <p className="mb-2 text-sm text-text">{task.title}</p>
      <div className="flex items-center justify-between">
        <span className={`text-xs ${overdue ? "text-status-blocked" : "text-text-faint"}`}>
          {task.due_date ?? ""}
        </span>
        {assigneeName && <Avatar name={assigneeName} size={20} />}
      </div>
    </div>
  );
}
