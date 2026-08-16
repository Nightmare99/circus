import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { api } from "../lib/api";
import { EmptyState, Spinner } from "../components/ui";

export default function OrgListPage() {
  const { data: orgs, isLoading } = useQuery({ queryKey: ["orgs"], queryFn: api.orgs.list });

  return (
    <div className="mx-auto max-w-2xl p-8">
      <h1 className="mb-1 text-lg font-semibold">Your organizations</h1>
      <p className="mb-6 text-sm text-text-faint">Pick one to see its projects and boards.</p>

      {isLoading && (
        <div className="flex justify-center py-12">
          <Spinner className="text-accent" />
        </div>
      )}

      {!isLoading && orgs?.length === 0 && (
        <EmptyState
          title="You're not part of any organization yet"
          hint="Create one from the sidebar, or ask a teammate to invite you."
        />
      )}

      <ul className="space-y-2">
        {orgs?.map((org) => (
          <li key={org.id}>
            <Link
              to={`/orgs/${org.id}`}
              className="block rounded-lg border border-border bg-surface px-4 py-3 hover:border-border-strong hover:bg-surface-2"
            >
              <p className="font-medium text-text">{org.name}</p>
              <p className="font-mono text-xs text-text-faint">{org.slug}</p>
            </Link>
          </li>
        ))}
      </ul>
    </div>
  );
}
