pub(crate) fn add_number(index: usize, initial_value: &f64) -> String {
    format!("\
    <label for=\"exampleInput{index}\" class=\"col-sm-2 col-form-label mt-3\"><i>x<sub>{index}</sub> = </i></label>\
    <div class=\"col-sm-10 mt-3\">\
        <input type=\"text\" class=\"form-control input\" id=\"exampleInput{index}\" name=\"x{index}\" aria-describedby=\"input6\" placeholder=\"x{index}\" value=\"{initial_value}\">\
    </div>")
}

pub(crate) fn add_text(index: usize, initial_value: &str) -> String {
    format!("\
    <label for=\"exampleInput{index}\" class=\"col-sm-2 col-form-label mt-3\"><i>x<sub>{index}</sub> = </i></label>\
    <div class=\"col-sm-10 mt-3\">\
        <input type=\"text\" class=\"form-control input\" id=\"exampleInput{index}\" name=\"x{index}\" aria-describedby=\"input6\" placeholder=\"x{index}\" value=\"{initial_value}\">\
    </div>")
}

pub(crate) fn add_slider(
    index: usize,
    initial_value: &f64,
    max: &f64,
    min: &f64,
    step: &f64,
) -> String {
    format!("\
    <label for=\"exampleInput{index}\" class=\"col-sm-2 col-form-label mt-3\"><i>x<sub>{index}</sub> = </i></label>\
    <div class=\"col-sm-10 mt-3 form-group\" style=\"display: flex\">\
        <input type=\"text\" class=\"form-control col-sm-2\" value=\"{initial_value}\" readonly>
        <span class=\"col-sm-1\" ></span>
        <input type=\"range\" class=\"form-control input col-sm-9\" min=\"{min}\" max=\"{max}\" step=\"{step}\" id=\"exampleInput{index}\" name=\"x{index}\" aria-describedby=\"input6\" placeholder=\"x{index}\" value=\"{initial_value}\" oninput=\"this.previousElementSibling.previousElementSibling.value = this.value\">\
    </div>")
}

pub(crate) fn add_dropdown(
    index: usize,
    initial_value_index: &usize,
    options: &Vec<f64>,
) -> String {
    let mut output = format!("\
    <label for=\"exampleInput{index}\" class=\"col-sm-2 col-form-label mt-3\"><i>x<sub>{index}</sub> = </i></label>\
    <div class=\"col-sm-10 mt-3\">\
        <select class=\" form-control input\" aria-label=\"Default select example\">");

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
    output = format!("{}, </select></div>", output);
    output
}

pub(crate) fn beginning(description: String) -> String {
    format!("<html lang=\"en\">
        <head>
            <meta charset=\"utf-8\">
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1, shrink-to-fit=no\">
            <link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/bootstrap@4.3.1/dist/css/bootstrap.min.css\" integrity=\"sha384-ggOyR0iXCbMQv3Xipma34MD+dH/1fQ784/j6cY/iJTQUOhcWr7x9JvoRxT2MZw1T\" crossorigin=\"anonymous\">
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
                <div class=\"row mt-3\">
                    <div class=\"col text-center\">
                        <form action=\"#\" method=\"POST\" onsubmit=\"run_calculation()\">
                            <div class=\"form-group row\" id=\"input-group\">")
}

pub(crate) fn end() -> String {
    "                        </div>

                            <div class=\"form-group\" id=\"submit\">
                                <button type=\"submit\" class=\"btn btn-primary\">Submit</button>
                            </div>

                            <div class=\"form-group row\" id=\"output-group\">
                                <label for=\"output\" class=\"col-sm-2 col-form-label\"><i>y = </i></label>
                                <div class=\"col-sm-10\">
                                    <input type=\"text\" class=\"form-control\" id=\"output\" name=\"output\" aria-describedby=\"output\" readonly>
                                </div>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
            <script src=\"https://code.jquery.com/jquery-3.3.1.slim.min.js\" integrity=\"sha384-q8i/X+965DzO0rT7abK41JStQIAqVgRVzpbzo5smXKp4YfRvH+8abtTE1Pi6jizo\" crossorigin=\"anonymous\"></script>
            <script src=\"https://cdn.jsdelivr.net/npm/popper.js@1.14.7/dist/umd/popper.min.js\" integrity=\"sha384-UO2eT0CpHqdSJQ6hJty5KVphtPhzWj9WO1clHTMGa3JDZwrnQq4sF86dIHNDz0W1\" crossorigin=\"anonymous\"></script>
            <script src=\"https://cdn.jsdelivr.net/npm/bootstrap@4.3.1/dist/js/bootstrap.min.js\" integrity=\"sha384-JjSmVgyd0p3pXB1rRibZUAYoIIy6OrQ6VrjIEaFf/nJGzIxFDsf4x0xIM+B07jRM\" crossorigin=\"anonymous\"></script>
        </body>
    </html>".to_string()
}
