use crate::{Input, Output, Teaser};
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
        let (unit, help) = match input {
            Input::Described { unit, help, .. } => {
                (unit.as_deref().unwrap_or(""), help.as_deref().unwrap_or(""))
            }
            _ => ("", ""),
        };
        let label = if unit.is_empty() {
            label
        } else {
            format!("{label} ({})", escape(unit))
        };
        write!(
            fields,
            "<div class=\"field\"><label for=\"input-{index}\">{label}</label>"
        )
        .unwrap();
        match input.base() {
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
                write!(fields, "<input data-input id=\"input-{index}\" type=\"number\" step=\"any\" value=\"{initial_value}\"{bounds} required aria-describedby=\"help-{index} error-{index}\">").unwrap();
            }
            Input::Slider {
                min,
                max,
                step,
                initial_value,
                ..
            } => {
                write!(fields, "<input data-input id=\"input-{index}\" type=\"range\" min=\"{min}\" max=\"{max}\" step=\"{step}\" value=\"{initial_value}\" aria-describedby=\"help-{index} error-{index}\"><output class=\"slider-value\" for=\"input-{index}\">{initial_value}</output>").unwrap();
            }
            Input::Dropdown {
                options,
                initial_value,
                ..
            } => {
                write!(
                    fields,
                    "<select data-input id=\"input-{index}\" aria-describedby=\"help-{index} error-{index}\">"
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
            Input::Text {
                initial_value,
                multiline,
                ..
            } => {
                if *multiline {
                    write!(fields, "<textarea data-input id=\"input-{index}\" rows=\"4\" aria-describedby=\"help-{index} error-{index}\">{}</textarea>", escape(initial_value)).unwrap();
                } else {
                    write!(fields, "<input data-input id=\"input-{index}\" type=\"text\" value=\"{}\" aria-describedby=\"help-{index} error-{index}\">", escape(initial_value)).unwrap();
                }
            }
            Input::Checkbox { initial_value, .. } => {
                write!(fields, "<input data-input id=\"input-{index}\" type=\"checkbox\" {} aria-describedby=\"help-{index} error-{index}\">", if *initial_value { "checked" } else { "" }).unwrap();
            }
            Input::MultiSelect {
                options,
                initial_values,
                ..
            } => {
                write!(fields, "<select data-input id=\"input-{index}\" multiple aria-describedby=\"help-{index} error-{index}\">").unwrap();
                for (i, choice) in options.iter().enumerate() {
                    write!(
                        fields,
                        "<option value=\"{i}\" {}>{}</option>",
                        if initial_values.contains(&i) {
                            "selected"
                        } else {
                            ""
                        },
                        escape(&choice.label)
                    )
                    .unwrap();
                }
                fields.push_str("</select>");
            }
            Input::Described { .. } => unreachable!(),
        }
        write!(
            fields,
            "<p class=\"help\" id=\"help-{index}\">{}</p>",
            escape(help)
        )
        .unwrap();
        write!(fields, "<p class=\"error\" id=\"error-{index}\"></p></div>").unwrap();
    }
    let mut outputs = String::new();
    for (index, output) in demo.outputs.iter().enumerate() {
        write!(outputs, "<div class=\"field\"><label for=\"output-{index}\">{}</label><output data-output id=\"output-{index}\"></output></div>", escape(&output.label(index))).unwrap();
    }
    let presets = demo.presets.iter().map(|p| serde_json::json!({"name":p.name,"inputs":demo.inputs.iter().zip(&p.inputs).map(|(i,v)|i.encode(v)).collect::<Vec<_>>()})).collect::<Vec<_>>();
    let config = serde_json::json!({"presets":presets,"comparison":demo.comparison,
        "inputLabels":demo.inputs.iter().enumerate().map(|(i,v)|v.label(i)).collect::<Vec<_>>(),
        "outputs":demo.outputs.iter().enumerate().map(|(i,v)|serde_json::json!({"label":v.label(i),"numeric":matches!(v, Output::Number { .. })})).collect::<Vec<_>>()});
    let config = config
        .to_string()
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e");
    let preset_control = if demo.presets.is_empty() {
        String::new()
    } else {
        format!("<label for=\"preset\">Example scenario</label><select id=\"preset\"><option value=\"\">Load a preset…</option>{}</select>", demo.presets.iter().enumerate().map(|(i,p)|format!("<option value=\"{i}\">{}</option>",escape(&p.name))).collect::<String>())
    };
    let comparison = if demo.comparison {
        "<section id=\"comparison\" aria-label=\"Scenario comparison\"><h2>Compare scenarios</h2><button id=\"pin\" type=\"button\" disabled>Pin result</button><p class=\"help\">Pin up to three results. Differences use the first pinned result as the baseline.</p><div id=\"scenarios\"></div></section>"
    } else {
        ""
    };
    format!(
        r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title><style>{style}</style></head><body><main>
<h1>{title}</h1><p class="description">{description}</p>
<div class="panels"><form id="tease-form" novalidate>{preset_control}{fields}<button id="submit" type="submit">Predict</button></form>
<section aria-label="Results" aria-live="polite">{outputs}<p id="status" role="status">Ready.</p></section></div>
<p id="error" class="error" role="alert" tabindex="-1"></p>
{comparison}</main><script type="application/json" id="tease-config">{config}</script><script>{render_script}</script><script>{script}</script></body></html>"#,
        title = escape(&demo.title),
        description = escape(&demo.description),
        style = include_str!("ui.css"),
        script = include_str!("ui.js"),
        render_script = include_str!("render.js")
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
