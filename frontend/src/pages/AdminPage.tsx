import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useAuth } from "../lib/auth";
import { Avatar, Button, Spinner } from "../components/ui";

export default function AdminPage() {
  const { user } = useAuth();
  const queryClient = useQueryClient();
  const usersQuery = useQuery({ queryKey: ["admin-users"], queryFn: api.admin.users });
  const orgsQuery = useQuery({ queryKey: ["admin-orgs"], queryFn: api.admin.orgs });

  if (user?.instance_role !== "superadmin") {
    return (
      <div className="p-8 text-sm text-text-dim">You don't have access to this page.</div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-8">
      <div className="mx-auto max-w-3xl space-y-8">
        <div>
          <h1 className="text-lg font-semibold">Instance admin</h1>
          <p className="text-sm text-text-faint">
            Manage users and organizations across this Circus instance.
          </p>
        </div>

        <section>
          <h2 className="mb-3 text-sm font-semibold text-text-dim">
            Users ({usersQuery.data?.length ?? 0})
          </h2>
          {usersQuery.isLoading ? (
            <Spinner className="text-accent" />
          ) : (
            <ul className="divide-y divide-border rounded-lg border border-border">
              {usersQuery.data?.map((u) => (
                <li key={u.id} className="flex items-center gap-3 px-4 py-2.5">
                  <Avatar name={u.display_name} />
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm text-text">{u.display_name}</p>
                    <p className="truncate text-xs text-text-faint">{u.email}</p>
                  </div>
                  {u.instance_role === "superadmin" && (
                    <span className="rounded-full bg-accent/15 px-2 py-0.5 text-xs font-medium text-accent">
                      Superadmin
                    </span>
                  )}
                  <Button
                    variant="ghost"
                    onClick={async () => {
                      await api.admin.updateUserRole(
                        u.id,
                        u.instance_role === "superadmin" ? "user" : "superadmin",
                      );
                      queryClient.invalidateQueries({ queryKey: ["admin-users"] });
                    }}
                  >
                    {u.instance_role === "superadmin" ? "Revoke" : "Make superadmin"}
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </section>

        <section>
          <h2 className="mb-3 text-sm font-semibold text-text-dim">
            Organizations ({orgsQuery.data?.length ?? 0})
          </h2>
          <ul className="divide-y divide-border rounded-lg border border-border">
            {orgsQuery.data?.map((o) => (
              <li key={o.id} className="px-4 py-2.5">
                <p className="text-sm text-text">{o.name}</p>
                <p className="font-mono text-xs text-text-faint">{o.slug}</p>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </div>
  );
}
