mod image_ops;

use serde::Serialize;

#[derive(Serialize)]
struct ImageInfo {
    path: String,
    width: u32,
    height: u32,
    megapixels: f64,
    format: String,
}

#[derive(Serialize)]
struct OutputInfo {
    output_path: String,
    width: u32,
    height: u32,
    megapixels: f64,
}

#[tauri::command]
fn inspect_image(path: String) -> Result<ImageInfo, String> {
    let details = image_ops::inspect_image(&path)?;

    Ok(ImageInfo {
        path,
        width: details.width,
        height: details.height,
        megapixels: details.megapixels,
        format: details.format,
    })
}

#[tauri::command]
fn upscale_image(input_path: String, target_megapixels: f64) -> Result<OutputInfo, String> {
    let result = image_ops::upscale_image(&input_path, target_megapixels)?;

    Ok(OutputInfo {
        output_path: result.output_path,
        width: result.width,
        height: result.height,
        megapixels: result.megapixels,
    })
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![inspect_image, upscale_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

