use std::path::Path;
use image::{ImageBuffer, Rgba, ImageFormat, imageops::flip_vertical};
use base64::Engine as _;
use base64::engine::general_purpose;
use std::io::Cursor;

#[cfg(target_os = "windows")]
mod win_icon {
    use super::*;
    use windows::{
        core::{HSTRING, PCWSTR},
        Win32::Foundation::SIZE,
        Win32::Graphics::Gdi::{GetObjectW, BITMAP, DeleteObject},
        Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
        Win32::UI::Shell::{
            IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_ICONONLY,
        },
    };

    pub fn get_file_icon_base64(path: &Path) -> Result<String, String> {
        let path_str = path.to_str().ok_or("Invalid path")?;
        let h_path = HSTRING::from(path_str);
        
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        let result = (|| -> Result<String, String> {
            let shell_item_image_factory: IShellItemImageFactory = unsafe {
                SHCreateItemFromParsingName(PCWSTR(h_path.as_ptr()), None)
                    .map_err(|e| format!("Failed to create shell item: {}", e))?
            };

            // 尝试获取 256x256 的大图标
            let mut image_buffer = get_image_buffer(&shell_item_image_factory, 256)?;

            // 参考 DawnLauncher 的逻辑：检查透明度比例
            // 如果透明区域太大，说明可能是小图标拉伸的，改用 48x48
            if is_mostly_transparent(&image_buffer) {
                if let Ok(small_buffer) = get_image_buffer(&shell_item_image_factory, 48) {
                    image_buffer = small_buffer;
                }
            }

            // 转换为 base64
            Ok(image_buffer_to_base64(image_buffer))
        })();

        unsafe {
            CoUninitialize();
        }

        result
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

#[cfg(not(target_os = "windows"))]
pub fn get_file_icon_base64(_path: &Path) -> Result<String, String> {
    Ok("".to_string())
}

#[cfg(target_os = "windows")]
pub fn get_file_icon_base64(path: &Path) -> Result<String, String> {
    win_icon::get_file_icon_base64(path)
}
