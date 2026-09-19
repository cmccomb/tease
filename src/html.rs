use crate::{Input, Teaser};
use std::fmt::Write;

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub(crate) fn render(demo: &Teaser) -> String {
    let mut fields = String::new();
    for (index, input) in demo.inputs.iter().enumerate() {
        let label = escape(&input.label(index));
        write!(
            fields,
            "<div class=\"field\"><label for=\"input-{index}\">{label}</label>"
        )
        .unwrap();
        match input {
            Input::Number {
                initial_value,
                min,
                max,
                ..
            } => {
                let bounds = format!(
                    "{}{}",
                    min.map(|n| format!(" min=\"{n}\"")).unwrap_or_default(),
                    max.map(|n| format!(" max=\"{n}\"")).unwrap_or_default()
                );
                write!(fields, "<input data-input id=\"input-{index}\" type=\"number\" step=\"any\" value=\"{initial_value}\"{bounds} required aria-describedby=\"error-{index}\">").unwrap();
            }
            Input::Slider {
                min,
                max,
                step,
                initial_value,
                ..
            } => {
                write!(fields, "<input data-input id=\"input-{index}\" type=\"range\" min=\"{min}\" max=\"{max}\" step=\"{step}\" value=\"{initial_value}\" aria-describedby=\"error-{index}\"><output class=\"slider-value\" for=\"input-{index}\">{initial_value}</output>").unwrap();
            }
            Input::Dropdown {
                options,
                initial_value,
                ..
            } => {
                write!(
                    fields,
                    "<select data-input id=\"input-{index}\" aria-describedby=\"error-{index}\">"
                )
                .unwrap();
                for (choice_index, choice) in options.iter().enumerate() {
                    let selected = if choice_index == *initial_value {
                        " selected"
                    } else {
                        ""
                    };
                    write!(
                        fields,
                        "<option value=\"{choice_index}\"{selected}>{}</option>",
                        escape(&choice.label)
                    )
                    .unwrap();
                }
                fields.push_str("</select>");
            }
        }
        write!(fields, "<p class=\"error\" id=\"error-{index}\"></p></div>").unwrap();
    }
    let mut outputs = String::new();
    for (index, output) in demo.outputs.iter().enumerate() {
        write!(outputs, "<div class=\"field\"><label for=\"output-{index}\">{}</label><output data-output id=\"output-{index}\"></output></div>", escape(&output.label(index))).unwrap();
    }
    format!(
        r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title><style>{style}</style></head><body><main>
<h1>{title}</h1><p class="description">{description}</p>
<div class="panels"><form id="tease-form" novalidate>{fields}<button id="submit" type="submit">Predict</button></form>
<section aria-label="Results" aria-live="polite">{outputs}<p id="status" role="status">Ready.</p></section></div>
<p id="error" class="error" role="alert" tabindex="-1"></p>
</main><script>{script}</script></body></html>"#,
        title = escape(&demo.title),
        description = escape(&demo.description),
        style = include_str!("ui.css"),
        script = include_str!("ui.js")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Choice, Output};

    #[test]
    fn text_is_escaped_in_every_component() {
        let payload = "\"><script>window.pwned=true</script>&'";
        let demo = Teaser::default()
            .with_title(payload)
            .with_description(payload)
            .with_inputs(vec![Input::Dropdown {
                label: Some(payload.into()),
                initial_value: 0,
                options: vec![Choice::text(payload, payload)],
            }])
            .with_output(Output::text(payload));
        let html = render(&demo);
        assert!(!html.contains(payload));
        assert!(html.contains("&lt;script&gt;window.pwned=true&lt;/script&gt;"));
        assert!(html.contains("for=\"input-0\""));
        assert!(html.contains("id=\"input-0\""));
    }
}
