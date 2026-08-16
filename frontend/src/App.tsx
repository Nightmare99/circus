import { Navigate, Route, Routes } from "react-router-dom";
import { useAuth } from "./lib/auth";
import { Spinner } from "./components/ui";
import AppShell from "./components/AppShell";
import LoginPage from "./pages/LoginPage";
import RegisterPage from "./pages/RegisterPage";
import AcceptInvitePage from "./pages/AcceptInvitePage";
import OrgListPage from "./pages/OrgListPage";
import OrgDashboardPage from "./pages/OrgDashboardPage";
import OrgSettingsPage from "./pages/OrgSettingsPage";
import ProjectBoardPage from "./pages/ProjectBoardPage";
import AdminPage from "./pages/AdminPage";

function RequireAuth({ children }: { children: React.ReactElement }) {
  const { user, loading } = useAuth();
  if (loading) {
    return (
      <div className="flex h-screen items-center justify-center">
        <Spinner className="h-6 w-6 text-accent" />
      </div>
    );
  }
  if (!user) return <Navigate to="/login" replace />;
  return children;
}

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route path="/register" element={<RegisterPage />} />
      <Route path="/accept-invite" element={<AcceptInvitePage />} />
      <Route
        element={
          <RequireAuth>
            <AppShell />
          </RequireAuth>
        }
      >
        <Route path="/" element={<OrgListPage />} />
        <Route path="/orgs/:orgId" element={<OrgDashboardPage />} />
        <Route path="/orgs/:orgId/settings" element={<OrgSettingsPage />} />
        <Route path="/orgs/:orgId/projects/:projectId" element={<ProjectBoardPage />} />
        <Route path="/admin" element={<AdminPage />} />
      </Route>
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}
