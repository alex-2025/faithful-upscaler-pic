use image::codecs::jpeg::JpegEncoder;
use image::imageops::{self, FilterType};
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
pub struct ImageDetails {
    pub width: u32,
    pub height: u32,
    pub megapixels: f64,
    pub format: String,
}

#[derive(Serialize)]
pub struct UpscaleResult {
    pub output_path: String,
    pub width: u32,
    pub height: u32,
    pub megapixels: f64,
}

pub fn inspect_image(path: &str) -> Result<ImageDetails, String> {
    let path_ref = Path::new(path);
    let image = ImageReader::open(path_ref)
        .map_err(|error| format!("无法打开图片：{error}"))?
        .with_guessed_format()
        .map_err(|error| format!("无法识别图片格式：{error}"))?
        .decode()
        .map_err(|error| format!("无法读取图片内容：{error}"))?;

    let (width, height) = image.dimensions();

    Ok(ImageDetails {
        width,
        height,
        megapixels: megapixels(width, height),
        format: extension_label(path_ref),
    })
}

pub fn upscale_image(path: &str, target_megapixels: f64) -> Result<UpscaleResult, String> {
    if target_megapixels <= 0.0 {
        return Err("目标像素必须大于 0".into());
    }

    let path_ref = Path::new(path);
    let image = ImageReader::open(path_ref)
        .map_err(|error| format!("无法打开图片：{error}"))?
        .with_guessed_format()
        .map_err(|error| format!("无法识别图片格式：{error}"))?
        .decode()
        .map_err(|error| format!("无法读取图片内容：{error}"))?;

    let (source_width, source_height) = image.dimensions();
    let source_megapixels = megapixels(source_width, source_height);

    if target_megapixels <= source_megapixels {
      return Err(format!(
          "目标像素 {:.2}MP 不大于原图 {:.2}MP",
          target_megapixels, source_megapixels
      ));
    }

    let (target_width, target_height) =
        calculate_target_size(source_width, source_height, target_megapixels);

    let enlarged = progressive_resize(image, target_width, target_height);
    let output_path = build_output_path(path_ref, target_megapixels)?;
    save_image(&enlarged, &output_path)?;

    Ok(UpscaleResult {
        output_path: output_path.to_string_lossy().to_string(),
        width: target_width,
        height: target_height,
        megapixels: megapixels(target_width, target_height),
    })
}

fn calculate_target_size(width: u32, height: u32, target_megapixels: f64) -> (u32, u32) {
    let ratio = width as f64 / height as f64;
    let target_pixels = target_megapixels * 1_000_000.0;
    let target_height = (target_pixels / ratio).sqrt().round().max(1.0) as u32;
    let target_width = (target_height as f64 * ratio).round().max(1.0) as u32;
    (target_width.max(width + 1), target_height.max(height + 1))
}

fn progressive_resize(image: DynamicImage, target_width: u32, target_height: u32) -> DynamicImage {
    let mut current = image;

    loop {
        let (width, height) = current.dimensions();
        let next_width = (width.saturating_mul(2)).min(target_width);
        let next_height = (height.saturating_mul(2)).min(target_height);

        if next_width == width && next_height == height {
            break;
        }

        current = current.resize(next_width, next_height, FilterType::Lanczos3);

        if next_width == target_width && next_height == target_height {
            break;
        }
    }

    let final_image = current.resize(target_width, target_height, FilterType::Lanczos3);
    let sharpened = imageops::unsharpen(&final_image.to_rgba8(), 0.6, 1);
    DynamicImage::ImageRgba8(sharpened)
}

fn build_output_path(input_path: &Path, target_megapixels: f64) -> Result<PathBuf, String> {
    let parent = input_path
        .parent()
        .ok_or_else(|| "无法定位原图目录".to_string())?;

    let stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "无法解析文件名".to_string())?;

    let ext = normalized_output_extension(input_path);
    let suffix = format!("_{}M", target_megapixels.round() as u32);
    let mut candidate = parent.join(format!("{stem}{suffix}.{ext}"));
    let mut index = 2_u32;

    while candidate.exists() {
        candidate = parent.join(format!("{stem}{suffix}_{index}.{ext}"));
        index += 1;
    }

    Ok(candidate)
}

fn normalized_output_extension(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "jpg",
        "bmp" => "bmp",
        _ => "png",
    }
}

fn save_image(image: &DynamicImage, output_path: &Path) -> Result<(), String> {
    match normalized_output_extension(output_path) {
        "jpg" => {
            let file = File::create(output_path)
                .map_err(|error| format!("无法创建输出文件：{error}"))?;
            let mut encoder = JpegEncoder::new_with_quality(file, 95);
            encoder
                .encode_image(&DynamicImage::ImageRgb8(image.to_rgb8()))
                .map_err(|error| format!("无法保存 JPG：{error}"))?;
        }
        "bmp" => image
            .save_with_format(output_path, ImageFormat::Bmp)
            .map_err(|error| format!("无法保存 BMP：{error}"))?,
        _ => image
            .save_with_format(output_path, ImageFormat::Png)
            .map_err(|error| format!("无法保存 PNG：{error}"))?,
    }

    Ok(())
}

fn extension_label(path: &Path) -> String {
    path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .to_ascii_uppercase()
}

fn megapixels(width: u32, height: u32) -> f64 {
    (width as f64 * height as f64) / 1_000_000.0
}
