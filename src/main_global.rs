use image::{ImageBuffer, Rgba, RgbaImage, Luma, open};
use imageproc::filter::gaussian_blur_f32;
use imageproc::map::map_colors;

fn main() {
    // Load the image
    let mut img = open("input.jpeg").expect("Failed to open image").to_rgba8();
    let (width, height) = img.dimensions();

    // Create a mask based on white color
    let white_threshold = 235; // Adjust this threshold to your needs
    let blur_sigma = 1.0; // Adjust the blur amount to your needs

    // Create a mask image where white or near-white pixels are marked as white and all others as black
    let mask = ImageBuffer::from_fn(width, height, |x, y| {
        let p = img.get_pixel(x, y);
        if p[0] >= white_threshold && p[1] >= white_threshold && p[2] >= white_threshold {
            Luma([255u8])
        } else {
            Luma([0u8])
        }
    });

    // Apply Gaussian blur to the mask to create the feathering effect
    let blurred_mask = gaussian_blur_f32(&mask, blur_sigma);

    // Apply the blurred mask to the image, setting the alpha channel of each pixel
    let mut output_img = RgbaImage::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels() {
        let mask_pixel = blurred_mask.get_pixel(x, y)[0];
        // Calculate the new alpha channel based on the mask
        // The white parts of the mask (which correspond to the white background of the original image)
        // should be transparent, so we invert the mask value for the alpha channel
        let alpha = 255 - mask_pixel; // Invert mask value for alpha channel
        output_img.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], alpha]));
    }

    // Save the modified image with the feathered transparency
    output_img.save("output_with_transparency.png").expect("Failed to save image");
}

