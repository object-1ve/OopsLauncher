import { execSync, spawnSync } from "node:child_process";

// 构建前自动把最新 tag 写入三处版本文件（package.json / tauri.conf.json / Cargo.toml）。
// 被 tauri.conf.json 的 beforeBuildCommand 调用，本地构建与 CI 共用同一机制。
// 非严格模式：无 git / 无 tag 时（如源码包构建）警告并保留文件现有版本，不中断构建。
// 发版正确性另由 release.yml 的严格步骤（github.ref_name）兜底。

let tag = "";
try {
  tag = execSync("git describe --tags --abbrev=0", {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "ignore"],
  }).trim();
} catch {
  console.warn("[sv-tag] 未找到 git tag，保留文件现有版本");
  process.exit(0);
}

const r = spawnSync(process.execPath, ["scripts/sv.js", tag], {
  stdio: "inherit",
});
process.exit(r.status ?? 1);
