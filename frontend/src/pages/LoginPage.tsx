import { useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { useAuth } from "../lib/auth";
import { Button, ErrorBanner, Input, Label } from "../components/ui";
import AuthLayout from "./AuthLayout";

export default function LoginPage() {
  const { login } = useAuth();
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const next = params.get("next") || "/";
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await login(email, password);
      navigate(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Login failed");
    } finally {
      setBusy(false);
    }
  };

  return (
    <AuthLayout title="Sign in">
      <form onSubmit={submit} className="space-y-4">
        <div>
          <Label>Email</Label>
          <Input
            type="email"
            autoFocus
            required
            value={email}
            onChange={(e) => setEmail(e.target.value)}
          />
        </div>
        <div>
          <Label>Password</Label>
          <Input
            type="password"
            required
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        </div>
        {error && <ErrorBanner message={error} />}
        <Button type="submit" className="w-full" disabled={busy}>
          Sign in
        </Button>
      </form>
      <p className="mt-4 text-center text-sm text-text-faint">
        No account?{" "}
        <Link
          to={`/register${next !== "/" ? `?next=${encodeURIComponent(next)}` : ""}`}
          className="text-accent hover:underline"
        >
          Create one
        </Link>
      </p>
    </AuthLayout>
  );
}
