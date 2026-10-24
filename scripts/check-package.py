#!/usr/bin/env python3
"""Verify the crate archive, then compile and run an isolated headless consumer."""
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile

root = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=root))
package = next(p for p in metadata["packages"] if Path(p["manifest_path"]).parent == root)
name = f'{package["name"]}-{package["version"]}'
target = Path(metadata["target_directory"])
subprocess.run(["cargo", "package", "--locked", "--allow-dirty"], cwd=root, check=True)
archive_path = target / "package" / f"{name}.crate"

with tempfile.TemporaryDirectory(prefix="tease-package-") as scratch:
    scratch = Path(scratch)
    unpacked = scratch / "unpacked"
    with tarfile.open(archive_path) as archive:
        members = archive.getmembers()
        names = {member.name for member in members}
        required = ["Cargo.toml", "Cargo.lock", "LICENSE-MIT", "LICENSE-APACHE", "README.md",
                    "CHANGELOG.md", "RELEASING.md", "docs/migration-0.2.md", "docs/examples.md",
                    "docs/guide.md", "docs/preview-response.png", "docs/preview-image.png",
                    "docs/index.html", "docs/live/index.html", "docs/live/model.js",
                    "docs/live/focus.css", "docs/live/focus.js",
                    "docs/rustdoc-preview.md",
                    "src/ui.js", "src/render.js", "src/files.js", "src/media.rs", "src/rich.rs", "src/worker.rs", "examples/smartcore_classification.rs"]
        missing = [path for path in required if f"{name}/{path}" not in names]
        if missing:
            raise RuntimeError(f"Package missing required files: {missing}")
        for member in members:
            destination = (unpacked / member.name).resolve()
            if not destination.is_relative_to(unpacked.resolve()):
                raise RuntimeError(f"Invalid archive path: {member.name}")
            if member.isdir():
                destination.mkdir(parents=True, exist_ok=True)
            elif member.isfile():
                destination.parent.mkdir(parents=True, exist_ok=True)
                with archive.extractfile(member) as source:
                    destination.write_bytes(source.read())
            else:
                raise RuntimeError(f"Unsupported archive member: {member.name}")
        if any(f"/{part}/" in path for path in names for part in ["assets", "doc_chunks", "bootstrap", ".github", "target"]):
            raise RuntimeError("Package contains obsolete assets or build/CI artifacts")

    consumer = scratch / "consumer"
    (consumer / "src").mkdir(parents=True)
    (consumer / "Cargo.toml").write_text(
        '[package]\nname = "tease-package-smoke"\nversion = "0.0.0"\n'
        'edition = "2021"\nrust-version = "1.88"\n\n[dependencies]\n'
        f'tease = {{ path = {json.dumps(str(unpacked / name))} }}\n')
    (consumer / "src" / "main.rs").write_text('''
use std::{cell::Cell, rc::Rc};
use tease::{Choice, Input, Output, Teaser, Value};
fn main() -> Result<(), tease::Error> {
    let simple = Teaser::default().with_function(|x| x.iter().sum()).prepare()?;
    assert_eq!(simple.predict(&[Value::Number(3.0)])?, vec![Value::Number(3.0)]);
    let session = Teaser::default()
        .with_inputs(vec![Input::number("Value", 2.0), Input::Dropdown {
            label: Some("Category".into()), initial_value: 0,
            options: vec![Choice::text("Alpha", "alpha")],
        }])
        .with_outputs(vec![Output::number("Value", 2), Output::text("Category")])
        .with_predictor_factory(|| {
            let calls = Rc::new(Cell::new(0));
            Ok(move |values: &[Value]| {
                calls.set(calls.get() + 1);
                let number = values[0].as_number()?;
                if number < 0.0 { return Err("Negative value.".into()); }
                Ok(vec![Value::Number(number), Value::Text(values[1].as_text()?.into())])
            })
        }).prepare()?;
    assert!(session.predict(&[Value::Number(-1.0), Value::Text("alpha".into())]).is_err());
    assert_eq!(session.predict(&[Value::Number(2.0), Value::Text("alpha".into())])?,
        vec![Value::Number(2.0), Value::Text("alpha".into())]);
    let plot = tease::Plot { kind: tease::PlotKind::Line, x_label: "x".into(), y_label: "y".into(), series: vec![tease::Series { name: "line".into(), points: vec![[0.0,1.0],[1.0,2.0]] }] };
    let rich = Teaser::default().with_inputs(vec![Input::text("Note", "", true), Input::checkbox("Enabled", true)])
        .with_presets(vec![tease::Preset::new("Example",vec![Value::Text("test".into()),Value::Bool(true)])])
        .with_comparison(true).with_outputs(vec![Output::plot("Plot"),Output::probabilities("Classes")])
        .with_predictor(move |_| Ok(vec![Value::Plot(plot.clone()),Value::Scores(vec![tease::ClassScore { label: "A".into(), score: 1.0 }])])).prepare()?;
    assert_eq!(rich.predict(&[Value::Text("test".into()),Value::Bool(true)])?.len(),2);
    let table=tease::Table {columns:vec!["x".into()],rows:vec![vec!["1".into()]]};
    assert_eq!(tease::Table::from_csv(&table.to_csv().unwrap()).unwrap(),table);
    let png: &[u8] = &[137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82,0,0,0,1,0,0,0,1,8,2,0,0,0,144,119,83,222,0,0,0,12,73,68,65,84,120,156,99,112,104,56,0,0,2,132,1,129,38,151,167,43,0,0,0,0,73,69,78,68,174,66,96,130];
    let image=tease::ImageData::from_bytes(png).unwrap(); assert_eq!(image.bytes().unwrap(),png);
    let tables=Teaser::default().with_inputs(vec![Input::table("Data")]).with_output(Output::table("Data")).with_predictor(|v|Ok(v.to_vec())).prepare()?;
    assert_eq!(tables.predict(&[Value::Table(table.clone())])?,vec![Value::Table(table)]);
    Teaser::default().with_live_prediction(200).validate()?;
    println!("Packaged Tease consumer passed.");
    Ok(())
}
''')
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target / "package-consumer")
    subprocess.run(["cargo", "run", "--manifest-path", str(consumer / "Cargo.toml")],
                   cwd=consumer, env=env, check=True)
print(f"Verified {archive_path.name}: {len(names)} files and an isolated consumer.")
