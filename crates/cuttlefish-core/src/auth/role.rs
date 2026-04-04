//! Role-based access control for projects.

use thiserror::Error;

/// Project roles in order of decreasing privilege.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// Full control, can transfer ownership.
    Owner = 4,
    /// Can manage members and settings.
    Admin = 3,
    /// Can read and write project content.
    Member = 2,
    /// Read-only access.
    Viewer = 1,
}

impl Role {
    /// Parse role from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "owner" => Some(Role::Owner),
            "admin" => Some(Role::Admin),
            "member" => Some(Role::Member),
            "viewer" => Some(Role::Viewer),
            _ => None,
        }
    }

    /// Convert role to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Owner => "owner",
            Role::Admin => "admin",
            Role::Member => "member",
            Role::Viewer => "viewer",
        }
    }

    /// Check if this role has at least the privileges of another role.
    pub fn has_at_least(&self, other: Role) -> bool {
        *self >= other
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Actions that can be performed on a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// View project content.
    View,
    /// Edit project content.
    Edit,
    /// Run builds and tests.
    Execute,
    /// Manage project settings.
    ManageSettings,
    /// Invite new members.
    InviteMembers,
    /// Remove members.
    RemoveMembers,
    /// Change member roles (except owner).
    ChangeRoles,
    /// Delete the project.
    Delete,
    /// Transfer ownership.
    TransferOwnership,
}

impl Action {
    /// Get the minimum role required for this action.
    pub fn minimum_role(&self) -> Role {
        match self {
            Action::View => Role::Viewer,
            Action::Edit | Action::Execute => Role::Member,
            Action::ManageSettings | Action::InviteMembers => Role::Admin,
            Action::RemoveMembers | Action::ChangeRoles => Role::Admin,
            Action::Delete | Action::TransferOwnership => Role::Owner,
        }
    }
}

/// Check if a role can perform an action.
pub fn can_perform(role: Role, action: Action) -> bool {
    role.has_at_least(action.minimum_role())
}

/// Role-related errors.
#[derive(Error, Debug)]
pub enum RoleError {
    /// Cannot remove the last owner.
    #[error("Cannot remove the last owner of a project")]
    CannotRemoveLastOwner,

    /// Cannot promote self to owner.
    #[error("Cannot promote yourself to owner")]
    CannotSelfPromoteToOwner,

    /// Insufficient permissions.
    #[error("Insufficient permissions: {role} cannot perform {action}")]
    InsufficientPermissions {
        /// The user's role.
        role: String,
        /// The attempted action.
        action: String,
    },

    /// Invalid role.
    #[error("Invalid role: {0}")]
    InvalidRole(String),

    /// User not a member.
    #[error("User is not a member of this project")]
    NotAMember,
}

/// Validate a role change operation.
pub fn validate_role_change(
    actor_role: Role,
    target_current_role: Role,
    new_role: Role,
    is_self: bool,
    owner_count: i64,
) -> Result<(), RoleError> {
    if is_self && new_role == Role::Owner {
        return Err(RoleError::CannotSelfPromoteToOwner);
    }

    if target_current_role == Role::Owner && new_role != Role::Owner && owner_count <= 1 {
        return Err(RoleError::CannotRemoveLastOwner);
    }

    if new_role == Role::Owner && actor_role != Role::Owner {
        return Err(RoleError::InsufficientPermissions {
            role: actor_role.to_string(),
            action: "promote to owner".to_string(),
        });
    }

    if !can_perform(actor_role, Action::ChangeRoles) {
        return Err(RoleError::InsufficientPermissions {
            role: actor_role.to_string(),
            action: "change roles".to_string(),
        });
    }

    if target_current_role >= actor_role && actor_role != Role::Owner {
        return Err(RoleError::InsufficientPermissions {
            role: actor_role.to_string(),
            action: format!("modify {} role", target_current_role),
        });
    }

    Ok(())
}

/// Validate a member removal operation.
pub fn validate_member_removal(
    actor_role: Role,
    target_role: Role,
    is_self: bool,
    owner_count: i64,
) -> Result<(), RoleError> {
    if target_role == Role::Owner && owner_count <= 1 {
        return Err(RoleError::CannotRemoveLastOwner);
    }

    if is_self {
        return Ok(());
    }

    if !can_perform(actor_role, Action::RemoveMembers) {
        return Err(RoleError::InsufficientPermissions {
            role: actor_role.to_string(),
            action: "remove members".to_string(),
        });
    }

    if target_role >= actor_role && actor_role != Role::Owner {
        return Err(RoleError::InsufficientPermissions {
            role: actor_role.to_string(),
            action: format!("remove {}", target_role),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_ordering() {
        assert!(Role::Owner > Role::Admin);
        assert!(Role::Admin > Role::Member);
        assert!(Role::Member > Role::Viewer);
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::parse("owner"), Some(Role::Owner));
        assert_eq!(Role::parse("ADMIN"), Some(Role::Admin));
        assert_eq!(Role::parse("Member"), Some(Role::Member));
        assert_eq!(Role::parse("viewer"), Some(Role::Viewer));
        assert_eq!(Role::parse("invalid"), None);
    }

    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::Owner.as_str(), "owner");
        assert_eq!(Role::Admin.as_str(), "admin");
        assert_eq!(Role::Member.as_str(), "member");
        assert_eq!(Role::Viewer.as_str(), "viewer");
    }

    #[test]
    fn test_role_has_at_least() {
        assert!(Role::Owner.has_at_least(Role::Viewer));
        assert!(Role::Owner.has_at_least(Role::Owner));
        assert!(Role::Admin.has_at_least(Role::Member));
        assert!(!Role::Viewer.has_at_least(Role::Member));
    }

    #[test]
    fn test_can_perform_viewer() {
        assert!(can_perform(Role::Viewer, Action::View));
        assert!(!can_perform(Role::Viewer, Action::Edit));
        assert!(!can_perform(Role::Viewer, Action::Delete));
    }

    #[test]
    fn test_can_perform_member() {
        assert!(can_perform(Role::Member, Action::View));
        assert!(can_perform(Role::Member, Action::Edit));
        assert!(can_perform(Role::Member, Action::Execute));
        assert!(!can_perform(Role::Member, Action::InviteMembers));
    }

    #[test]
    fn test_can_perform_admin() {
        assert!(can_perform(Role::Admin, Action::View));
        assert!(can_perform(Role::Admin, Action::Edit));
        assert!(can_perform(Role::Admin, Action::InviteMembers));
        assert!(can_perform(Role::Admin, Action::RemoveMembers));
        assert!(!can_perform(Role::Admin, Action::Delete));
    }

    #[test]
    fn test_can_perform_owner() {
        assert!(can_perform(Role::Owner, Action::View));
        assert!(can_perform(Role::Owner, Action::Delete));
        assert!(can_perform(Role::Owner, Action::TransferOwnership));
    }

    #[test]
    fn test_validate_role_change_self_promote_to_owner() {
        let result = validate_role_change(Role::Admin, Role::Admin, Role::Owner, true, 1);
        assert!(matches!(result, Err(RoleError::CannotSelfPromoteToOwner)));
    }

    #[test]
    fn test_validate_role_change_remove_last_owner() {
        let result = validate_role_change(Role::Owner, Role::Owner, Role::Admin, false, 1);
        assert!(matches!(result, Err(RoleError::CannotRemoveLastOwner)));
    }

    #[test]
    fn test_validate_role_change_non_owner_promote_to_owner() {
        let result = validate_role_change(Role::Admin, Role::Member, Role::Owner, false, 1);
        assert!(matches!(
            result,
            Err(RoleError::InsufficientPermissions { .. })
        ));
    }

    #[test]
    fn test_validate_role_change_owner_can_promote() {
        let result = validate_role_change(Role::Owner, Role::Admin, Role::Owner, false, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_role_change_admin_can_change_member() {
        let result = validate_role_change(Role::Admin, Role::Member, Role::Viewer, false, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_role_change_admin_cannot_change_admin() {
        let result = validate_role_change(Role::Admin, Role::Admin, Role::Member, false, 1);
        assert!(matches!(
            result,
            Err(RoleError::InsufficientPermissions { .. })
        ));
    }

    #[test]
    fn test_validate_member_removal_last_owner() {
        let result = validate_member_removal(Role::Owner, Role::Owner, false, 1);
        assert!(matches!(result, Err(RoleError::CannotRemoveLastOwner)));
    }

    #[test]
    fn test_validate_member_removal_self_leave() {
        let result = validate_member_removal(Role::Member, Role::Member, true, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_member_removal_admin_remove_member() {
        let result = validate_member_removal(Role::Admin, Role::Member, false, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_member_removal_admin_cannot_remove_admin() {
        let result = validate_member_removal(Role::Admin, Role::Admin, false, 1);
        assert!(matches!(
            result,
            Err(RoleError::InsufficientPermissions { .. })
        ));
    }

    #[test]
    fn test_validate_member_removal_owner_can_remove_anyone() {
        let result = validate_member_removal(Role::Owner, Role::Admin, false, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_role_display() {
        assert_eq!(format!("{}", Role::Owner), "owner");
        assert_eq!(format!("{}", Role::Admin), "admin");
    }
}
