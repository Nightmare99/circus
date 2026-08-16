import { useState } from "react";
import { useParams } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useAuth } from "../lib/auth";
import type { OrgRole } from "../lib/types";
import { Avatar, Button, ErrorBanner, Input, Select, Spinner } from "../components/ui";

export default function OrgSettingsPage() {
  const { orgId } = useParams();
  const { user } = useAuth();
  const queryClient = useQueryClient();
  const [inviteEmail, setInviteEmail] = useState("");
  const [inviteRole, setInviteRole] = useState<OrgRole>("member");
  const [lastInviteLink, setLastInviteLink] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const membersQuery = useQuery({
    queryKey: ["org-members", orgId],
    queryFn: () => api.orgs.members(orgId!),
    enabled: !!orgId,
  });
  const invitesQuery = useQuery({
    queryKey: ["org-invites", orgId],
    queryFn: () => api.orgs.invites(orgId!),
    enabled: !!orgId,
  });

  const myRole = membersQuery.data?.find((m) => m.user_id === user?.id)?.role;
  const canManage = myRole === "admin" || myRole === "owner";

  if (!orgId) return null;

  const invalidateMembers = () => queryClient.invalidateQueries({ queryKey: ["org-members", orgId] });

  const sendInvite = async () => {
    if (!inviteEmail.trim()) return;
    setError(null);
    try {
      const invite = await api.orgs.createInvite(orgId, inviteEmail.trim(), inviteRole);
      const link = `${window.location.origin}/accept-invite?token=${invite.token}`;
      setLastInviteLink(link);
      setInviteEmail("");
      queryClient.invalidateQueries({ queryKey: ["org-invites", orgId] });
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to create invite");
    }
  };

  return (
    <div className="h-full overflow-y-auto p-8">
      <div className="mx-auto max-w-3xl space-y-8">
        <div>
          <h1 className="text-lg font-semibold">Members &amp; invites</h1>
          <p className="text-sm text-text-faint">Manage who has access to this organization.</p>
        </div>

        <section>
          <h2 className="mb-3 text-sm font-semibold text-text-dim">Members</h2>
          {membersQuery.isLoading ? (
            <Spinner className="text-accent" />
          ) : (
            <ul className="divide-y divide-border rounded-lg border border-border">
              {membersQuery.data?.map((m) => (
                <li key={m.user_id} className="flex items-center gap-3 px-4 py-2.5">
                  <Avatar name={m.display_name} />
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm text-text">{m.display_name}</p>
                    <p className="truncate text-xs text-text-faint">{m.email}</p>
                  </div>
                  {canManage ? (
                    <div className="flex items-center gap-2">
                      <Select
                        value={m.role}
                        onChange={async (e) => {
                          await api.orgs.updateMemberRole(orgId, m.user_id, e.target.value as OrgRole);
                          invalidateMembers();
                        }}
                      >
                        <option value="member">Member</option>
                        <option value="admin">Admin</option>
                        <option value="owner">Owner</option>
                      </Select>
                      {m.user_id !== user?.id && (
                        <Button
                          variant="ghost"
                          onClick={async () => {
                            await api.orgs.removeMember(orgId, m.user_id);
                            invalidateMembers();
                          }}
                        >
                          Remove
                        </Button>
                      )}
                    </div>
                  ) : (
                    <span className="text-xs text-text-faint capitalize">{m.role}</span>
                  )}
                </li>
              ))}
            </ul>
          )}
        </section>

        {canManage && (
          <section>
            <h2 className="mb-3 text-sm font-semibold text-text-dim">Invite someone</h2>
            <div className="flex items-end gap-2">
              <div className="flex-1">
                <Input
                  type="email"
                  placeholder="teammate@example.com"
                  value={inviteEmail}
                  onChange={(e) => setInviteEmail(e.target.value)}
                />
              </div>
              <Select value={inviteRole} onChange={(e) => setInviteRole(e.target.value as OrgRole)}>
                <option value="member">Member</option>
                <option value="admin">Admin</option>
                <option value="owner">Owner</option>
              </Select>
              <Button onClick={sendInvite} disabled={!inviteEmail.trim()}>
                Create invite
              </Button>
            </div>
            {error && (
              <div className="mt-2">
                <ErrorBanner message={error} />
              </div>
            )}
            {lastInviteLink && (
              <div className="mt-3 rounded-md border border-accent/40 bg-accent/10 p-3 text-sm">
                <p className="mb-1 text-text-dim">
                  Circus doesn't send email — share this link with the invitee:
                </p>
                <code className="block truncate font-mono text-xs text-accent">{lastInviteLink}</code>
              </div>
            )}

            <h3 className="mt-6 mb-2 text-xs font-medium tracking-wide text-text-faint uppercase">
              Pending invites
            </h3>
            <ul className="divide-y divide-border rounded-lg border border-border">
              {invitesQuery.data?.map((inv) => (
                <li key={inv.id} className="flex items-center justify-between px-4 py-2.5">
                  <div>
                    <p className="text-sm text-text">{inv.email}</p>
                    <p className="text-xs text-text-faint capitalize">{inv.role}</p>
                  </div>
                  <Button
                    variant="ghost"
                    onClick={async () => {
                      await api.orgs.revokeInvite(orgId, inv.id);
                      queryClient.invalidateQueries({ queryKey: ["org-invites", orgId] });
                    }}
                  >
                    Revoke
                  </Button>
                </li>
              ))}
              {invitesQuery.data?.length === 0 && (
                <li className="px-4 py-3 text-sm text-text-faint">No pending invites</li>
              )}
            </ul>
          </section>
        )}
      </div>
    </div>
  );
}
