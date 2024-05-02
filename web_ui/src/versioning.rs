// Must be kept up to date with web_ui/build_service_worker_file.sh
pub const VERSION: usize = 2;

pub fn add_version_to_path(path: &str) -> String {
    return format!("{path}?v={VERSION}");
}
