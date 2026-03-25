//! 辅助函数
//!
//! 提供路径处理和其他通用工具函数

use super::types::MaaCallbackEvent;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

/// 发送回调事件到前端
pub fn emit_callback_event<S: Into<String>>(app: &AppHandle, message: S, details: S) {
    let event = MaaCallbackEvent {
        message: message.into(),
        details: details.into(),
    };
    if let Err(e) = app.emit("maa-callback", event) {
        log::error!("Failed to emit maa-callback: {}", e);
    }
}

/// 获取应用数据目录
/// - macOS: ~/Library/Application Support/MXU/
/// - Windows/Linux: exe 所在目录（保持便携式部署）
pub fn get_app_data_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").map_err(|_| "无法获取 HOME 环境变量".to_string())?;
        let path = PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("MXU");
        Ok(path)
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Windows/Linux 保持便携式，使用 exe 所在目录
        get_exe_directory()
    }
}

/// 规范化路径：移除冗余的 `.`、处理 `..`、统一分隔符
/// 使用 Path::components() 解析，不需要路径实际存在
pub fn normalize_path(path: &str) -> PathBuf {
    use std::path::{Component, Path};

    let path = Path::new(path);
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            // 跳过当前目录标记 "."
            Component::CurDir => {}
            // 处理父目录 ".."：如果栈顶是普通目录则弹出，否则保留
            Component::ParentDir => {
                if matches!(components.last(), Some(Component::Normal(_))) {
                    components.pop();
                } else {
                    components.push(component);
                }
            }
            // 保留其他组件（Prefix、RootDir、Normal）
            _ => components.push(component),
        }
    }

    // 重建路径
    components.into_iter().collect()
}

/// 获取日志目录（应用数据目录下的 debug 子目录）
pub fn get_logs_dir() -> PathBuf {
    get_app_data_dir()
        .unwrap_or_else(|_| {
            // 回退到 exe 目录
            let exe_path = std::env::current_exe().unwrap_or_default();
            exe_path
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        })
        .join("debug")
}

fn get_project_root_override() -> Result<Option<PathBuf>, String> {
    match std::env::var_os("MXU_PROJECT_ROOT") {
        Some(value) => {
            let path = PathBuf::from(value);
            if path.exists() {
                Ok(Some(path))
            } else {
                Err(format!(
                    "MXU_PROJECT_ROOT 指向的目录不存在: {}",
                    path.display()
                ))
            }
        }
        None => Ok(None),
    }
}

fn find_dev_project_root(exe_dir: &Path) -> Option<PathBuf> {
    exe_dir.ancestors().find_map(|dir| {
        let has_interface = dir.join("interface.json").exists();
        let has_maafw = dir.join("maafw").exists();
        if has_interface && has_maafw {
            Some(dir.to_path_buf())
        } else {
            None
        }
    })
}

/// 获取 exe 所在目录路径（内部使用）
pub fn get_exe_directory() -> Result<PathBuf, String> {
    if let Some(project_root) = get_project_root_override()? {
        return Ok(project_root);
    }

    let exe_path = std::env::current_exe().map_err(|e| format!("获取 exe 路径失败: {}", e))?;
    let exe_dir = exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "无法获取 exe 所在目录".to_string())?;

    if exe_dir.join("interface.json").exists() {
        return Ok(exe_dir);
    }

    if let Some(project_root) = find_dev_project_root(&exe_dir) {
        log::info!(
            "Detected development project root from exe directory: {}",
            project_root.display()
        );
        return Ok(project_root);
    }

    Ok(exe_dir)
}

/// 获取可执行文件所在目录下的 maafw 子目录
pub fn get_maafw_dir() -> Result<PathBuf, String> {
    Ok(get_exe_directory()?.join("maafw"))
}

/// 构建 User-Agent 字符串
pub fn build_user_agent() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let tauri_version = tauri::VERSION;
    format!("MXU/{} ({}; {}) Tauri/{}", version, os, arch, tauri_version)
}
