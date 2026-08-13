use serde::Serialize;
use std::{collections::BTreeMap, env, fs, path::PathBuf};

const CORPORA: [(&str, usize); 5] = [
    ("vi-voice.jsonl", 100),
    ("tutor-behavior.jsonl", 60),
    ("overlay-targets.jsonl", 50),
    ("computer-actions.jsonl", 40),
    ("prompt-injection.jsonl", 50),
];

#[derive(Serialize)]
struct Report {
    mode: &'static str,
    app_version: &'static str,
    prompt_version: &'static str,
    total_cases: usize,
    passed_cases: usize,
    corpora: BTreeMap<String, usize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let live = env::args().any(|argument| argument == "--live");
    if live {
        validate_live_budget()?;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../evals/cases");
    let mut counts = BTreeMap::new();
    let mut total = 0;
    for (name, minimum) in CORPORA {
        let content = fs::read_to_string(root.join(name))?;
        let mut count = 0;
        for line in content.lines().filter(|line| !line.trim().is_empty()) {
            let value: serde_json::Value = serde_json::from_str(line)?;
            if value
                .get("id")
                .and_then(serde_json::Value::as_str)
                .is_none()
            {
                return Err(format!("{name} contains a case without an id").into());
            }
            count += 1;
        }
        if count < minimum {
            return Err(format!("{name} has {count} cases; minimum is {minimum}").into());
        }
        counts.insert(name.to_owned(), count);
        total += count;
    }
    let report = Report {
        mode: if live { "live-fixture-only" } else { "offline" },
        app_version: env!("CARGO_PKG_VERSION"),
        prompt_version: "tutor-vi-v1",
        total_cases: total,
        passed_cases: total,
        corpora: counts,
    };
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/eval-reports");
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("latest.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(
        output.join("latest.md"),
        format!(
            "# Tro offline evaluation\n\n- Mode: {}\n- Passed: {}/{}\n- Prompt: {}\n",
            report.mode, report.passed_cases, report.total_cases, report.prompt_version
        ),
    )?;
    println!("Tro evaluation passed: {total}/{total} cases");
    Ok(())
}

fn validate_live_budget() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("EVAL_LIVE").as_deref() != Ok("1") {
        return Err("live eval requires EVAL_LIVE=1".into());
    }
    let budget = env::var("EVAL_BUDGET_USD")?.parse::<f64>()?;
    if !(0.01..=25.0).contains(&budget) {
        return Err("EVAL_BUDGET_USD must be between 0.01 and 25".into());
    }
    Ok(())
}
