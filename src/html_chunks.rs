pub(crate) fn add_number(index: usize, initial_value: &f64, label: &Option<String>) -> String {
    let html_label = match label {
        None => {
            format!("Input {index}")
        }
        Some(string) => string.to_string(),
    };

    format!("\
    <label for=\"exampleInput{index}\" class=\"col-form-label mt-3\"><i>{html_label}</i></label>\
    <input type=\"text\" class=\"form-control input\" id=\"exampleInput{index}\" name=\"x{index}\" aria-describedby=\"input6\" placeholder=\"x{index}\" value=\"{initial_value}\">\
    ")
}

pub(crate) fn add_slider(
    index: usize,
    initial_value: &f64,
    max: &f64,
    min: &f64,
    step: &f64,
    label: &Option<String>,
) -> String {
    let html_label = match label {
        None => {
            format!("x<sub>{index}</sub> = ")
        }
        Some(string) => string.to_string(),
    };
    format!("\
    <label for=\"exampleInput{index}\" class=\"col-form-label mt-3\"><i>{html_label}</i></label>\
    <div class=\"form-group mb-0\" style=\"display: flex\">\
    <input type=\"text\" class=\"form-control col-sm-3 \" value=\"{initial_value}\" readonly>
    <span class=\"col-sm-1\" ></span>
    <input type=\"range\" class=\"form-control input col-sm-8\" min=\"{min}\" max=\"{max}\" step=\"{step}\" id=\"exampleInput{index}\" name=\"x{index}\" aria-describedby=\"input6\" placeholder=\"x{index}\" value=\"{initial_value}\" oninput=\"this.previousElementSibling.previousElementSibling.value = this.value\">\
   </div>")
}

pub(crate) fn add_dropdown(
    index: usize,
    initial_value_index: &usize,
    options: &Vec<f64>,
    label: &Option<String>,
) -> String {
    let html_label = match label {
        None => {
            format!("x<sub>{index}</sub> = ")
        }
        Some(string) => string.to_string(),
    };
    let mut output = format!(
        "\
    <label for=\"exampleInput{index}\" class=\"col-form-label mt-3\"><i>{html_label}</i></label>\
        <select class=\" form-control input\" aria-label=\"Default select example\">"
    );

    for (idx, option) in options.iter().enumerate() {
        if idx == *initial_value_index {
            output = format!(
                "{}<option selected value=\"{option}\">{option}</option>",
                output
            )
        } else {
            output = format!("{}<option value=\"{option}\">{option}</option>", output)
        }
    }
    output = format!("{}, </select>", output);
    output
}

pub(crate) fn beginning(description: String) -> String {
    let bootstrap = include_str!("bootstrap/bootstrap.min.css");
    format!("<html lang=\"en\">
        <head>
            <meta charset=\"utf-8\">
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1, shrink-to-fit=no\">
            <style>{bootstrap}</style>
            <script type=\"text/javascript\">
                function run_calculation() {{
                    var numbers = [];
                    var classes = document.getElementsByClassName('input');
                    Array.from(classes).forEach((x, i) => numbers.push(Number(x.value)));
                    ipc.postMessage(numbers.toString());
                }}
            </script>
        </head>
        <body>
            <div class=\"container\">
                <p class=\"mt-3 text-center\">{description}</p>
                <div class=\"row my-3\">
                    <div class=\"col text-center bg-light mr-1\">
                        <form class=\"m-3\" action=\"#\" method=\"POST\" onsubmit=\"run_calculation()\">
                            <div class=\"form-group row\" id=\"input-group\">")
}

pub(crate) fn end() -> String {
    let jquery = include_str!("bootstrap/jquery-3.3.1.slim.min.js");
    let popper = include_str!("bootstrap/popper.min.js");
    let bootstrap = include_str!("bootstrap/bootstrap.min.js");

    format!("                        </div>

                            <div class=\"form-group \" id=\"submit\">
                                <button type=\"submit\" class=\"btn btn-primary\">Submit</button>
                            </div>
                        </form>
                    </div>

                    <div class=\"col bg-light ml-1\">
                        <div class=\"form-group m-3\" id=\"output-group\">
                            <label for=\"output\" class=\"col-form-label mt-3\"><i>Result</i></label>
                            <input type=\"text\" class=\"form-control\" id=\"output\" name=\"output\" aria-describedby=\"output\" readonly>
                        </div>
                    </div>
                </div>
            </div>

            <script>{jquery}</script>
            <script>{popper}</script>
            <script>{bootstrap}</script>
        </body>
    </html>")
}
