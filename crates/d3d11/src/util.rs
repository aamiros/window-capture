use std::io;

pub fn get_last_error(message: &str) -> anyhow::Error {
    let os_error = io::Error::last_os_error();
    anyhow::anyhow!("{}: {}", message, os_error)
}
