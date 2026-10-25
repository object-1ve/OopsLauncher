//! 登录自启动：用「登录时」触发的计划任务代替 `HKCU\...\Run`。
//!
//! 不用 Run 键的原因：登录时 explorer 会逐条执行 Run 项并等待其就绪，
//! 排在后面的程序会被拖到登录后几十秒才真正启动（实测 SecurityHealthSystray.exe
//! 25.5 s、RtkAudUService64.exe 15.2 s）。计划任务由 Task Scheduler 服务在登录时
//! 并行触发，不排在这条队列里。
//!
//! 注册走 `/XML` 而不是 `schtasks /Create /SC ONLOGON`：后者不带 `/RU` 时要求管理员
//! 权限，而「只在自己登录时、以当前用户交互令牌、普通权限运行」的任务普通用户可以注册。

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// 计划任务名
const TASK_NAME: &str = "OopsLauncher";
/// 旧版经 tauri-plugin-autostart 写入的 Run 值名，迁移时清理
const LEGACY_RUN_VALUE: &str = "OopsLauncher";
/// HKCU 的 Run 键
const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
/// 子进程不弹控制台窗口
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

fn output_message(out: &std::process::Output) -> String {
    let raw = if out.stderr.is_empty() {
        &out.stdout
    } else {
        &out.stderr
    };
    String::from_utf8_lossy(raw).trim().to_string()
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 当前用户 `DOMAIN\USERNAME`，用于计划任务的触发器和运行主体
fn current_user() -> String {
    let domain = std::env::var("USERDOMAIN").unwrap_or_default();
    let user = std::env::var("USERNAME").unwrap_or_default();
    if domain.is_empty() {
        user
    } else {
        format!("{domain}\\{user}")
    }
}

/// 生成注册用的任务 XML（schtasks 需要 UTF-16，见 [`write_task_xml`]）
fn task_xml(exe: &str, args: &[String], user: &str) -> String {
    let arguments = if args.is_empty() {
        String::new()
    } else {
        format!(
            "\n      <Arguments>{}</Arguments>",
            xml_escape(&args.join(" "))
        )
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>OopsLauncher 登录自启动</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
      <UserId>{user}</UserId>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{user}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>false</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{command}</Command>{arguments}
    </Exec>
  </Actions>
</Task>
"#,
        user = xml_escape(user),
        command = xml_escape(exe),
        arguments = arguments,
    )
}

fn write_task_xml(xml: &str) -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(format!("oopslauncher-autostart-{}.xml", std::process::id()));
    // schtasks 靠 BOM 判定编码；没有 BOM 会按 ANSI 解析并报「无效 XML 格式」
    let mut bytes: Vec<u8> = vec![0xFF, 0xFE];
    bytes.extend(xml.encode_utf16().flat_map(u16::to_le_bytes));
    fs::write(&path, bytes).map_err(|e| format!("写入任务定义失败: {e}"))?;
    Ok(path)
}

fn remove_legacy_run_value() {
    let _ = hidden_command("reg")
        .args(["delete", RUN_KEY, "/v", LEGACY_RUN_VALUE, "/f"])
        .output();
}

/// 是否已注册登录计划任务
#[tauri::command]
pub fn autostart_is_enabled() -> bool {
    hidden_command("schtasks")
        .args(["/Query", "/TN", TASK_NAME])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// 注册（或覆盖）登录计划任务；启动参数由前端按「最小化启动」设置传入
#[tauri::command]
pub fn autostart_enable(args: Vec<String>) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("获取程序路径失败: {e}"))?;
    let exe = exe.to_string_lossy().to_string();
    let xml = task_xml(&exe, &args, &current_user());
    let xml_path = write_task_xml(&xml)?;

    let out = hidden_command("schtasks")
        .args(["/Create", "/TN", TASK_NAME, "/XML"])
        .arg(&xml_path)
        .arg("/F")
        .output()
        .map_err(|e| format!("执行 schtasks 失败: {e}"));
    let _ = fs::remove_file(&xml_path);
    let out = out?;
    if !out.status.success() {
        return Err(format!("注册登录计划任务失败: {}", output_message(&out)));
    }

    // 迁移：删掉旧版本写在 Run 键里的同名自启动项，否则登录时会启动两份
    remove_legacy_run_value();
    Ok(())
}

/// 删除登录计划任务并清理旧版 Run 项；任务本就不存在时不算失败
#[tauri::command]
pub fn autostart_disable() -> Result<(), String> {
    let out = hidden_command("schtasks")
        .args(["/Delete", "/TN", TASK_NAME, "/F"])
        .output()
        .map_err(|e| format!("执行 schtasks 失败: {e}"))?;
    if !out.status.success() {
        let message = output_message(&out);
        // 任务不存在（尚未注册过）按成功处理
        if !autostart_is_enabled() {
            remove_legacy_run_value();
            return Ok(());
        }
        return Err(format!("删除登录计划任务失败: {message}"));
    }
    remove_legacy_run_value();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_xml_uses_exec_and_arguments() {
        let xml = task_xml(
            r"C:\Program Files\OopsLauncher\app.exe",
            &["--minimized".to_string()],
            "PC\\user",
        );
        assert!(xml.contains("<Command>C:\\Program Files\\OopsLauncher\\app.exe</Command>"));
        assert!(xml.contains("<Arguments>--minimized</Arguments>"));
        assert!(xml.contains("<LogonTrigger>"));
        assert!(xml.contains("<LogonType>InteractiveToken</LogonType>"));
        assert!(xml.contains("<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>"));
    }

    #[test]
    fn task_xml_omits_arguments_when_empty() {
        let xml = task_xml(r"C:\app.exe", &[], "PC\\user");
        assert!(!xml.contains("<Arguments>"));
        assert!(xml.contains("<Command>C:\\app.exe</Command>"));
    }

    #[test]
    fn task_xml_escapes_special_characters() {
        let xml = task_xml("C:\\a&b\\app.exe", &["--path=<x>".to_string()], "DOM&IN\\u\"ser");
        assert!(xml.contains("<Command>C:\\a&amp;b\\app.exe</Command>"));
        assert!(xml.contains("<Arguments>--path=&lt;x&gt;</Arguments>"));
        assert!(xml.contains("DOM&amp;IN\\u&quot;ser"));
    }

    #[test]
    fn task_xml_is_utf16_encodable() {
        let xml = task_xml(r"C:\Program Files\OopsLauncher\app.exe", &[], "PC\\user");
        let bytes: Vec<u8> = xml.encode_utf16().flat_map(u16::to_le_bytes).collect();
        assert_eq!(bytes.len(), xml.encode_utf16().count() * 2);
    }

    #[test]
    fn written_task_xml_starts_with_utf16_bom() {
        let xml = task_xml(r"C:\app.exe", &[], "PC\\user");
        let path = write_task_xml(&xml).unwrap();
        let bytes = fs::read(&path).unwrap();
        let _ = fs::remove_file(&path);
        assert_eq!(&bytes[..2], &[0xFF, 0xFE], "schtasks 依赖 BOM 判定 UTF-16");
        assert_eq!(bytes.len(), 2 + xml.encode_utf16().count() * 2);
    }
}
