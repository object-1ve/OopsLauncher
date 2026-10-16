use std::path::Path;

#[cfg(target_os = "windows")]
mod win_icon {
    use std::cell::Cell;
    use std::collections::{HashMap, VecDeque};
    use std::io::Cursor;
    use std::path::Path;
    use std::sync::LazyLock;

    use base64::Engine as _;
    use base64::engine::general_purpose;
    use image::{imageops::flip_vertical, ImageBuffer, ImageFormat, Rgba};
    use parking_lot::Mutex;
    use windows::{
        core::{HSTRING, PCWSTR},
        Win32::Foundation::SIZE,
        Win32::Graphics::Gdi::{DeleteObject, GetObjectW, BITMAP},
        Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED},
        Win32::UI::Shell::{
            IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_ICONONLY,
        },
    };

    /// 默认图标边长（像素）。资源管理器网格项图标 CSS 尺寸 48px，2 倍屏需要 96px，128 留余量。
    pub const DEFAULT_ICON_SIZE: u32 = 128;

    /// 允许的图标边长范围，防止前端传入异常值导致无谓的位图分配。
    const ICON_SIZE_MIN: u32 = 16;
    const ICON_SIZE_MAX: u32 = 256;

    /// 这些类型的图标随文件本身变化（图标来自文件内嵌资源），必须按完整路径缓存；
    /// 其它类型在资源管理器里按扩展名共用同一个系统图标，可按扩展名缓存。
    const PER_FILE_ICON_EXTS: &[&str] = &[
        "exe", "lnk", "ico", "url", "msi", "scr", "cpl", "com", "pif", "bat", "cmd", "ps1",
        "vbs", "jar", "dll", "ocx",
    ];

    /// 缓存键数量上限：一个键 = 一种尺寸 + 一个目录/扩展名/路径。
    const ICON_CACHE_LIMIT: usize = 512;

    struct IconCache {
        entries: HashMap<String, String>,
        order: VecDeque<String>,
    }

    impl IconCache {
        fn insert(&mut self, key: String, icon: String) {
            if self.entries.len() >= ICON_CACHE_LIMIT {
                let drop_count = ICON_CACHE_LIMIT / 4 + 1;
                for _ in 0..drop_count {
                    if let Some(oldest) = self.order.pop_front() {
                        self.entries.remove(&oldest);
                    }
                }
            }
            self.order.push_back(key.clone());
            self.entries.insert(key, icon);
        }
    }

    /// 进程级图标缓存：目录/扩展名类图标只走一次 Shell。
    static ICON_CACHE: LazyLock<Mutex<IconCache>> = LazyLock::new(|| {
        Mutex::new(IconCache {
            entries: HashMap::new(),
            order: VecDeque::new(),
        })
    });

    /// 计算图标缓存键：目录共用一个图标；扩展名类图标按扩展名共用；带内嵌图标的类型按完整路径区分。
    pub(super) fn cache_key(path: &Path, size: u32) -> String {
        if path.is_dir() {
            return format!("dir:{size}");
        }
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if PER_FILE_ICON_EXTS.contains(&ext.as_str()) {
            format!("file:{size}:{}", path.to_string_lossy().to_lowercase())
        } else {
            format!("ext:{size}:{ext}")
        }
    }

    /// 该路径的图标能否被其它文件复用（目录图标、按扩展名的类型图标）：能则返回其通用缓存键。
    /// 前端按同一键前缀直接命中通用图标，避免「先占位符再换图标」的闪烁。
    pub(super) fn shared_cache_key(path: &Path, size: u32) -> Option<String> {
        let key = cache_key(path, size);
        let shared = key.starts_with("dir:") || key.starts_with("ext:");
        shared.then_some(key)
    }

    thread_local! {
        /// 每个线程只初始化一次 COM。
        /// 不再调用 CoUninitialize：原先每取一个图标就 CoInitializeEx + CoUninitialize，
        /// 既慢（每次重建套间），又会在已被其它组件按其它模式初始化的线程上误减引用计数。
        static COM_READY: Cell<bool> = const { Cell::new(false) };
    }

    fn ensure_com_ready() {
        COM_READY.with(|ready| {
            if !ready.get() {
                // 线程已按其它模式初始化（返回 RPC_E_CHANGED_MODE）也无妨，Shell API 在 STA/MTA 下都可用
                unsafe {
                    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
                }
                ready.set(true);
            }
        });
    }

    /// 取指定边长的图标（data:image/png;base64），命中缓存时不做任何 Shell 调用。
    pub fn get_file_icon(path: &Path, size: u32) -> Result<String, String> {
        let size = size.clamp(ICON_SIZE_MIN, ICON_SIZE_MAX);
        let path_str = path.to_str().ok_or("Invalid path")?;
        let key = cache_key(path, size);

        if let Some(cached) = ICON_CACHE.lock().entries.get(&key) {
            return Ok(cached.clone());
        }

        let icon = extract_icon(path_str, size)?;
        // 只缓存成功结果：失败多为路径不可访问或临时占用，缓存会让整类文件永久丢图标
        ICON_CACHE.lock().insert(key, icon.clone());
        Ok(icon)
    }

    fn extract_icon(path_str: &str, size: u32) -> Result<String, String> {
        ensure_com_ready();

        // SHCreateItemFromParsingName 只认反斜杠，混用 '/' 会直接返回 E_INVALIDARG(0x80070057)
        let shell_path = path_str.replace('/', "\\");
        let h_path = HSTRING::from(shell_path.as_str());

        let shell_item_image_factory: IShellItemImageFactory = unsafe {
            SHCreateItemFromParsingName(PCWSTR(h_path.as_ptr()), None)
                .map_err(|e| format!("Failed to create shell item: {}", e))?
        };

        let mut image_buffer = get_image_buffer(&shell_item_image_factory, size as i32)?;

        // 参考 DawnLauncher 的逻辑：检查透明度比例
        // 如果透明区域太大，说明可能是小图标拉伸的，改用 48x48
        if size >= DEFAULT_ICON_SIZE && is_mostly_transparent(&image_buffer) {
            if let Ok(small_buffer) = get_image_buffer(&shell_item_image_factory, 48) {
                image_buffer = small_buffer;
            }
        }

        // 转换为 base64
        Ok(image_buffer_to_base64(image_buffer))
    }

    fn get_image_buffer(
        factory: &IShellItemImageFactory,
        size: i32,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
        unsafe {
            let h_bitmap = factory
                .GetImage(SIZE { cx: size, cy: size }, SIIGBF_ICONONLY)
                .map_err(|e| format!("Failed to get image: {}", e))?;

            let mut bitmap: BITMAP = BITMAP::default();
            GetObjectW(
                h_bitmap,
                std::mem::size_of::<BITMAP>() as i32,
                Some(&mut bitmap as *mut _ as _),
            );

            let width = bitmap.bmWidth as u32;
            let height = bitmap.bmHeight as u32;

            // 确保有位图数据
            if bitmap.bmBits.is_null() {
                DeleteObject(h_bitmap);
                return Err("Bitmap bits are null".to_string());
            }

            let pixel_data = std::slice::from_raw_parts(
                bitmap.bmBits as *const u8,
                (width * height * 4) as usize,
            );

            let mut buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, pixel_data.to_vec())
                .ok_or("Failed to create image buffer")?;

            // Windows 返回的是 BGRA，转换为 RGBA
            for pixel in buffer.pixels_mut() {
                let b = pixel[0];
                let r = pixel[2];
                pixel[0] = r;
                pixel[2] = b;
            }

            DeleteObject(h_bitmap);
            Ok(buffer)
        }
    }

    fn is_mostly_transparent(buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> bool {
        let mut transparent_pixels = 0;
        let total_pixels = buffer.width() * buffer.height();

        for pixel in buffer.pixels() {
            if pixel[3] == 0 {
                transparent_pixels += 1;
            }
        }

        let transparency_ratio = (transparent_pixels as f64) / (total_pixels as f64);
        transparency_ratio >= 0.85 // 提高一点阈值，DawnLauncher 是 0.7
    }

    fn image_buffer_to_base64(buffer: ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
        let mut cursor = Cursor::new(Vec::new());
        // Windows GDI 位图通常是倒置的，所以需要翻转
        let flipped_buffer = flip_vertical(&buffer);
        flipped_buffer.write_to(&mut cursor, ImageFormat::Png).unwrap();
        let encoded = general_purpose::STANDARD.encode(cursor.into_inner());
        format!("data:image/png;base64,{}", encoded)
    }
}

#[cfg(target_os = "windows")]
pub use win_icon::DEFAULT_ICON_SIZE;

/// 该路径的图标能否被其它文件复用（目录图标、按扩展名的类型图标），能则返回其通用缓存键。
#[cfg(target_os = "windows")]
pub fn shared_cache_key(path: &Path, size: u32) -> Option<String> {
    win_icon::shared_cache_key(path, size)
}

/// 非 Windows 平台没有原生图标，统一返回空串（前端显示占位符）。
#[cfg(not(target_os = "windows"))]
pub const DEFAULT_ICON_SIZE: u32 = 128;

/// 非 Windows 平台没有可复用的系统图标。
#[cfg(not(target_os = "windows"))]
pub fn shared_cache_key(_path: &Path, _size: u32) -> Option<String> {
    None
}

/// 取文件/目录的系统图标，返回 `data:image/png;base64,...`。
/// 同一目录、同扩展名的文件共用一份缓存，重复调用不会重复走 Shell。
#[cfg(target_os = "windows")]
pub fn get_file_icon(path: &Path, size: u32) -> Result<String, String> {
    win_icon::get_file_icon(path, size)
}

#[cfg(not(target_os = "windows"))]
pub fn get_file_icon(_path: &Path, _size: u32) -> Result<String, String> {
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    mod windows_tests {
        use super::super::win_icon::cache_key;
        use std::path::Path;

        #[test]
        fn test_cache_key_groups_files_by_extension() {
            // 同一扩展名 + 同一尺寸必须落在同一个键上（这批文件共用一次 Shell 图标提取）
            let a = cache_key(Path::new("C:\\dir\\a.txt"), 32);
            let b = cache_key(Path::new("C:\\other\\b.TXT"), 32);
            assert_eq!(a, b, "同扩展名文件必须共用一个缓存键");

            // 尺寸不同必须分开缓存，否则网格视图会拿到列表尺寸的图标
            assert_ne!(a, cache_key(Path::new("C:\\dir\\a.txt"), 64));
        }

        #[test]
        fn test_cache_key_separates_per_file_icon_types() {
            // exe/lnk 等类型的图标来自文件内嵌资源，必须按路径区分，否则两个 exe 会显示同一个图标
            let one = cache_key(Path::new("C:\\dir\\one.exe"), 32);
            let two = cache_key(Path::new("C:\\dir\\two.exe"), 32);
            assert_ne!(one, two, "exe 必须按完整路径区分");

            // 目录与无扩展名文件不能混在一起
            assert_ne!(
                cache_key(Path::new("C:\\Windows"), 32),
                cache_key(Path::new("C:\\Windows\\hosts"), 32)
            );
        }

        #[test]
        fn test_get_file_icon_accepts_forward_slashes() {
            // 用户在地址栏输入 C:/Windows 时列出的路径会混用分隔符，
            // SHCreateItemFromParsingName 只认反斜杠，这里必须仍然取到图标
            let icon = crate::icon::get_file_icon(Path::new("C:/Windows"), 32);
            assert!(
                icon.as_ref().is_ok_and(|i| i.starts_with("data:image/png;base64,")),
                "C:/Windows 应能取到 PNG 图标: {:?}",
                icon.err()
            );
        }
    }
}
