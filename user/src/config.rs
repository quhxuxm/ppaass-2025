use std::path::Path;
pub trait UserRepositoryConfig {
    fn refresh_interval_sec(&self) -> u64;
}
pub trait FileSystemUserRepositoryConfig: UserRepositoryConfig {
    fn user_repo_directory(&self) -> &Path;
}