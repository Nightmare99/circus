import { useState } from "react";
import { useParams, Link } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useAuth } from "../lib/auth";
import { Button, EmptyState, ErrorBanner, Input, Label, Modal, Spinner, Textarea } from "../components/ui";

export default function OrgDashboardPage() {
  const { orgId } = useParams();
  const { user } = useAuth();
  const queryClient = useQueryClient();
  const [creating, setCreating] = useState(false);

  const orgQuery = useQuery({
    queryKey: ["org", orgId],
    queryFn: () => api.orgs.get(orgId!),
    enabled: !!orgId,
  });
  const projectsQuery = useQuery({
    queryKey: ["projects", orgId],
    queryFn: () => api.projects.list(orgId!),
    enabled: !!orgId,
  });
  const membersQuery = useQuery({
    queryKey: ["org-members", orgId],
    queryFn: () => api.orgs.members(orgId!),
    enabled: !!orgId,
  });

  const myRole = membersQuery.data?.find((m) => m.user_id === user?.id)?.role;
  const canCreateProject = myRole === "admin" || myRole === "owner";

  if (!orgId) return null;

  return (
    <div className="h-full overflow-y-auto p-8">
      <div className="mx-auto max-w-4xl">
        <div className="mb-6 flex items-center justify-between">
          <div>
            <h1 className="text-lg font-semibold">{orgQuery.data?.name ?? "…"}</h1>
            <p className="text-sm text-text-faint">Projects</p>
          </div>
          {canCreateProject && (
            <Button onClick={() => setCreating(true)}>New project</Button>
          )}
        </div>

        {projectsQuery.isLoading && (
          <div className="flex justify-center py-12">
            <Spinner className="text-accent" />
          </div>
        )}

        {!projectsQuery.isLoading && projectsQuery.data?.length === 0 && (
          <EmptyState
            title="No projects yet"
            hint={canCreateProject ? "Create the first one to start a board." : "Ask an org admin to create one."}
          />
        )}

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          {projectsQuery.data?.map((p) => (
            <Link
              key={p.id}
              to={`/orgs/${orgId}/projects/${p.id}`}
              className="rounded-lg border border-border bg-surface p-4 hover:border-border-strong hover:bg-surface-2"
            >
              <div className="mb-1 flex items-center gap-2">
                <span className="rounded bg-surface-3 px-1.5 py-0.5 font-mono text-xs text-text-dim">
                  {p.key}
                </span>
                <span className="font-medium text-text">{p.name}</span>
              </div>
              {p.description && (
                <p className="line-clamp-2 text-sm text-text-faint">{p.description}</p>
              )}
            </Link>
          ))}
        </div>
      </div>

      {creating && (
        <CreateProjectModal
          orgId={orgId}
          onClose={() => setCreating(false)}
          onCreated={() => {
            setCreating(false);
            queryClient.invalidateQueries({ queryKey: ["projects", orgId] });
          }}
        />
      )}
    </div>
  );
}

function CreateProjectModal({
  orgId,
  onClose,
  onCreated,
}: {
  orgId: string;
  onClose: () => void;
  onCreated: () => void;
}) {
  const [key, setKey] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async () => {
    if (!key.trim() || !name.trim()) return;
    setBusy(true);
    setError(null);
    try {
      await api.projects.create(orgId, key.trim(), name.trim(), description.trim());
      onCreated();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to create project");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Modal title="New project" onClose={onClose}>
      <div className="space-y-3">
        <div>
          <Label>Key</Label>
          <Input
            autoFocus
            placeholder="ENG"
            value={key}
            onChange={(e) => setKey(e.target.value.toUpperCase())}
            maxLength={10}
          />
          <p className="mt-1 text-xs text-text-faint">
            Short prefix for task IDs, e.g. {key || "ENG"}-1.
          </p>
        </div>
        <div>
          <Label>Name</Label>
          <Input placeholder="Engineering" value={name} onChange={(e) => setName(e.target.value)} />
        </div>
        <div>
          <Label>Description (optional)</Label>
          <Textarea rows={2} value={description} onChange={(e) => setDescription(e.target.value)} />
        </div>
        {error && <ErrorBanner message={error} />}
        <div className="flex justify-end gap-2">
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={submit} disabled={busy || !key.trim() || !name.trim()}>
            Create
          </Button>
        </div>
      </div>
    </Modal>
  );
}
