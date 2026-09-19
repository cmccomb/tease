//! Analytical response curves, model-supplied probabilities, and scenario comparison.
use tease::{Choice, ClassScore, Input, Output, Plot, PlotKind, Preset, Series, Teaser, Value};
fn scenario(gain: f64, name: &str) -> Vec<Value> {
    vec![
        Value::Number(gain),
        Value::Number(0.0),
        Value::Text(name.into()),
        Value::Bool(false),
        Value::List(vec![
            Value::Text("response".into()),
            Value::Text("baseline".into()),
        ]),
        Value::Text("line".into()),
    ]
}
fn demo() -> Teaser {
    Teaser::default().with_title("Explore a response model")
        .with_description("A logistic response model. Compare curves and pin scenarios; probabilities are analytical model outputs, not measured classification accuracy.")
        .with_inputs(vec![
            Input::Number { label: Some("Gain".into()), initial_value: 1.0, min: Some(0.0), max: Some(5.0) }.with_unit("×").with_help("Larger gain makes the transition steeper."),
            Input::Slider { label: Some("Offset".into()), min: -3.0, max: 3.0, step: 0.1, initial_value: 0.0 },
            Input::text("Scenario notes", "Default response", true), Input::checkbox("Reverse response", false),
            Input::MultiSelect { label: Some("Visible series".into()), options: vec![Choice::text("Response", "response"), Choice::text("Reference", "baseline")], initial_values: vec![0,1] }.with_help("Use Command/Ctrl to select more than one series."),
            Input::Dropdown { label: Some("Plot style".into()), options: vec![Choice::text("Line", "line"),Choice::text("Scatter", "scatter"),Choice::text("Bar", "bar")], initial_value: 0 },
        ])
        .with_outputs(vec![Output::number("Response at x = 1", 4), Output::text("Notes"), Output::plot("Response curve"), Output::probabilities("Outcomes at x = 1")])
        .with_presets(vec![Preset::new("Gentle", scenario(0.5,"Gentle transition")),Preset::new("Steep", scenario(3.0,"Steep transition"))])
        .with_comparison(true)
        .with_predictor(|v| {
            let gain=v[0].as_number()?; let offset=v[1].as_number()?; let reverse=v[3].as_bool()?;
            let response=|x:f64| { let p=1.0/(1.0+(-gain*x-offset).exp()); if reverse {1.0-p} else {p} };
            let mut series=Vec::new();
            for item in v[4].as_list()? {
                let name=item.as_text()?;
                series.push(Series { name: name.into(), points:(0..=40).map(|i| { let x=-3.0+f64::from(i)*0.15; [x,if name=="baseline" {0.5} else {response(x)}] }).collect() });
            }
            if series.is_empty() { return Err("Select at least one series.".into()); }
            let kind=match v[5].as_text()? {"scatter"=>PlotKind::Scatter,"bar"=>PlotKind::Bar,_=>PlotKind::Line};
            let p=response(1.0);
            Ok(vec![Value::Number(p),v[2].clone(),Value::Plot(Plot {kind,x_label:"Input x".into(),y_label:"Response probability".into(),series}),
                Value::Scores(vec![ClassScore{label:"Positive".into(),score:p},ClassScore{label:"Negative".into(),score:1.0-p}])])
        })
}
fn main() -> Result<(), tease::Error> {
    demo().run()
}
#[test]
fn documented_scenarios_match_the_model() {
    let session = demo().prepare().unwrap();
    let gentle = session.predict(&scenario(0.5, "Gentle")).unwrap();
    let steep = session.predict(&scenario(3.0, "Steep")).unwrap();
    assert!((gentle[0].as_number().unwrap() - 0.6224593312).abs() < 1e-9);
    assert!((steep[0].as_number().unwrap() - 0.9525741268).abs() < 1e-9);
    let Value::Plot(plot) = &gentle[2] else {
        panic!()
    };
    assert_eq!(plot.series[0].points.len(), 41);
}
