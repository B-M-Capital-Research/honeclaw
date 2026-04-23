use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedChannelProcess {
    pub channel: String,
    pub pid: u32,
    pub executable: String,
    pub command: String,
}

pub fn scan_channel_processes(channel: &str) -> Vec<ObservedChannelProcess> {
    let binary = channel_binary_name(channel);
    let Ok(output) = Command::new("ps").args(["-Ao", "pid=,args="]).output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| parse_ps_line(channel, binary, line))
        .collect()
}

fn parse_ps_line(channel: &str, binary: &str, line: &str) -> Option<ObservedChannelProcess> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let pid = parts.next()?.parse::<u32>().ok()?;
    let command = parts.collect::<Vec<_>>().join(" ");
    if command.is_empty() {
        return None;
    }

    let executable = command.split_whitespace().next()?.to_string();
    let executable_name = Path::new(&executable).file_name()?.to_string_lossy();
    if executable_name != binary {
        return None;
    }

    Some(ObservedChannelProcess {
        channel: channel.to_string(),
        pid,
        executable,
        command,
    })
}

pub fn channel_binary_name(channel: &str) -> &'static str {
    match channel {
        "imessage" => "hone-imessage",
        "discord" => "hone-discord",
        "feishu" => "hone-feishu",
        "telegram" => "hone-telegram",
        _ => "",
    }
}
