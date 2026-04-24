//! `hone-cli cleanup` 子命令：交互式删除 `~/.honeclaw` 下的配置 / 数据 / 下载包。
//!
//! 布局约定（与 `install_hone_cli.sh` / packaged install 保持一致）：
//! ```text
//! $HONE_HOME/
//! ├── config.yaml
//! ├── soul.md
//! ├── data/           (runtime: sessions / crons / audit 等)
//! ├── releases/       (下载的 release bundle 解压后放这里)
//! └── current -> releases/<tag>  (symlink)
//! ```
//!
//! 设计点：
//! - **默认不删 config**：brew uninstall 之后用户还能保留 API key 重新装
//! - **非交互 CI 必须显式 `--yes` 或 `--all`**：避免 stdin 挂起
//! - **两次清理都跑一遍**：`prune_empty_dir` 确保 HONE_HOME 被掏空后自动消失

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::{env, fs};

use clap::Args;
use dialoguer::theme::ColorfulTheme;

use crate::prompt_bool;

/// `cleanup` 子命令的 CLI 参数。
#[derive(Args, Debug, Default)]
pub(crate) struct CleanupArgs {
    /// 清掉所有能清的:runtime data + release bundles + config + soul.md。
    #[arg(long)]
    pub all: bool,
    /// 非交互模式：删 runtime data + release bundles,但保留 config / soul.md。
    #[arg(long)]
    pub yes: bool,
    /// 覆盖 `$HONE_HOME`(缺省取环境变量或 `~/.honeclaw`)。主要给测试 / packaged 场景用。
    #[arg(long)]
    pub home: Option<PathBuf>,
}

/// `$HONE_HOME` 下能被 cleanup 清掉的 6 个路径。
#[derive(Debug, Clone, PartialEq, Eq)]
struct CleanupTargets {
    home_dir: PathBuf,
    config_path: PathBuf,
    soul_path: PathBuf,
    data_dir: PathBuf,
    releases_dir: PathBuf,
    current_link: PathBuf,
}

/// 用户（交互或非交互）最终选择了清理哪三类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CleanupSelection {
    remove_config_and_profile: bool,
    remove_runtime_data: bool,
    remove_release_bundles: bool,
}

/// 解析 `$HONE_HOME`：参数 > 环境变量 > `$HOME/.honeclaw`。
/// 相对路径一律解析成绝对路径（基于当前 cwd),避免 chdir 之后行为漂移。
fn cleanup_home_dir(explicit_home: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = explicit_home {
        return Ok(if path.is_absolute() {
            path.to_path_buf()
        } else {
            env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        });
    }
    if let Some(home) = env::var_os("HONE_HOME") {
        let path = PathBuf::from(home);
        return Ok(if path.is_absolute() {
            path
        } else {
            env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        });
    }
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "无法确定 HOME，请传入 `hone-cli cleanup --home <path>`。".to_string())?;
    Ok(home.join(".honeclaw"))
}

fn cleanup_targets(home_dir: &Path) -> CleanupTargets {
    CleanupTargets {
        home_dir: home_dir.to_path_buf(),
        config_path: home_dir.join("config.yaml"),
        soul_path: home_dir.join("soul.md"),
        data_dir: home_dir.join("data"),
        releases_dir: home_dir.join("releases"),
        current_link: home_dir.join("current"),
    }
}

/// 根据 flag + 交互决定要清理的范围。
///
/// 优先级：`--all` > `--yes` > 交互 prompt。非交互场景没传任何 flag 直接报错,
/// 避免在 CI / systemd 环境里挂住 stdin。
fn select_cleanup_targets(
    theme: &ColorfulTheme,
    args: &CleanupArgs,
) -> Result<CleanupSelection, String> {
    if args.all {
        return Ok(CleanupSelection {
            remove_config_and_profile: true,
            remove_runtime_data: true,
            remove_release_bundles: true,
        });
    }
    if args.yes {
        return Ok(CleanupSelection {
            remove_config_and_profile: false,
            remove_runtime_data: true,
            remove_release_bundles: true,
        });
    }
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err("`hone-cli cleanup` 在非交互环境下请显式传 `--yes` 或 `--all`。".to_string());
    }
    println!("Cleanup Hone local files");
    println!("  - 默认只清 runtime 数据和已下载 bundle，不会删除用户 config。");
    println!("  - 如果你连 `config.yaml` / `soul.md` 也想删除，请在下面确认。");
    Ok(CleanupSelection {
        remove_runtime_data: prompt_bool(theme, "Remove runtime data under HONE_HOME/data?", true)?,
        remove_release_bundles: prompt_bool(
            theme,
            "Remove downloaded bundles and current symlink under HONE_HOME/releases and HONE_HOME/current?",
            true,
        )?,
        remove_config_and_profile: prompt_bool(
            theme,
            "Also remove HONE_HOME/config.yaml and HONE_HOME/soul.md?",
            false,
        )?,
    })
}

/// 删除文件 / 目录 / symlink(broken link 也要清)。
/// 返回 `Ok(true)` 表示确实删了东西,`Ok(false)` 表示原本就不存在。
fn remove_path_if_exists(path: &Path) -> Result<bool, String> {
    if !path.exists() && fs::symlink_metadata(path).is_err() {
        return Ok(false);
    }
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).map_err(|e| format!("{}: {e}", path.display()))?;
    } else if metadata.is_dir() {
        fs::remove_dir_all(path).map_err(|e| format!("{}: {e}", path.display()))?;
    } else {
        fs::remove_file(path).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(true)
}

/// 只有当目录已经空了才删。用来在所有子项清完后顺手收掉 HONE_HOME 自身。
fn prune_empty_dir(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }
    let mut entries = fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if entries.next().is_some() {
        return Ok(false);
    }
    fs::remove_dir(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(true)
}

/// `hone-cli cleanup` 子命令入口。
pub(crate) fn run_cleanup(args: CleanupArgs) -> Result<(), String> {
    let home_dir = cleanup_home_dir(args.home.as_deref())?;
    let targets = cleanup_targets(&home_dir);
    let selection = select_cleanup_targets(&ColorfulTheme::default(), &args)?;

    if !selection.remove_config_and_profile
        && !selection.remove_runtime_data
        && !selection.remove_release_bundles
    {
        println!("No cleanup selected. Nothing changed.");
        return Ok(());
    }

    let mut removed = Vec::new();

    if selection.remove_runtime_data && remove_path_if_exists(&targets.data_dir)? {
        removed.push(targets.data_dir.display().to_string());
    }
    if selection.remove_release_bundles && remove_path_if_exists(&targets.releases_dir)? {
        removed.push(targets.releases_dir.display().to_string());
    }
    if selection.remove_release_bundles && remove_path_if_exists(&targets.current_link)? {
        removed.push(targets.current_link.display().to_string());
    }
    if selection.remove_config_and_profile && remove_path_if_exists(&targets.config_path)? {
        removed.push(targets.config_path.display().to_string());
    }
    if selection.remove_config_and_profile && remove_path_if_exists(&targets.soul_path)? {
        removed.push(targets.soul_path.display().to_string());
    }

    let home_removed = prune_empty_dir(&targets.home_dir)?;

    if removed.is_empty() && !home_removed {
        println!(
            "No matching Hone files found under {}.",
            targets.home_dir.display()
        );
    } else {
        println!("Removed:");
        for path in removed {
            println!("  - {path}");
        }
        if home_removed {
            println!("  - {}", targets.home_dir.display());
        }
    }

    println!();
    println!("Homebrew uninstall only removes the package files.");
    println!("If you installed via Homebrew, run one of:");
    println!("  - `brew uninstall honeclaw`");
    println!("  - `brew uninstall B-M-Capital-Research/honeclaw/honeclaw`");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn select_cleanup_targets_defaults_to_runtime_and_bundles_for_yes() {
        let selection = select_cleanup_targets(
            &ColorfulTheme::default(),
            &CleanupArgs {
                yes: true,
                ..CleanupArgs::default()
            },
        )
        .unwrap();

        assert!(selection.remove_runtime_data);
        assert!(selection.remove_release_bundles);
        assert!(!selection.remove_config_and_profile);
    }

    #[test]
    fn remove_path_if_exists_handles_symlink() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("target.txt");
        let link = dir.path().join("link.txt");
        fs::write(&target, "hello").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert!(remove_path_if_exists(&link).unwrap());
        assert!(!link.exists());
        assert!(target.exists());
    }
}
