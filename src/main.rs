use image::{ImageBuffer, Rgba, RgbaImage, Luma, open};
use std::collections::VecDeque;
use std::path::Path;

fn main() {
    // Load the image
    let img = open(Path::new("input.jpeg")).expect("Failed to open image").to_rgba8();
    let (width, height) = img.dimensions();

    // Initialize the mask to be the same size as the image, filled with black (meaning no transparency)
    let mut mask = ImageBuffer::from_pixel(width, height, Luma([0]));

    // Define a tolerance for color matching; pixels within this tolerance to white will be considered as background
    let tolerance = 10;

    // Perform flood fill from the corners to create a mask.
    // Corners are top-left, top-right, bottom-left, bottom-right.
    // We assume the corners are part of the background.
    flood_fill(&img, &mut mask, 0, 0, tolerance); // top-left
    flood_fill(&img, &mut mask, width - 1, 0, tolerance); // top-right
    flood_fill(&img, &mut mask, 0, height - 1, tolerance); // bottom-left
    flood_fill(&img, &mut mask, width - 1, height - 1, tolerance); // bottom-right

    // Save the mask to check what has been filled
    // mask.save(Path::new("/mnt/data/mask.png")).expect("Failed to save mask");

    // Apply the mask to the image.
    let mut output_img = RgbaImage::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels() {
        let mask_pixel = mask.get_pixel(x, y)[0];
        let alpha = if mask_pixel == 0 { 255 } else { 0 }; // If mask pixel is black, keep the image pixel opaque.
        output_img.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], alpha]));
    }

    // Save the modified image with the background removed
    output_img.save(Path::new("output.png")).expect("Failed to save image with transparency");
}

// Tolerance-based flood fill algorithm
fn flood_fill(image: &RgbaImage, mask: &mut ImageBuffer<Luma<u8>, Vec<u8>>, start_x: u32, start_y: u32, tolerance: u8) {
    let mut queue = VecDeque::new();
    queue.push_back((start_x, start_y));

    while let Some((x, y)) = queue.pop_front() {
        // Check if the pixel is already filled
        if mask.get_pixel(x, y)[0] != 0 {
            continue;
        }

        let image_pixel = image.get_pixel(x, y);

        // Check if the pixel color is within the tolerance range of white
        if is_within_tolerance(image_pixel, tolerance) {
            mask.put_pixel(x, y, Luma([255]));

            // Add neighboring pixels to the queue
            if x > 0 { queue.push_back((x - 1, y)); }
            if y > 0 { queue.push_back((x, y - 1)); }
            if x < image.width() - 1 { queue.push_back((x + 1, y)); }
            if y < image.height() - 1 { queue.push_back((x, y + 1)); }
        }
    }
}

// Helper function to determine if a pixel is within the tolerance range of white
fn is_within_tolerance(pixel: &Rgba<u8>, tolerance: u8) -> bool {
    let Rgba([r, g, b, _]) = *pixel;
    let white = 255;
    
    r > white - tolerance && g > white - tolerance && b > white - tolerance
}

