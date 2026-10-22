use super::StoreResult;

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: i64,
    pub github_id: i64,
    pub login: String,
    pub admin: bool,
    pub blocked: bool,
}

#[derive(Debug, Clone)]
pub struct KeyRow {
    pub id: i64,
    pub name: String,
    pub read: bool,
    pub write: bool,
    pub export: bool,
}

pub trait AccessRepo: Send + Sync {
    fn upsert_user(&self, github_id: i64, login: &str) -> StoreResult<UserRow>;
    fn user(&self, id: i64) -> StoreResult<Option<UserRow>>;
    fn list_users(&self) -> StoreResult<Vec<UserRow>>;
    /// Admins that are not blocked (the ones who can still manage the site).
    fn count_active_admins(&self) -> StoreResult<u32>;
    fn set_blocked(&self, id: i64, blocked: bool) -> StoreResult<()>;
    fn set_admin(&self, id: i64, admin: bool) -> StoreResult<()>;
    fn create_key(
        &self,
        name: &str,
        hash: &str,
        read: bool,
        write: bool,
        export: bool,
    ) -> StoreResult<i64>;
    fn key_by_hash(&self, hash: &str) -> StoreResult<Option<KeyRow>>;
    fn touch_key(&self, id: i64) -> StoreResult<()>;
    fn revoke_key(&self, id: i64) -> StoreResult<()>;
    fn list_keys(&self) -> StoreResult<Vec<KeyRow>>;
}
