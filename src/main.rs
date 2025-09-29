use std::env;
use std::path::Path;
use qrcode::QrCode;
use image::{DynamicImage, Rgb, RgbImage, imageops};
use image::ImageReader;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: {} <url> <icon_path> <output_path>", args[0]);
        eprintln!("Example: {} https://example.com logo.png output.png", args[0]);
        std::process::exit(1);
    }

    let url = &args[1];
    let icon_path = &args[2];
    let output_path = &args[3];

    match generate_qr_with_icon(url, icon_path, output_path) {
        Ok(_) => println!("QR code with icon generated successfully: {}", output_path),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn generate_qr_with_icon(url: &str, icon_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Generate QR code
    let code = QrCode::new(url)?;

    // Create QR code image that occupies the entire canvas
    let qr_size = 400;
    let qr_width = code.width();
    let module_size = qr_size / qr_width as u32; // Calculate module size to fill entire image
    let actual_qr_size = module_size * qr_width as u32; // Actual size might be slightly smaller

    let mut qr_image = RgbImage::new(actual_qr_size, actual_qr_size);

    // Fill with white background
    for pixel in qr_image.pixels_mut() {
        *pixel = Rgb([255, 255, 255]);
    }

    // Draw QR code modules to fill the entire image
    for y in 0..qr_width {
        for x in 0..qr_width {
            if code[(x, y)] == qrcode::Color::Dark {
                // Draw a dark module
                let start_x = (x as u32) * module_size;
                let start_y = (y as u32) * module_size;

                for dy in 0..module_size {
                    for dx in 0..module_size {
                        let px = start_x + dx;
                        let py = start_y + dy;
                        if px < actual_qr_size && py < actual_qr_size {
                            qr_image.put_pixel(px, py, Rgb([0, 0, 0]));
                        }
                    }
                }
            }
        }
    }

    // Load and process the icon (make it proportional to QR code size)
    let icon_size = actual_qr_size / 5; // Icon will be 1/5 of the QR code size
    let icon = load_and_resize_icon(icon_path, icon_size)?;

    // Create the final image with icon in center
    let final_image = overlay_icon_on_qr(qr_image, icon)?;

    // Save the result
    final_image.save(output_path)?;

    Ok(())
}

fn load_and_resize_icon(icon_path: &str, size: u32) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    // Check if file exists
    if !Path::new(icon_path).exists() {
        return Err(format!("Icon file not found: {}", icon_path).into());
    }

    // Load the icon image
    let icon = ImageReader::open(icon_path)?
        .decode()?;

    // Resize the icon to the specified size while maintaining aspect ratio
    let resized_icon = icon.resize(size, size, imageops::FilterType::Lanczos3);

    Ok(resized_icon)
}

fn overlay_icon_on_qr(mut qr_image: RgbImage, icon: DynamicImage) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    // Calculate center position for the icon
    let qr_width = qr_image.width();
    let qr_height = qr_image.height();
    let icon_width = icon.width();
    let icon_height = icon.height();

    let x_offset = (qr_width - icon_width) / 2;
    let y_offset = (qr_height - icon_height) / 2;

    // Create a white background for the icon area to ensure it's readable
    let background_size = icon_width + 10; // Add 5 pixels padding on each side
    let bg_x = if x_offset >= 5 { x_offset - 5 } else { 0 };
    let bg_y = if y_offset >= 5 { y_offset - 5 } else { 0 };

    // Draw white background directly on the RGB image
    for y in 0..background_size {
        for x in 0..background_size {
            let px = bg_x + x;
            let py = bg_y + y;
            if px < qr_width && py < qr_height {
                qr_image.put_pixel(px, py, Rgb([255, 255, 255]));
            }
        }
    }

    // Convert to DynamicImage for overlay operation
    let mut base_image = DynamicImage::ImageRgb8(qr_image);

    // Overlay the icon
    imageops::overlay(&mut base_image, &icon, x_offset as i64, y_offset as i64);

    Ok(base_image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_qr_generation() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_qr.png");

        // Create a simple test icon (1x1 white pixel)
        let test_icon = DynamicImage::new_rgb8(1, 1);
        let icon_path = temp_dir.path().join("test_icon.png");
        test_icon.save(&icon_path).unwrap();

        // Test QR generation
        let result = generate_qr_with_icon(
            "https://example.com",
            icon_path.to_str().unwrap(),
            output_path.to_str().unwrap()
        );

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_icon_loading_and_resizing() {
        let temp_dir = tempdir().unwrap();

        // Create a test icon
        let test_icon = DynamicImage::new_rgb8(100, 100);
        let icon_path = temp_dir.path().join("test_icon.png");
        test_icon.save(&icon_path).unwrap();

        // Test loading and resizing
        let result = load_and_resize_icon(icon_path.to_str().unwrap(), 50);
        assert!(result.is_ok());

        let resized = result.unwrap();
        assert_eq!(resized.width(), 50);
        assert_eq!(resized.height(), 50);
    }
}