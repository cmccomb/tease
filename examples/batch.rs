//! CSV batch inference with row-level error recovery and export.
use tease::{Input, Output, Preset, Table, Teaser, Value};
fn demo() -> Teaser {
    Teaser::default().with_title("CSV batch predictions")
        .with_description("Import a CSV with one column named x. The demonstration model computes y = 2x + 1. Invalid rows retain an error; valid rows still produce predictions.")
        .with_inputs(vec![Input::table("CSV data")])
        .with_output(Output::table("Predictions"))
        .with_presets(vec![Preset::new("Mixed rows",vec![Value::Table(Table::from_csv("x\n1\n2\ninvalid\n-3\n").unwrap())])])
        .with_predictor_factory(|| {
            let model=Teaser::default().with_function(|x|2.0*x[0]+1.0).prepare().map_err(|e|e.to_string())?;
            Ok(move |values:&[Value]| {
                let table=values[0].as_table()?;
                if table.columns!=["x"] {return Err("Use one CSV column named x.".into());}
                let inputs:Vec<_>=table.rows.iter().map(|row|vec![row[0].trim().parse::<f64>().map(Value::Number).unwrap_or_else(|_|Value::Text(row[0].clone()))]).collect();
                let rows=table.rows.iter().zip(model.predict_batch(&inputs)).map(|(row,result)|{
                    match result {
                        Ok(output)=>vec![row[0].clone(),output[0].as_number().unwrap().to_string(),String::new()],
                        Err(error)=>vec![row[0].clone(),String::new(),error.to_string()],
                    }
                }).collect();
                Ok(vec![Value::Table(Table{columns:vec!["x".into(),"prediction".into(),"error".into()],rows})])
            })
        })
}
fn main() -> Result<(), tease::Error> {
    demo().run()
}
#[test]
fn bad_rows_do_not_discard_successful_predictions() {
    let result = demo()
        .prepare()
        .unwrap()
        .predict(&[Value::Table(
            Table::from_csv("x\n1\ninvalid\n-3\n").unwrap(),
        )])
        .unwrap();
    let table = result[0].as_table().unwrap();
    assert_eq!(table.rows[0][1], "3");
    assert!(!table.rows[1][2].is_empty());
    assert_eq!(table.rows[2][1], "-5");
}
