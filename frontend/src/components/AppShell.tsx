import { useState } from "react";
import { Link, Outlet, useNavigate, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useAuth } from "../lib/auth";
import { Avatar, Button, Input, Modal } from "./ui";

export default function AppShell() {
  const { orgId } = useParams();
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const [creatingOrg, setCreatingOrg] = useState(false);

  const orgsQuery = useQuery({ queryKey: ["orgs"], queryFn: api.orgs.list });
  const projectsQuery = useQuery({
    queryKey: ["projects", orgId],
    queryFn: () => api.projects.list(orgId!),
    enabled: !!orgId,
  });

  const currentOrg = orgsQuery.data?.find((o) => o.id === orgId);

  return (
    <div className="flex h-screen bg-bg text-text">
      <aside className="flex w-60 shrink-0 flex-col border-r border-border bg-surface">
        <div className="border-b border-border">
          <Link to="/" className="block px-4 py-4">
            <span className="font-mono text-sm font-bold tracking-tight text-accent">
              circus
            </span>
          </Link>
          <div className="marquee-underline" />
        </div>

        <div className="border-b border-border p-3">
          <select
            className="w-full rounded-md border border-border-strong bg-surface-2 px-2 py-1.5 text-sm text-text"
            value={orgId ?? ""}
            onChange={(e) => {
              if (e.target.value === "__new__") setCreatingOrg(true);
              else navigate(`/orgs/${e.target.value}`);
            }}
          >
            <option value="" disabled>
              Select organization
            </option>
            {orgsQuery.data?.map((o) => (
              <option key={o.id} value={o.id}>
                {o.name}
              </option>
            ))}
            <option value="__new__">+ New organization…</option>
          </select>
        </div>

        {orgId && (
          <>
            <nav className="flex-1 overflow-y-auto p-3">
              <div className="mb-1 flex items-center justify-between px-1">
                <span className="text-xs font-medium tracking-wide text-text-faint uppercase">
                  Projects
                </span>
              </div>
              <ul className="space-y-0.5">
                {projectsQuery.data?.map((p) => (
                  <li key={p.id}>
                    <Link
                      to={`/orgs/${orgId}/projects/${p.id}`}
                      className="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm text-text-dim hover:bg-surface-2 hover:text-text"
                    >
                      <span className="font-mono text-xs text-text-faint">{p.key}</span>
                      <span className="truncate">{p.name}</span>
                    </Link>
                  </li>
                ))}
                {projectsQuery.data?.length === 0 && (
                  <li className="px-2 py-1 text-xs text-text-faint">No projects yet</li>
                )}
              </ul>
            </nav>
            <div className="border-t border-border p-3">
              <Link
                to={`/orgs/${orgId}/settings`}
                className="block rounded-md px-2 py-1.5 text-sm text-text-dim hover:bg-surface-2 hover:text-text"
              >
                Members &amp; invites
              </Link>
              {currentOrg && (
                <p className="mt-1 truncate px-2 text-xs text-text-faint">{currentOrg.slug}</p>
              )}
            </div>
          </>
        )}

        {user?.instance_role === "superadmin" && (
          <div className="border-t border-border p-3">
            <Link
              to="/admin"
              className="block rounded-md px-2 py-1.5 text-sm text-text-dim hover:bg-surface-2 hover:text-text"
            >
              Instance admin
            </Link>
          </div>
        )}

        <div className="flex items-center gap-2 border-t border-border p-3">
          {user && <Avatar name={user.display_name} />}
          <div className="min-w-0 flex-1">
            <p className="truncate text-sm text-text">{user?.display_name}</p>
          </div>
          <button
            onClick={() => logout()}
            className="text-xs text-text-faint hover:text-text"
          >
            Sign out
          </button>
        </div>
      </aside>

      <main className="min-w-0 flex-1 overflow-hidden">
        <Outlet />
      </main>

      {creatingOrg && (
        <CreateOrgModal
          onClose={() => setCreatingOrg(false)}
          onCreated={(org) => {
            setCreatingOrg(false);
            orgsQuery.refetch();
            navigate(`/orgs/${org.id}`);
          }}
        />
      )}
    </div>
  );
}

function CreateOrgModal({
  onClose,
  onCreated,
}: {
  onClose: () => void;
  onCreated: (org: { id: string }) => void;
}) {
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    if (!name.trim()) return;
    setBusy(true);
    setError(null);
    try {
      const org = await api.orgs.create(name.trim());
      onCreated(org);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to create organization");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Modal title="New organization" onClose={onClose}>
      <div className="space-y-3">
        <Input
          autoFocus
          placeholder="Acme Inc"
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && submit()}
        />
        {error && <p className="text-sm text-status-blocked">{error}</p>}
        <div className="flex justify-end gap-2">
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={submit} disabled={busy || !name.trim()}>
            Create
          </Button>
        </div>
      </div>
    </Modal>
  );
}
