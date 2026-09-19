//! Local raster selection, preview, processing, and image output.
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
use std::io::Cursor;
use tease::{ImageData, Input, Output, Preset, Teaser, Value};
fn encode(image: &DynamicImage) -> Result<ImageData, String> {
    let mut out = Cursor::new(Vec::new());
    image
        .write_to(&mut out, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    ImageData::from_bytes(out.get_ref())
}
fn sample() -> ImageData {
    let pixels = ImageBuffer::from_fn(128, 128, |x, y| Rgb([(x * 2) as u8, (y * 2) as u8, 128]));
    encode(&DynamicImage::ImageRgb8(pixels)).unwrap()
}
fn demo() -> Teaser {
    Teaser::default().with_title("Local image processing")
        .with_description("Select a PNG/JPEG or load the built-in gradient. The image stays local. Toggle grayscale and adjust contrast, then compare the result.")
        .with_inputs(vec![Input::image("Source image").with_help("PNG/JPEG, at most 8 MiB and 4096×4096 pixels."),Input::checkbox("Grayscale",true),Input::Slider{label:Some("Contrast".into()),min:-50.0,max:50.0,step:1.0,initial_value:0.0}])
        .with_outputs(vec![Output::image("Processed image"),Output::text("Dimensions")])
        .with_presets(vec![Preset::new("Gradient",vec![Value::Image(sample()),Value::Bool(true),Value::Number(0.0)])])
        .with_comparison(true)
        .with_predictor(|values| {
            let bytes=values[0].as_image()?.bytes()?;
            let mut image=image::load_from_memory(&bytes).map_err(|e|e.to_string())?;
            if values[1].as_bool()? { image=image.grayscale(); }
            image=image.adjust_contrast(values[2].as_number()? as f32);
            let dimensions=format!("{} × {} pixels",image.width(),image.height());
            Ok(vec![Value::Image(encode(&image)?),Value::Text(dimensions)])
        })
}
fn main() -> Result<(), tease::Error> {
    demo().run()
}
#[test]
fn gradient_processing_returns_a_valid_grayscale_image() {
    let result = demo()
        .prepare()
        .unwrap()
        .predict(&[
            Value::Image(sample()),
            Value::Bool(true),
            Value::Number(0.0),
        ])
        .unwrap();
    let decoded = image::load_from_memory(&result[0].as_image().unwrap().bytes().unwrap())
        .unwrap()
        .to_rgb8();
    assert_eq!(decoded.dimensions(), (128, 128));
    let pixel = decoded.get_pixel(20, 60);
    assert_eq!(pixel[0], pixel[1]);
    assert_eq!(pixel[1], pixel[2]);
}
