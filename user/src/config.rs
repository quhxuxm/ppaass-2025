use std::path::Path;
pub trait UserRepositoryConfig {
    fn refresh_interval_sec(&self) -> u64;
}
pub trait FileSystemUserRepositoryConfig: UserRepositoryConfig {
    fn user_repo_directory(&self) -> &Path;
    fn public_key_file_name(&self) -> &str;
    fn private_key_file_name(&self) -> &str;
    fn user_info_file_name(&self) -> &str;
}
