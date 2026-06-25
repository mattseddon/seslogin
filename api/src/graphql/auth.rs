use anyhow::Result;
use anyhow::anyhow;
use async_graphql::Context;
use async_graphql::Guard;

use crate::auth::AuthInfo;

#[derive(Eq, PartialEq, Copy, Clone)]
pub(crate) enum AuthRequirement {
    Session,
    /// Any authenticated principal: user, session, or API token.
    Authenticated,
    User,
    /// A user or an API token, but not a kiosk session.
    UserOrApiToken,
    SuperUser,
}

pub(crate) struct AuthGuard {
    requirement: AuthRequirement,
}

impl AuthGuard {
    pub(crate) fn new(requirement: AuthRequirement) -> Self {
        Self { requirement }
    }
}

impl Guard for AuthGuard {
    async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
        let auth = ctx.data_opt::<AuthInfo>();
        match self.requirement {
            AuthRequirement::Session => {
                if match auth {
                    Some(AuthInfo::Session { .. }) => true,
                    Some(AuthInfo::ApiToken { .. }) => true,
                    Some(AuthInfo::User { .. }) => false,
                    None => false,
                } {
                    Ok(())
                } else {
                    Err("Must provide session token".into())
                }
            }
            AuthRequirement::Authenticated => {
                if auth.is_some() {
                    Ok(())
                } else {
                    Err("Must be authenticated".into())
                }
            }
            AuthRequirement::User => {
                if match auth {
                    Some(AuthInfo::User { .. }) => true,
                    Some(AuthInfo::ApiToken { .. }) => false,
                    Some(AuthInfo::Session { .. }) => false,
                    None => false,
                } {
                    Ok(())
                } else {
                    Err("Must provide user token".into())
                }
            }
            AuthRequirement::UserOrApiToken => {
                if match auth {
                    Some(AuthInfo::User { .. }) => true,
                    Some(AuthInfo::ApiToken { .. }) => true,
                    Some(AuthInfo::Session { .. }) => false,
                    None => false,
                } {
                    Ok(())
                } else {
                    Err("Must provide user or API token".into())
                }
            }
            AuthRequirement::SuperUser => {
                if match auth {
                    Some(AuthInfo::User { is_super, .. }) => *is_super,
                    Some(AuthInfo::Session { .. }) => false,
                    Some(AuthInfo::ApiToken { .. }) => false,
                    None => false,
                } {
                    Ok(())
                } else {
                    Err("Must provide super user token".into())
                }
            }
        }
    }
}

/// Check the caller is allowed to act on the given location:
/// - super users bypass
/// - regular users must have the location in `location_grants`
/// - sessions must be bound to the same location
/// - api tokens must have the location in their per-token `location_grants`
pub(crate) fn require_location_access(ctx: &Context<'_>, location_id: &str) -> Result<()> {
    match ctx.data_opt::<AuthInfo>() {
        Some(AuthInfo::User { is_super: true, .. }) => Ok(()),
        Some(AuthInfo::User {
            location_grants, ..
        }) if location_grants.iter().any(|g| g == location_id) => Ok(()),
        Some(AuthInfo::Session { location, .. }) if location == location_id => Ok(()),
        Some(AuthInfo::ApiToken {
            location_grants, ..
        }) if location_grants.iter().any(|g| g == location_id) => Ok(()),
        _ => Err(anyhow!("Not authorized for this location")),
    }
}

/// Reject read-only API tokens. Called at the top of every mutation resolver.
/// User/Session callers always pass; API tokens pass only if `read_only` is false.
pub(crate) fn require_writable(ctx: &Context<'_>) -> Result<()> {
    match ctx.data_opt::<AuthInfo>() {
        Some(AuthInfo::ApiToken {
            read_only: true, ..
        }) => Err(anyhow!("API token is read-only")),
        _ => Ok(()),
    }
}
