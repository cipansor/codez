//! Port of `gitea.dev/modules/cachegroup` — CacheGroup names.
//!
//! | upstream | here |
//! | --- | --- |
//! | `cachegroup.go` | this module |

/// `cachegroup` group names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheGroup {
    /// `cachegroup.User`.
    User,
    /// `cachegroup.EmailAvatarLink`.
    EmailAvatarLink,
    /// `cachegroup.UserEmailAddresses`.
    UserEmailAddresses,
    /// `cachegroup.GPGKeyWithSubKeys`.
    GpgKeyWithSubKeys,
    /// `cachegroup.RepoUserPermission`.
    RepoUserPermission,
}

impl CacheGroup {
    /// The upstream string value, for callers that need to key a map or log it.
    pub fn as_str(self) -> &'static str {
        match self {
            CacheGroup::User => "user",
            CacheGroup::EmailAvatarLink => "email_avatar_link",
            CacheGroup::UserEmailAddresses => "user_email_addresses",
            CacheGroup::GpgKeyWithSubKeys => "gpg_key_with_subkeys",
            CacheGroup::RepoUserPermission => "repo_user_permission",
        }
    }
}

impl std::fmt::Display for CacheGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// `cachegroup.User` — kept as a constant for parity with upstream call sites.
pub const USER: CacheGroup = CacheGroup::User;
/// `cachegroup.EmailAvatarLink`.
pub const EMAIL_AVATAR_LINK: CacheGroup = CacheGroup::EmailAvatarLink;
/// `cachegroup.UserEmailAddresses`.
pub const USER_EMAIL_ADDRESSES: CacheGroup = CacheGroup::UserEmailAddresses;
/// `cachegroup.GPGKeyWithSubKeys`.
pub const GPG_KEY_WITH_SUBKEYS: CacheGroup = CacheGroup::GpgKeyWithSubKeys;
/// `cachegroup.RepoUserPermission`.
pub const REPO_USER_PERMISSION: CacheGroup = CacheGroup::RepoUserPermission;

#[cfg(test)]
mod tests {
    use super::*;

    /// Every group keeps the exact upstream string, since it is used as a cache key.
    #[test]
    fn group_names_match_upstream() {
        let cases: &[(CacheGroup, &str)] = &[
            (USER, "user"),
            (EMAIL_AVATAR_LINK, "email_avatar_link"),
            (USER_EMAIL_ADDRESSES, "user_email_addresses"),
            (GPG_KEY_WITH_SUBKEYS, "gpg_key_with_subkeys"),
            (REPO_USER_PERMISSION, "repo_user_permission"),
        ];
        for (group, want) in cases {
            assert_eq!(group.as_str(), *want);
            assert_eq!(group.to_string(), *want);
        }
    }

    #[test]
    fn groups_are_distinct() {
        assert_ne!(USER, EMAIL_AVATAR_LINK);
    }
}
