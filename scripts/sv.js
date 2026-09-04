import fs from "node:fs";
import path from "node:path";

const rootDir = process.cwd();
const packageJsonPath = path.join(rootDir, "package.json");
const tauriConfigPath = path.join(rootDir, "src-tauri", "tauri.conf.json");
const cargoTomlPath = path.join(rootDir, "src-tauri", "Cargo.toml");

// 允许传入 tag 形式（v0.4.32），去前缀后作为唯一版本源；CI 直接传 github.ref_name 即可
const inputVersion = process.argv[2]?.trim().replace(/^v/i, "");

const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
const currentVersion = packageJson.version;
const targetVersion = inputVersion || currentVersion;

if (!/^\d+\.\d+\.\d+$/.test(targetVersion)) {
  console.error("版本号格式必须是 x.y.z，例如 0.3.25");
  process.exit(1);
}

packageJson.version = targetVersion;
fs.writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`, "utf8");

const tauriConfig = JSON.parse(fs.readFileSync(tauriConfigPath, "utf8"));
tauriConfig.version = targetVersion;
fs.writeFileSync(tauriConfigPath, `${JSON.stringify(tauriConfig, null, 2)}\n`, "utf8");

const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
const nextCargoToml = cargoToml.replace(
  /^version\s*=\s*".*"$/m,
  `version = "${targetVersion}"`
);
fs.writeFileSync(cargoTomlPath, nextCargoToml, "utf8");

console.log(`版本已同步到 ${targetVersion}`);
