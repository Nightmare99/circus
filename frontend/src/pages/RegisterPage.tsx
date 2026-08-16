import { useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { useAuth } from "../lib/auth";
import { Button, ErrorBanner, Input, Label } from "../components/ui";
import AuthLayout from "./AuthLayout";

export default function RegisterPage() {
  const { register } = useAuth();
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const next = params.get("next") || "/";
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await register(email, password, displayName);
      navigate(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Registration failed");
    } finally {
      setBusy(false);
    }
  };

  return (
    <AuthLayout title="Create your account">
      <form onSubmit={submit} className="space-y-4">
        <div>
          <Label>Name</Label>
          <Input autoFocus required value={displayName} onChange={(e) => setDisplayName(e.target.value)} />
        </div>
        <div>
          <Label>Email</Label>
          <Input type="email" required value={email} onChange={(e) => setEmail(e.target.value)} />
        </div>
        <div>
          <Label>Password</Label>
          <Input
            type="password"
            required
            minLength={8}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
          <p className="mt-1 text-xs text-text-faint">At least 8 characters.</p>
        </div>
        {error && <ErrorBanner message={error} />}
        <Button type="submit" className="w-full" disabled={busy}>
          Create account
        </Button>
      </form>
      <p className="mt-4 text-center text-sm text-text-faint">
        Already have an account?{" "}
        <Link
          to={`/login${next !== "/" ? `?next=${encodeURIComponent(next)}` : ""}`}
          className="text-accent hover:underline"
        >
          Sign in
        </Link>
      </p>
    </AuthLayout>
  );
}
