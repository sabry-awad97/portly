use anyhow::Context;
use crate::{details, scanner::Scanner};

pub fn handle_details(
    scanner: &mut Scanner,
    port: u16,
    no_prompt: bool,
    json: bool,
    no_color: bool,
) -> anyhow::Result<()> {
    let port_info = scanner
        .get_port_details(port)
        .context(format!("Failed to get details for port {}", port))?;

    let process_info = scanner
        .get_process_info_by_pid(port_info.pid)
        .context(format!(
            "Failed to get process info for PID {}",
            port_info.pid
        ))?;

    if json {
        // JSON output
        let json_output = serde_json::json!({
            "port": port_info.port,
            "pid": port_info.pid,
            "process_name": port_info.process_name,
            "status": port_info.status,
            "framework": port_info.framework,
            "project_name": port_info.project_name,
            "memory_kb": process_info.memory_kb,
            "command": process_info.command,
            "working_dir": process_info.working_dir,
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Detailed view
        details::show_port_details(&port_info, &process_info, scanner, !no_color)?;

        // Interactive kill prompt
        if !no_prompt {
            details::prompt_kill_process(port_info.pid, scanner)?;
        }
    }

    Ok(())
}
