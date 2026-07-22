use std::process::Command;

use crate::invoker::exec::detect_shell_static;

// ============================================================================
// Placeholder substitution
// ============================================================================

#[derive(Debug, Clone)]
pub struct Placeholder {
    pub original: String,
    pub name: String,
    pub display_name: String,
}

pub fn detect_placeholders(command: &str) -> Vec<Placeholder> {
    use regex::Regex;

    let re = Regex::new(r"<([^>]+)>").unwrap();
    re.captures_iter(command)
        .map(|cap| {
            let name = cap[1].to_string();
            let display_name = match name.as_str() {
                "pid" => "Process ID",
                "port" => "Port",
                "package" => "Package",
                "file" => "File",
                "directory" | "dir" => "Directory",
                "service" | "svc" => "Service",
                "message" | "msg" => "Message",
                "image" | "img" => "Image",
                "container" => "Container",
                "signal" => "Signal",
                "user" => "User",
                "host" => "Host",
                "name" => "Name",
                "tag" => "Tag",
                _ => &name,
            }
            .to_string();

            Placeholder {
                original: cap[0].to_string(),
                name: name.clone(),
                display_name,
            }
        })
        .collect()
}

pub fn substitute_placeholders(command: &str, values: &[(String, String)]) -> String {
    let mut result = command.to_string();
    for (name, value) in values {
        result = result.replace(&format!("<{}>", name), value);
    }
    result
}

pub fn validate_password() -> bool {
    let shell = detect_shell_static();

    let result = Command::new("sudo")
        .arg("-v")
        .env("SHELL", &shell)
        .env(
            "HOME",
            std::env::var("HOME").unwrap_or_else(|_| "/".to_string()),
        )
        .output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}
