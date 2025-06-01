use users::get_current_username;

pub fn username() -> Option<String> {
    get_current_username().map(|u| u.to_string_lossy().to_string())
}

