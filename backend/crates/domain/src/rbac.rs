use serde::{Deserialize, Serialize};

/// Instance-wide role. `Superadmin` exists outside any organization and can
/// manage orgs/users at the deployment level (bootstrap, support, etc).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "instance_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum InstanceRole {
    User,
    Superadmin,
}

/// Role within a single organization (tenant).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "org_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrgRole {
    Member,
    Admin,
    Owner,
}

/// Role within a single project, scoped to a member of that project's org.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "project_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ProjectRole {
    Viewer,
    Contributor,
    Lead,
}

/// Actions gated by RBAC. Kept as an explicit enum (rather than ad-hoc
/// string checks scattered through handlers) so every permission decision
/// goes through one place and is easy to audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    // Org-level
    ManageOrgSettings,
    ManageOrgMembers,
    CreateProject,
    DeleteProject,
    // Project-level
    ViewBoard,
    EditTask,
    DeleteTask,
    ManageProjectMembers,
}

impl OrgRole {
    pub fn can(self, action: Action) -> bool {
        match action {
            Action::ManageOrgSettings | Action::DeleteProject => self >= OrgRole::Owner,
            Action::ManageOrgMembers | Action::CreateProject => self >= OrgRole::Admin,
            _ => false, // project-level actions are decided by ProjectRole
        }
    }
}

impl ProjectRole {
    pub fn can(self, action: Action) -> bool {
        match action {
            Action::ViewBoard => true, // any project member, including Viewer
            Action::EditTask => self >= ProjectRole::Contributor,
            Action::DeleteTask | Action::ManageProjectMembers => self >= ProjectRole::Lead,
            _ => false, // org-level actions are decided by OrgRole
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn org_admin_can_manage_members_but_not_org_settings() {
        assert!(OrgRole::Admin.can(Action::ManageOrgMembers));
        assert!(!OrgRole::Admin.can(Action::ManageOrgSettings));
        assert!(OrgRole::Owner.can(Action::ManageOrgSettings));
    }

    #[test]
    fn project_viewer_is_read_only() {
        assert!(ProjectRole::Viewer.can(Action::ViewBoard));
        assert!(!ProjectRole::Viewer.can(Action::EditTask));
        assert!(ProjectRole::Contributor.can(Action::EditTask));
        assert!(!ProjectRole::Contributor.can(Action::DeleteTask));
        assert!(ProjectRole::Lead.can(Action::DeleteTask));
    }
}
