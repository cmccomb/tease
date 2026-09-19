use std::io::Cursor;
use tease::{ImageData, Input, Output, Table, Teaser, Value};
#[test]
fn raster_bytes_are_validated_not_just_the_mime_label() {
    let image = image::DynamicImage::new_rgb8(2, 3);
    let mut data = Cursor::new(Vec::new());
    image.write_to(&mut data, image::ImageFormat::Png).unwrap();
    assert!(ImageData::from_bytes(&data.get_ref()[..16]).is_err());
    let mut oversized = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(4097, 1)
        .write_to(&mut oversized, image::ImageFormat::Png)
        .unwrap();
    assert!(ImageData::from_bytes(oversized.get_ref()).is_err());
    let raster = ImageData::from_bytes(data.get_ref()).unwrap();
    assert_eq!(raster.bytes().unwrap(), *data.get_ref());
    let mut invalid = raster.clone();
    invalid.mime = "image/jpeg".into();
    assert!(invalid.bytes().is_err());
    assert!(ImageData::from_bytes(b"<svg onload='alert(1)'/>").is_err());
    assert!(ImageData {
        mime: "image/png".into(),
        base64: "bad!".into()
    }
    .bytes()
    .is_err());
    let session = Teaser::default()
        .with_inputs(vec![Input::image("Image")])
        .with_output(Output::image("Image"))
        .with_predictor(|v| Ok(v.to_vec()))
        .prepare()
        .unwrap();
    assert!(session.predict(&[Value::Image(raster)]).is_ok());
    assert!(session.predict(&[Value::Image(invalid)]).is_err());
}
#[test]
fn csv_quotes_unicode_and_newlines_round_trip_and_shape_is_checked() {
    let text = "name,note\r\n\"🦀,Rust\",\"two\nlines and \"\"quotes\"\"\"\r\n";
    let table = Table::from_csv(text).unwrap();
    assert_eq!(table.rows[0][0], "🦀,Rust");
    assert_eq!(Table::from_csv(&table.to_csv().unwrap()).unwrap(), table);
    for text in ["a,a\n1,2\n", "a,b\n1\n", ",b\n1,2\n", ""] {
        assert!(Table::from_csv(text).is_err(), "{text}");
    }
    let too_large = format!("a\n{}", "1\n".repeat(1001));
    assert!(Table::from_csv(&too_large).is_err());
    let quoted = Table {
        columns: vec!["x".into()],
        rows: vec![vec!["\"".repeat(8192)]; 200],
    };
    let too_large_preset = Teaser::default()
        .with_inputs(vec![Input::table("Data")])
        .with_output(Output::table("Data"))
        .with_predictor(|v| Ok(v.to_vec()))
        .with_presets(vec![tease::Preset::new(
            "Oversize CSV",
            vec![Value::Table(quoted)],
        )]);
    assert!(too_large_preset.validate().is_err());
    let invalid = Table {
        columns: vec!["a".into()],
        rows: vec![vec![]],
    };
    assert!(invalid.to_csv().is_err());
}
#[test]
fn batch_api_reuses_model_and_retains_row_errors() {
    let session = Teaser::default()
        .with_function(|v| 2.0 * v[0])
        .prepare()
        .unwrap();
    let results = session.predict_batch(&[
        vec![Value::Number(2.0)],
        vec![Value::Text("bad".into())],
        vec![Value::Number(3.0)],
    ]);
    assert_eq!(results[0].as_ref().unwrap(), &[Value::Number(4.0)]);
    assert!(results[1].is_err());
    assert_eq!(results[2].as_ref().unwrap(), &[Value::Number(6.0)]);
}
