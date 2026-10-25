import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "@/utils/env";

// 自启动使用「登录时」触发的 Windows 计划任务（Rust 侧 commands::autostart），
// 不再写 HKCU\...\Run：Run 项在登录时由 explorer 串行执行并等待就绪，排在
// SecurityHealthSystray / RtkAudUService64 之后会被拖到登录后几十秒才启动。

export const isAutoStartEnabled = () =>
  isTauri() ? invoke("autostart_is_enabled") : false;

export const enableAutoStart = (args = []) =>
  isTauri() ? invoke("autostart_enable", { args }) : undefined;

export const disableAutoStart = () =>
  isTauri() ? invoke("autostart_disable") : undefined;
