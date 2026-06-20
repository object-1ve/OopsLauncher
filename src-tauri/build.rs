fn main() {
    // 必须显式声明：tauri_build 会发出自己的 rerun-if-changed 指令，
    // 一旦如此，Cargo 只会跟踪被列出的文件。若不加这一行，修改 app.manifest
    // （如 requireAdministrator -> asInvoker）不会触发 build.rs 重跑，
    // 旧的 manifest 会一直被编译进二进制，导致拖放被 UIPI 拦截（禁止光标）。
    println!("cargo:rerun-if-changed=app.manifest");

    let mut windows_attributes = tauri_build::WindowsAttributes::new();
    windows_attributes = windows_attributes.app_manifest(include_str!("app.manifest"));

    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows_attributes))
        .expect("failed to run tauri-build");
}
