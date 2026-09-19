# Tease example guide

Run each example from a checkout using `cargo run --locked --example NAME`.
All numerical inputs must be finite. Model preparation happens once on the
inference worker; controls become available when the model is ready. Predictions
use a snapshot of the submitted fields, and controls remain disabled until the
result arrives. The native window stays responsive throughout.

| Example | Purpose | Try this | Expected result |
| --- | --- | --- | --- |
| `minimal` | Default identity function | Input 7 | 7.00 |
| `addition` | Numerical closure | 2 and 3 | 5.00 |
| `addition_without_closure` | Named Rust function | 2 and 3 | 5.00 |
| `classification` | Typed labels, dropdowns, multiple outputs | Score 0.7, Standard | Positive; margin 0.20 |
| `division` | Validation and model error recovery | 10 / 0, then 10 / 4 | Error, then 2.500 and Calculated |
| `slow` | Initialization, busy state, closing during work | Value 3, delay 3 seconds | 9.00 after the delay |
| `smartcore` | Real linear regression, Longley employment data | Leave defaults | 60.056 |
| `smartcore_classification` | Real Gaussian naive Bayes, Iris data | Leave defaults | Iris setosa |
| `automl` | Published AutoML regression wrapper | Leave defaults | 206.12 |

## Visual exploration (0.3)

Run `cargo run --locked --example exploration`. The analytical logistic model shows
plots, probabilities, text, checkboxes, multi-selects, units/help, presets, and pinned
comparisons. Load Gentle and predict: response at x=1 is `0.6225`. Pin that result,
load Steep and predict: `0.9526`, a displayed difference of `+0.3301`. Try all three
plot styles. Selecting no series produces a recoverable error. Scores describe this
analytical model, not measured accuracy or calibrated real-world uncertainty.

## SmartCore regression

The six Longley features are GNP deflator, GNP, unemployment, armed forces,
population, and year. Values retain the scaling in the embedded historical
dataset, including employment targets around 60–70. The default sample is
`[83, 234.289, 235.6, 159, 107.608, 1947]`. The predicted result is approximately
`60.055659`. Move the armed-forces slider to 250.2 for approximately `59.113`.

This demonstrates model integration; it does not establish a causal relationship
between a changed feature and employment. Training uses fixed data and no randomness.

## SmartCore classification

Inputs are sepal length, sepal width, petal length, and petal width, in centimeters.
These checked samples exercise all three predicted species:

| Measurements | Expected species |
| --- | --- |
| 5.1, 3.5, 1.4, 0.2 | Iris setosa |
| 6.0, 2.9, 4.5, 1.5 | Iris versicolor |
| 6.9, 3.1, 5.4, 2.1 | Iris virginica |

The model fits the complete embedded Iris dataset. These are integration examples,
not held-out estimates of classification accuracy. Inputs are constrained to
0.1–10 cm; these broad demonstration bounds do not certify biological plausibility.

## AutoML regression

The diabetes dataset supplies ten standardized features, in its original feature
order. The interface uses the dataset's feature names and first row as defaults.
The expected default prediction is approximately `206.117070` progression units.
For bounded startup and reproducibility, this example restricts AutoML to linear
regression with three unshuffled cross-validation folds. It demonstrates the wrapper
and worker-local initialization, rather than a full algorithm search.

AutoML 0.3.1 contains non-Send model state; its model is constructed inside
`with_predictor_factory` and remains on the inference thread. This example is for
research data demonstration and is not a clinical decision tool.

## Errors and long-running work

In `division`, a blank divisor triggers field validation, a negative divisor
violates its configured bound, and zero returns a model error. Correcting the field
allows another prediction without losing the other input.

In `slow`, preparation pauses for half a second and predictions wait for the chosen
0–5 second delay. A negative value produces an error after the delay. Drag, resize,
or close the window while it computes. Duplicate submissions are ignored and old
responses cannot overwrite a new request. Closing exits without waiting; callback
completion is not guaranteed. `slow` uses an artificial delay to make this observable.
