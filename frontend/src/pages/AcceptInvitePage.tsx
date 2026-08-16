import { useEffect, useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { useAuth } from "../lib/auth";
import { api } from "../lib/api";
import { Button, ErrorBanner, Spinner } from "../components/ui";
import AuthLayout from "./AuthLayout";

export default function AcceptInvitePage() {
  const [params] = useSearchParams();
  const token = params.get("token") ?? "";
  const { user, loading } = useAuth();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);
  const [accepting, setAccepting] = useState(false);

  useEffect(() => {
    if (loading || !user || !token || accepting) return;
    setAccepting(true);
    api.orgs
      .acceptInvite(token)
      .then((org) => navigate(`/orgs/${org.id}`, { replace: true }))
      .catch((e) => setError(e instanceof Error ? e.message : "Could not accept invite"));
  }, [loading, user, token, accepting, navigate]);

  if (!token) {
    return (
      <AuthLayout title="Invalid invite">
        <ErrorBanner message="This invite link is missing a token." />
      </AuthLayout>
    );
  }

  if (loading) {
    return (
      <AuthLayout title="Accepting invite">
        <div className="flex justify-center py-4">
          <Spinner className="text-accent" />
        </div>
      </AuthLayout>
    );
  }

  if (!user) {
    return (
      <AuthLayout title="Sign in to accept this invite">
        <p className="mb-4 text-sm text-text-dim">
          You need an account to join this organization.
        </p>
        <div className="flex gap-2">
          <Link
            to={`/login?next=${encodeURIComponent(`/accept-invite?token=${token}`)}`}
            className="flex-1"
          >
            <Button className="w-full">Sign in</Button>
          </Link>
          <Link
            to={`/register?next=${encodeURIComponent(`/accept-invite?token=${token}`)}`}
            className="flex-1"
          >
            <Button variant="secondary" className="w-full">
              Register
            </Button>
          </Link>
        </div>
      </AuthLayout>
    );
  }

  return (
    <AuthLayout title="Joining organization">
      {error ? <ErrorBanner message={error} /> : <Spinner className="text-accent" />}
    </AuthLayout>
  );
}
