use tease::{Choice, ClassScore, Input, Output, Plot, PlotKind, Preset, Series, Teaser, Value};
#[test]
fn rich_controls_validate_values_and_presets() {
    let fields = vec![
        Input::text("Notes", "", true),
        Input::checkbox("Flag", false),
        Input::MultiSelect {
            label: None,
            options: vec![Choice::text("A", "a"), Choice::text("B", "b")],
            initial_values: vec![],
        },
    ];
    let values = vec![
        Value::Text("🦀\n<literal>".into()),
        Value::Bool(true),
        Value::List(vec![Value::Text("a".into())]),
    ];
    let make = || {
        Teaser::default()
            .with_inputs(fields.clone())
            .with_output(Output::text("Notes"))
            .with_predictor(|v| Ok(vec![v[0].clone()]))
    };
    let session = make()
        .with_presets(vec![Preset::new("Example", values.clone())])
        .prepare()
        .unwrap();
    assert_eq!(session.predict(&values).unwrap()[0], values[0]);
    let mut invalid = values.clone();
    invalid[2] = Value::List(vec![Value::Text("a".into()); 2]);
    assert!(session.predict(&invalid).is_err());
    assert!(make()
        .with_presets(vec![Preset::new("Bad", invalid)])
        .validate()
        .is_err());
    assert!(make()
        .with_presets(vec![Preset::new("Bad count", vec![])])
        .validate()
        .is_err());
    assert!(Teaser::default()
        .with_inputs(vec![Input::checkbox("Flag", false)])
        .validate()
        .is_err());
    assert!(Teaser::default()
        .with_inputs(vec![Input::number("Distance", 2.0)
            .with_unit("m")
            .with_help("Length")])
        .prepare()
        .unwrap()
        .predict(&[Value::Number(3.0)])
        .is_ok());
}
#[test]
fn malformed_rich_results_are_rejected() {
    for points in [vec![], vec![[0.0, f64::NAN]], vec![[0.0, 1e101]]] {
        let value = Value::Plot(Plot {
            kind: PlotKind::Line,
            x_label: "x".into(),
            y_label: "y".into(),
            series: vec![Series {
                name: "test".into(),
                points,
            }],
        });
        let session = Teaser::default()
            .with_output(Output::plot("Plot"))
            .with_predictor(move |_| Ok(vec![value.clone()]))
            .prepare()
            .unwrap();
        assert!(session.predict(&[Value::Number(1.0)]).is_err());
    }
    for scores in [
        vec![],
        vec![ClassScore {
            label: "a".into(),
            score: -0.1,
        }],
        vec![ClassScore {
            label: "a".into(),
            score: 0.4,
        }],
        vec![
            ClassScore {
                label: "a".into(),
                score: 0.5
            };
            2
        ],
    ] {
        let session = Teaser::default()
            .with_output(Output::probabilities("Classes"))
            .with_predictor(move |_| Ok(vec![Value::Scores(scores.clone())]))
            .prepare()
            .unwrap();
        assert!(session.predict(&[Value::Number(1.0)]).is_err());
    }
}
#[test]
fn unicode_length_limits_preserve_empty_text() {
    let session = Teaser::default()
        .with_inputs(vec![Input::Text {
            label: None,
            initial_value: String::new(),
            multiline: true,
            max_length: 2,
        }])
        .with_output(Output::text("Text"))
        .with_predictor(|v| Ok(v.to_vec()))
        .prepare()
        .unwrap();
    for text in ["", "🦀é"] {
        assert!(session.predict(&[Value::Text(text.into())]).is_ok());
    }
    assert!(session.predict(&[Value::Text("abc".into())]).is_err());
}
