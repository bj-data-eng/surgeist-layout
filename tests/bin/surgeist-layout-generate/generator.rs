use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;
use std::time::{Duration, Instant};

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::fetcher::{BrowserFetcher, BrowserFetcherOptions, BrowserKind, BrowserVersion};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

const ROOT_ENV: &str = "SURGEIST_LAYOUT_BROWSER_PARITY_ROOT";
const FILTER_ENV: &str = "SURGEIST_LAYOUT_GENERATE_FILTER";
const BROWSER_PATH_ENV: &str = "SURGEIST_BROWSER_PATH";
const BROWSER_CACHE_ENV: &str = "SURGEIST_BROWSER_CACHE";
const BROWSER_VERSION_ENV: &str = "SURGEIST_BROWSER_VERSION";
const DEFAULT_ROOT: &str = "tests/layout/browser_parity";
const DEFAULT_BROWSER_CACHE: &str = "target/surgeist-browser";
const DEFAULT_BROWSER_VERSION: &str = "149.0.7827.115";
const BROWSER_FIXTURE_BATCH_SIZE: usize = 50;
const BROWSER_NAVIGATION_TIMEOUT: Duration = Duration::from_secs(10);
const BROWSER_NAVIGATION_POLL_INTERVAL: Duration = Duration::from_millis(25);
const SOURCE_CACHE_DIR: &str = "target/surgeist-sources";
const TAFFY_REPO: &str = "https://github.com/DioxusLabs/taffy.git";
const TAFFY_COMMIT: &str = "d1ff7e339b9ee35b33858779f8d7653197e93d92";
const TAFFY_EXPECTED_COUNT: usize = 1103;
const TAFFY_SOURCE_DIR: &str = "test_fixtures";
const GENERATION_REPORT_BUCKETS: [&str; 5] = [
    "generated",
    "unsupported",
    "expected_fail",
    "quarantined",
    "failed_to_generate",
];
const TEST_HELPER_SOURCE: &str =
    include_str!("../../layout/browser_parity/scripts/gentest/test_helper.js");
const TEST_BASE_STYLE_SOURCE: &str =
    include_str!("../../layout/browser_parity/scripts/gentest/test_base_style.css");
const GRID_TEMPLATE_AREA_CAPTURE_SCRIPT: &str = r#"(() => {
  if (window.__surgeistGridTemplateAreaCaptureInstalled) return true;

  function parseSurgeistGridTemplateAreas(input) {
    if (!input || input === "none") return undefined;
    const rows = Array.from(input.matchAll(/"([^"]*)"/g), match => match[1].trim());
    if (rows.length === 0) return undefined;
    return rows.map(row => row.split(/\s+/).map(cell => /^\.+$/.test(cell) ? null : cell));
  }

  function authoredSurgeistGridTemplateAreas(element) {
    const computedStyle = getComputedStyle(element);
    if (element.style.gridTemplateAreas) return element.style.gridTemplateAreas;
    if (typeof authoredStyleValue === "function") {
      const authored = authoredStyleValue(element, "gridTemplateAreas", computedStyle);
      if (authored) return authored;
      if (computedStyle.gridTemplateAreas && computedStyle.gridTemplateAreas !== "none") {
        return computedStyle.gridTemplateAreas;
      }
    }
    return "";
  }

  const originalDescribeElement = describeElement;
  describeElement = function(element, expectedElement = null) {
    const data = originalDescribeElement(element, expectedElement);
    if (data && data.style) {
      data.style.gridTemplateAreas = parseSurgeistGridTemplateAreas(
        authoredSurgeistGridTemplateAreas(element)
      );
    }
    return data;
  };

  window.__surgeistGridTemplateAreaCaptureInstalled = true;
  return true;
})()"#;

pub async fn run_from_env() -> Result<(), String> {
    let config = Config::from_env()?;
    match env::args().nth(1).as_deref() {
        Some("check-corpus") => check_corpus(&config),
        Some("check-taffy-corpus") => check_taffy_corpus(&config),
        Some("import-taffy") => import_taffy(&config),
        Some(command) => Err(format!("unknown command `{command}`")),
        None => generate(config).await,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Config {
    root: PathBuf,
    html_root: PathBuf,
    xml_root: PathBuf,
    filter: Option<String>,
    browser_cache: PathBuf,
    browser_path: Option<PathBuf>,
    browser_version: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct CorpusManifest {
    imports: CorpusImports,
    #[serde(default)]
    cases: Vec<CorpusCase>,
}

#[derive(Clone, Debug, Deserialize)]
struct CorpusImports {
    taffy: CorpusTaffyImport,
}

#[derive(Clone, Debug, Deserialize)]
struct CorpusTaffyImport {
    repo: String,
    commit: String,
    source_dir: String,
    destination: String,
    expected_count: usize,
    #[serde(default)]
    allowed_extra_paths: Vec<String>,
    #[serde(default)]
    excluded_destination_dirs: Vec<String>,
    #[serde(flatten)]
    _extra: BTreeMap<String, toml::Value>,
}

#[derive(Clone, Debug, Deserialize)]
struct CorpusCase {
    id: String,
    source_root: String,
    source: String,
    generator: String,
    status: String,
    #[serde(default)]
    reason: Option<String>,
    #[serde(flatten)]
    _extra: BTreeMap<String, toml::Value>,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let root = match env::var_os(ROOT_ENV) {
            Some(path) => PathBuf::from(path),
            None => PathBuf::from(DEFAULT_ROOT),
        };
        Self::from_root(root)
    }

    fn from_root(root: PathBuf) -> Result<Self, String> {
        let browser_cache = env::var_os(BROWSER_CACHE_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BROWSER_CACHE));
        let browser_path = env::var_os(BROWSER_PATH_ENV).map(PathBuf::from);
        let browser_version = env::var(BROWSER_VERSION_ENV)
            .ok()
            .filter(|value| !value.is_empty())
            .or_else(|| Some(DEFAULT_BROWSER_VERSION.to_string()));
        let filter = env::var(FILTER_ENV).ok().filter(|value| !value.is_empty());
        Ok(Self {
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter,
            browser_cache,
            browser_path,
            browser_version,
            root,
        })
    }
}

async fn generate(config: Config) -> Result<(), String> {
    let mut report = GenerationReport {
        filter: config.filter.clone(),
        ..GenerationReport::default()
    };
    let constrained_fixtures = collect_constrained_fixtures_for_generation(&config, &mut report)?;
    let jobs = generation_jobs(constrained_fixtures);
    if jobs.is_empty() {
        if report.has_entries() {
            fs::create_dir_all(&config.xml_root).map_err(|error| {
                format!("failed to create {}: {error}", config.xml_root.display())
            })?;
            write_generation_report(&config, &report)?;
            return Ok(());
        }
        let scope = config.filter.as_ref().map_or_else(
            || format!("{}", config.html_root.display()),
            |filter| format!("parity sources matching {filter:?}"),
        );
        return Err(format!("no parity fixtures found under {scope}"));
    }

    let browser_path = resolve_browser_path(&config).await?;
    fs::create_dir_all(&config.xml_root)
        .map_err(|error| format!("failed to create {}: {error}", config.xml_root.display()))?;

    for (batch_index, batch) in jobs.chunks(BROWSER_FIXTURE_BATCH_SIZE).enumerate() {
        if let Err(error) =
            generate_batch(&config, &browser_path, batch, batch_index, &mut report).await
        {
            for job in batch {
                record_failed_generation_job(&config, job, &mut report, error.clone());
            }
        }
    }
    prune_stale_generated_xml_outputs_after_success(&config, &report)?;
    write_generation_report(&config, &report)?;
    if report.summary.failed_to_generate > 0 {
        return Err(format!(
            "{} generation job(s) failed; see {}",
            report.summary.failed_to_generate,
            generation_report_path(&config).display()
        ));
    }

    Ok(())
}

fn check_corpus(config: &Config) -> Result<(), String> {
    check_corpus_junk_files(config)?;
    check_gentest_helper_only_assets(config)?;
    validate_taffy_manifest(config)?;
    validate_surgeist_constrained_case_files(config)?;
    validate_generation_report_freshness(config)?;
    validate_xml_provenance_freshness(config)?;
    check_taffy_corpus_from_existing_source(config)?;
    Ok(())
}

fn check_taffy_corpus(config: &Config) -> Result<(), String> {
    check_corpus_junk_files(config)?;
    check_gentest_helper_only_assets(config)?;
    validate_taffy_manifest(config)?;
    validate_surgeist_constrained_case_files(config)?;
    check_taffy_corpus_from_existing_source(config)
}

fn check_taffy_corpus_from_existing_source(config: &Config) -> Result<(), String> {
    let source_root = taffy_source_root();
    if !source_root.is_dir() {
        return Err(format!(
            "missing pinned Taffy source at {}; run `cargo run -p surgeist-layout --features layout-golden-generate --bin surgeist-layout-generate -- import-taffy`",
            source_root.display()
        ));
    }
    validate_taffy_source_revision(&source_root)?;
    check_taffy_corpus_against_verified_source(config, &source_root, TAFFY_EXPECTED_COUNT)
}

fn check_taffy_corpus_against_verified_source(
    config: &Config,
    source_root: &Path,
    expected_count: usize,
) -> Result<(), String> {
    let allowed_extra_paths = manifest_surgeist_constrained_paths(config)?;

    let taffy_files = collect_relative_html(&source_root.join(TAFFY_SOURCE_DIR))?;
    if taffy_files.len() != expected_count {
        return Err(format!(
            "expected {expected_count} Taffy HTML fixtures, found {} under {}",
            taffy_files.len(),
            source_root.join(TAFFY_SOURCE_DIR).display()
        ));
    }
    let imported_taffy_files = taffy_files
        .into_iter()
        .filter(|rel| !is_excluded_corpus_dir(rel))
        .collect::<Vec<_>>();
    let imported_taffy_count = imported_taffy_files.len();

    for rel in &imported_taffy_files {
        let source = source_root.join(TAFFY_SOURCE_DIR).join(rel);
        let target = config.html_root.join(rel);
        let source_raw = fs::read(&source)
            .map_err(|error| format!("failed to read {}: {error}", source.display()))?;
        let target_raw = fs::read(&target)
            .map_err(|error| format!("failed to read {}: {error}", target.display()))?;
        if source_raw != target_raw {
            return Err(format!(
                "Taffy baseline drift: {} differs from {}",
                target.display(),
                source.display()
            ));
        }
    }

    let taffy_set = imported_taffy_files.into_iter().collect::<BTreeSet<_>>();
    for rel in collect_relative_files(&config.html_root)? {
        if rel.extension().and_then(|extension| extension.to_str()) != Some("html") {
            continue;
        }
        if taffy_set.contains(&rel) || allowed_extra_paths.contains(&rel) {
            continue;
        }
        return Err(format!(
            "unexpected HTML fixture outside Taffy baseline allow-list: {}",
            config.html_root.join(rel).display()
        ));
    }

    println!(
        "checked {imported_taffy_count} imported Taffy baseline fixtures ({expected_count} upstream) against {}",
        config.html_root.display()
    );
    Ok(())
}

fn check_corpus_junk_files(config: &Config) -> Result<(), String> {
    for rel in collect_relative_files(&config.root)? {
        if is_corpus_junk_file(&rel) {
            return Err(format!(
                "unexpected non-fixture file in parity corpus: {}",
                config.root.join(rel).display()
            ));
        }
    }
    Ok(())
}

fn check_gentest_helper_only_assets(config: &Config) -> Result<(), String> {
    let helper_root = config.root.join("scripts/gentest");
    if !helper_root.is_dir() {
        return Err(format!(
            "missing browser helper asset directory {}",
            helper_root.display()
        ));
    }
    let expected = BTreeSet::from([
        PathBuf::from("test_base_style.css"),
        PathBuf::from("test_helper.js"),
    ]);
    let actual = collect_relative_files(&helper_root)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!(
            "scripts/gentest must contain only test_helper.js and test_base_style.css; found {:?}",
            actual
        ));
    }
    Ok(())
}

fn is_corpus_junk_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, ".DS_Store" | "Thumbs.db" | "desktop.ini"))
}

fn import_taffy(config: &Config) -> Result<(), String> {
    validate_taffy_manifest(config)?;
    let source_root = taffy_source_root();
    ensure_taffy_source(&source_root)?;
    validate_taffy_source_revision(&source_root)?;

    let plan = prepare_taffy_import(&source_root)?;
    let expected_files = plan
        .fixtures
        .iter()
        .map(|(rel, _)| rel.clone())
        .collect::<BTreeSet<_>>();
    clear_taffy_import_outputs(config, &expected_files)?;
    for (rel, raw) in plan.fixtures {
        let target = config.html_root.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        fs::write(&target, raw).map_err(|error| {
            format!(
                "failed to write Taffy fixture {}: {error}",
                target.display()
            )
        })?;
    }

    check_taffy_corpus(config)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreparedTaffyImport {
    fixtures: Vec<(PathBuf, Vec<u8>)>,
}

fn prepare_taffy_import(source_root: &Path) -> Result<PreparedTaffyImport, String> {
    prepare_taffy_import_with_expected_count(source_root, TAFFY_EXPECTED_COUNT)
}

fn prepare_taffy_import_with_expected_count(
    source_root: &Path,
    expected_count: usize,
) -> Result<PreparedTaffyImport, String> {
    let taffy_files = collect_relative_html(&source_root.join(TAFFY_SOURCE_DIR))?;
    if taffy_files.len() != expected_count {
        return Err(format!(
            "expected {expected_count} Taffy HTML fixtures, found {} under {}",
            taffy_files.len(),
            source_root.join(TAFFY_SOURCE_DIR).display()
        ));
    }

    let mut fixtures = Vec::new();
    for rel in taffy_files {
        if is_excluded_corpus_dir(&rel) {
            continue;
        }
        let source = source_root.join(TAFFY_SOURCE_DIR).join(&rel);
        let raw = fs::read(&source)
            .map_err(|error| format!("failed to read {}: {error}", source.display()))?;
        fixtures.push((rel, raw));
    }
    Ok(PreparedTaffyImport { fixtures })
}

fn clear_taffy_import_outputs(
    config: &Config,
    expected_files: &BTreeSet<PathBuf>,
) -> Result<(), String> {
    let allowed_extra_paths = manifest_surgeist_constrained_paths(config)?;
    fs::create_dir_all(&config.html_root)
        .map_err(|error| format!("failed to create {}: {error}", config.html_root.display()))?;
    for rel in collect_relative_html(&config.html_root)? {
        if allowed_extra_paths.contains(&rel) || expected_files.contains(&rel) {
            continue;
        }
        let path = config.html_root.join(&rel);
        fs::remove_file(&path).map_err(|error| {
            format!(
                "failed to remove stale Taffy fixture {}: {error}",
                path.display()
            )
        })?;
    }
    Ok(())
}

fn manifest_surgeist_constrained_paths(config: &Config) -> Result<BTreeSet<PathBuf>, String> {
    let manifest = read_corpus_manifest(config)?;
    let mut paths = BTreeSet::new();
    for case in manifest.cases {
        if case.source_root != "surgeist" {
            continue;
        }
        validate_surgeist_constrained_case(&case)?;
        let path = validate_relative_path("Surgeist constrained case source", &case.source)?;
        paths.insert(path);
    }
    Ok(paths)
}

fn validate_taffy_manifest(config: &Config) -> Result<(), String> {
    let manifest_path = config.root.join("corpus.toml");
    let manifest = read_corpus_manifest(config)?;
    let taffy = &manifest.imports.taffy;
    if taffy.repo != TAFFY_REPO {
        return Err(format!(
            "{} [imports.taffy].repo is `{}`, expected `{TAFFY_REPO}`",
            manifest_path.display(),
            taffy.repo
        ));
    }
    if taffy.commit != TAFFY_COMMIT {
        return Err(format!(
            "{} [imports.taffy].commit is `{}`, expected `{TAFFY_COMMIT}`",
            manifest_path.display(),
            taffy.commit
        ));
    }
    if taffy.source_dir != TAFFY_SOURCE_DIR {
        return Err(format!(
            "{} [imports.taffy].source_dir is `{}`, expected `{TAFFY_SOURCE_DIR}`",
            manifest_path.display(),
            taffy.source_dir
        ));
    }
    if taffy.destination != "html" {
        return Err(format!(
            "{} [imports.taffy].destination is `{}`, expected `html`",
            manifest_path.display(),
            taffy.destination
        ));
    }
    if taffy.expected_count != TAFFY_EXPECTED_COUNT {
        return Err(format!(
            "{} [imports.taffy].expected_count is {}, expected {TAFFY_EXPECTED_COUNT}",
            manifest_path.display(),
            taffy.expected_count
        ));
    }
    let excluded = taffy
        .excluded_destination_dirs
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if excluded != BTreeSet::from(["grid-lanes", "subgrid"]) {
        return Err(format!(
            "{} [imports.taffy].excluded_destination_dirs is {:?}, expected [\"subgrid\", \"grid-lanes\"]",
            manifest_path.display(),
            taffy.excluded_destination_dirs
        ));
    }
    if !taffy.allowed_extra_paths.is_empty() {
        return Err(
            "[imports.taffy].allowed_extra_paths is no longer supported; declare local constrained fixtures as [[cases]]"
                .to_string(),
        );
    }
    validate_root_cases(&manifest.cases)?;
    Ok(())
}

fn validate_root_cases(cases: &[CorpusCase]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for case in cases {
        match case.source_root.as_str() {
            "taffy" | "surgeist" => {}
            other => {
                return Err(format!(
                    "root case {} uses unsupported source_root `{other}`",
                    case.id
                ));
            }
        }
        if !ids.insert(case.id.clone()) {
            return Err(format!("duplicate root case id `{}`", case.id));
        }
        let source = validate_relative_path("root case source", &case.source)?;
        let source_key = source.to_string_lossy().replace('\\', "/");
        if !sources.insert(source_key.clone()) {
            return Err(format!("duplicate root case source `{source_key}`"));
        }
        if case.source_root == "surgeist" {
            validate_surgeist_constrained_case(case)?;
        }
    }
    Ok(())
}

fn validate_surgeist_constrained_case(case: &CorpusCase) -> Result<(), String> {
    if case.id.trim().is_empty() {
        return Err("Surgeist constrained case id must not be empty".to_string());
    }
    if case.generator != "constrained-html" {
        return Err(format!(
            "Surgeist constrained case {} uses generator `{}`, expected `constrained-html`",
            case.id, case.generator
        ));
    }
    match case.status.as_str() {
        "active" | "expected-fail" | "unsupported" | "quarantined" => {}
        other => {
            return Err(format!(
                "Surgeist constrained case {} uses unsupported status `{other}`",
                case.id
            ));
        }
    }
    let source = validate_relative_path("Surgeist constrained case source", &case.source)?;
    if source.extension().and_then(|extension| extension.to_str()) != Some("html") {
        return Err(format!(
            "Surgeist constrained case {} source {} must be an HTML fixture",
            case.id,
            source.display()
        ));
    }
    Ok(())
}

fn validate_surgeist_constrained_case_files(config: &Config) -> Result<(), String> {
    let manifest = read_corpus_manifest(config)?;
    for case in manifest
        .cases
        .iter()
        .filter(|case| case.source_root == "surgeist")
    {
        validate_surgeist_constrained_case(case)?;
        let source = validate_relative_path("Surgeist constrained case source", &case.source)?;
        let path = config.html_root.join(&source);
        if !path.is_file() {
            return Err(format!(
                "missing Surgeist constrained case {} at {}",
                case.id,
                path.display()
            ));
        }
    }
    Ok(())
}

fn read_corpus_manifest(config: &Config) -> Result<CorpusManifest, String> {
    let manifest = config.root.join("corpus.toml");
    let raw = fs::read_to_string(&manifest)
        .map_err(|error| format!("failed to read {}: {error}", manifest.display()))?;
    toml::from_str::<CorpusManifest>(&raw)
        .map_err(|error| format!("failed to parse {}: {error}", manifest.display()))
}

fn taffy_source_root() -> PathBuf {
    PathBuf::from(SOURCE_CACHE_DIR)
        .join("taffy")
        .join(TAFFY_COMMIT)
}

fn validate_relative_path(kind: &str, raw: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        return Err(format!(
            "{kind} `{raw}` must be a relative path under its corpus root"
        ));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(segment) => normalized.push(segment),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                return Err(format!(
                    "{kind} `{raw}` must be a relative path under its corpus root"
                ));
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(format!(
                    "{kind} `{raw}` must be a relative path under its corpus root"
                ));
            }
        }
    }
    Ok(normalized)
}

fn validate_taffy_source_revision(source_root: &Path) -> Result<(), String> {
    validate_git_source_revision("Taffy", source_root, TAFFY_COMMIT)
}

fn validate_git_source_revision(
    label: &str,
    source_root: &Path,
    expected_commit: &str,
) -> Result<(), String> {
    if !source_root.join(".git").is_dir() {
        return Err(format!(
            "{label} source {} must be a git checkout at {expected_commit}",
            source_root.display(),
        ));
    }
    let output = Command::new("git")
        .current_dir(source_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| {
            format!(
                "failed to inspect {label} source {}: {error}",
                source_root.display()
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect {label} source {}:\nstdout:\n{}\nstderr:\n{}",
            source_root.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let actual = String::from_utf8_lossy(&output.stdout);
    if !actual.trim().starts_with(expected_commit) {
        return Err(format!(
            "{label} source {} is at {}, expected {expected_commit}",
            source_root.display(),
            actual.trim()
        ));
    }
    let output = Command::new("git")
        .current_dir(source_root)
        .args(["status", "--short"])
        .output()
        .map_err(|error| {
            format!(
                "failed to inspect {label} source {}: {error}",
                source_root.display()
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect {label} source {}:\nstdout:\n{}\nstderr:\n{}",
            source_root.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !output.stdout.is_empty() {
        return Err(format!(
            "{label} source {} has uncommitted changes:\n{}",
            source_root.display(),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}

fn ensure_taffy_source(source_root: &Path) -> Result<(), String> {
    ensure_git_source(source_root, TAFFY_REPO, TAFFY_COMMIT)
}

fn ensure_git_source(source_root: &Path, repo: &str, commit: &str) -> Result<(), String> {
    if source_root.join(".git").is_dir() {
        ensure_git_origin(source_root, repo)?;
        run_git(source_root, ["fetch", "--depth=1", "origin", commit])?;
        run_git(source_root, ["checkout", "--detach", commit])?;
        return Ok(());
    }

    if let Some(parent) = source_root.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    fs::create_dir_all(source_root)
        .map_err(|error| format!("failed to create {}: {error}", source_root.display()))?;
    run_git(source_root, ["init"])?;
    ensure_git_origin(source_root, repo)?;
    run_git(source_root, ["fetch", "--depth=1", "origin", commit])?;
    run_git(source_root, ["checkout", "--detach", "FETCH_HEAD"])?;
    Ok(())
}

fn ensure_git_origin(source_root: &Path, repo: &str) -> Result<(), String> {
    let output = Command::new("git")
        .current_dir(source_root)
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(|error| {
            format!(
                "failed to inspect git origin in {}: {error}",
                source_root.display()
            )
        })?;
    if !output.status.success() {
        run_git(source_root, ["remote", "add", "origin", repo])?;
        return Ok(());
    }

    let actual = String::from_utf8_lossy(&output.stdout);
    if actual.trim() != repo {
        run_git(source_root, ["remote", "set-url", "origin", repo])?;
    }
    Ok(())
}

fn run_git<const N: usize>(dir: &Path, args: [&str; N]) -> Result<(), String> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run git in {}: {error}", dir.display()))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "git failed in {}:\nstdout:\n{}\nstderr:\n{}",
        dir.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn collect_relative_html(root: &Path) -> Result<Vec<PathBuf>, String> {
    let files = collect_relative_files(root)?;
    Ok(files
        .into_iter()
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("html"))
        .collect())
}

fn collect_relative_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_relative_files_into(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_relative_files_into(
    root: &Path,
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|error| format!("failed to read {}: {error}", dir.display()))?
    {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_relative_files_into(root, &path, files)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .map_err(|error| {
                    format!(
                        "failed to make {} relative to {}: {error}",
                        path.display(),
                        root.display()
                    )
                })?
                .to_path_buf();
            files.push(rel);
        }
    }
    Ok(())
}

fn is_excluded_corpus_dir(path: &Path) -> bool {
    matches!(
        path.components()
            .next()
            .and_then(|component| component.as_os_str().to_str()),
        Some("subgrid" | "grid-lanes")
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum GenerationJob {
    ConstrainedHtml(PathBuf),
}

fn generation_jobs(constrained_fixtures: Vec<PathBuf>) -> Vec<GenerationJob> {
    let mut jobs = constrained_fixtures
        .into_iter()
        .map(GenerationJob::ConstrainedHtml)
        .collect::<Vec<_>>();
    jobs.sort_by_key(generation_job_path);
    jobs
}

fn generation_job_path(job: &GenerationJob) -> String {
    match job {
        GenerationJob::ConstrainedHtml(path) => path.to_string_lossy().into_owned(),
    }
}

async fn generate_batch(
    config: &Config,
    browser_path: &Path,
    jobs: &[GenerationJob],
    batch_index: usize,
    report: &mut GenerationReport,
) -> Result<(), String> {
    let profile = config
        .browser_cache
        .parent()
        .unwrap_or(&config.browser_cache)
        .join("surgeist-browser-profile")
        .join(format!("{}-{batch_index}", process::id()));
    fs::create_dir_all(&profile)
        .map_err(|error| format!("failed to create {}: {error}", profile.display()))?;

    let browser_config = BrowserConfig::builder()
        .chrome_executable(browser_path)
        .with_head()
        .disable_default_args()
        .disable_cache()
        .user_data_dir(&profile)
        .args(browser_args())
        .build()
        .map_err(|error| format!("failed to configure browser: {error}"))?;

    let (mut browser, mut handler) = Browser::launch(browser_config)
        .await
        .map_err(|error| format!("failed to launch browser: {error}"))?;
    let handler_task = tokio::spawn(async move { while handler.next().await.is_some() {} });

    let result = async {
        for (job_index, job) in jobs.iter().enumerate() {
            let page = match browser.new_page("about:blank").await {
                Ok(page) => page,
                Err(error) => {
                    let error = format!("failed to create page: {error}");
                    record_failed_generation_job(config, job, report, error.clone());
                    eprintln!(
                        "reporting failed browser parity generation for {}: {error}",
                        generation_job_path(job)
                    );
                    continue;
                }
            };
            let result = match generate_job(config, browser_path, &page, job, report).await {
                Err(error) if is_retryable_generation_error(&error) => {
                    eprintln!(
                        "retrying browser parity generation for {} after navigation timeout",
                        generation_job_path(job)
                    );
                    retry_job_in_fresh_browser(
                        config,
                        browser_path,
                        job,
                        batch_index,
                        job_index,
                        report,
                    )
                    .await
                    .map_err(|retry_error| format!("{error}; retry failed: {retry_error}"))
                }
                result => result,
            };
            page.close().await.ok();
            if let Err(error) = result {
                record_failed_generation_job(config, job, report, error.clone());
                eprintln!(
                    "reporting failed browser parity generation for {}: {error}",
                    generation_job_path(job)
                );
            }
        }
        Ok::<(), String>(())
    }
    .await;

    browser.close().await.ok();
    handler_task.await.ok();
    fs::remove_dir_all(profile).ok();

    result
}

async fn generate_job(
    config: &Config,
    browser_path: &Path,
    page: &chromiumoxide::Page,
    job: &GenerationJob,
    report: &mut GenerationReport,
) -> Result<(), String> {
    match job {
        GenerationJob::ConstrainedHtml(fixture) => match describe_fixture(page, fixture).await {
            Ok(desc) => write_fixture_goldens(config, browser_path, fixture, &desc, report),
            Err(error) => Err(error),
        },
    }
}

async fn retry_job_in_fresh_browser(
    config: &Config,
    browser_path: &Path,
    job: &GenerationJob,
    batch_index: usize,
    job_index: usize,
    report: &mut GenerationReport,
) -> Result<(), String> {
    let profile = config
        .browser_cache
        .parent()
        .unwrap_or(&config.browser_cache)
        .join("surgeist-browser-profile")
        .join(format!("{}-{batch_index}-{job_index}-retry", process::id()));
    fs::create_dir_all(&profile)
        .map_err(|error| format!("failed to create {}: {error}", profile.display()))?;

    let browser_config = BrowserConfig::builder()
        .chrome_executable(browser_path)
        .with_head()
        .disable_default_args()
        .disable_cache()
        .user_data_dir(&profile)
        .args(browser_args())
        .build()
        .map_err(|error| format!("failed to configure retry browser: {error}"))?;

    let (mut browser, mut handler) = Browser::launch(browser_config)
        .await
        .map_err(|error| format!("failed to launch retry browser: {error}"))?;
    let handler_task = tokio::spawn(async move { while handler.next().await.is_some() {} });

    let result = async {
        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|error| format!("failed to create retry page: {error}"))?;
        let result = generate_job(config, browser_path, &page, job, report).await;
        page.close().await.ok();
        result
    }
    .await;

    browser.close().await.ok();
    handler_task.await.ok();
    fs::remove_dir_all(profile).ok();

    result
}

fn is_retryable_generation_error(error: &str) -> bool {
    (error.contains("failed to open")
        || error.contains("failed to load")
        || error.contains("failed to reset"))
        && (error.contains("Request timed out")
            || error.contains("timed out waiting for DOM readiness"))
}

async fn resolve_browser_path(config: &Config) -> Result<PathBuf, String> {
    if let Some(path) = &config.browser_path {
        return Ok(path.clone());
    }

    fs::create_dir_all(&config.browser_cache).map_err(|error| {
        format!(
            "failed to create browser cache {}: {error}",
            config.browser_cache.display()
        )
    })?;

    let mut fetcher_options = BrowserFetcherOptions::builder()
        .with_kind(BrowserKind::Chrome)
        .with_path(&config.browser_cache);
    if let Some(version) = &config.browser_version {
        let version = version
            .parse::<BrowserVersion>()
            .map_err(|error| format!("invalid {BROWSER_VERSION_ENV}={version:?}: {error}"))?;
        fetcher_options = fetcher_options.with_version(version);
    }
    let fetcher = BrowserFetcher::new(
        fetcher_options
            .build()
            .map_err(|error| format!("failed to configure browser fetcher: {error}"))?,
    );
    let browser = fetcher
        .fetch()
        .await
        .map_err(|error| format!("failed to fetch local Chromium binary: {error}"))?;
    Ok(browser.executable_path)
}

fn browser_args() -> [&'static str; 28] {
    [
        "headless=new",
        "mute-audio",
        "disable-background-networking",
        "disable-background-timer-throttling",
        "disable-backgrounding-occluded-windows",
        "disable-breakpad",
        "disable-client-side-phishing-detection",
        "disable-component-extensions-with-background-pages",
        "disable-component-update",
        "disable-default-apps",
        "disable-dev-shm-usage",
        "disable-domain-reliability",
        "disable-features=TranslateUI,MediaRouter,OptimizationHints,AutofillServerCommunication",
        "disable-hang-monitor",
        "disable-ipc-flooding-protection",
        "disable-popup-blocking",
        "disable-prompt-on-repost",
        "disable-renderer-backgrounding",
        "disable-sync",
        "enable-automation",
        "enable-blink-features=IdleDetection,CSSGridLanesLayout",
        "enable-features=NetworkService,NetworkServiceInProcess",
        "force-color-profile=srgb",
        "lang=en_US",
        "metrics-recording-only",
        "no-default-browser-check",
        "no-first-run",
        "use-mock-keychain",
    ]
}

async fn describe_fixture(page: &chromiumoxide::Page, fixture: &Path) -> Result<Value, String> {
    open_fixture_page(page, fixture).await?;
    ensure_test_helper(page, fixture).await?;
    let json: String = page
        .evaluate_function("() => getTestData()")
        .await
        .map_err(|error| format!("failed to measure {}: {error}", fixture.display()))?
        .into_value()
        .map_err(|error| format!("failed to read measurement JSON: {error}"))?;
    serde_json::from_str(&json).map_err(|error| format!("invalid measurement JSON: {error}"))
}

async fn open_fixture_page(page: &chromiumoxide::Page, fixture: &Path) -> Result<(), String> {
    let raw = fs::read_to_string(fixture)
        .map_err(|error| format!("failed to read fixture {}: {error}", fixture.display()))?;
    let base_url = fixture_base_url(fixture)?;
    let html = browser_fixture_document(&raw, base_url.as_str())?;
    let script = browser_document_write_script(&html);
    page.evaluate_expression(script).await.map_err(|error| {
        format!(
            "failed to load {} into browser page: {error}",
            fixture.display()
        )
    })?;
    wait_for_fixture_dom(page, fixture).await
}

fn browser_fixture_document(raw: &str, base_url: &str) -> Result<String, String> {
    let mut head_injection = format!("<base href=\"{}\">", escape_attr(base_url));
    if raw.contains("test_base_style.css") {
        head_injection.push_str("<style>");
        head_injection.push_str(TEST_BASE_STYLE_SOURCE);
        head_injection.push_str("</style>");
    }

    let lower = raw.to_ascii_lowercase();
    if let Some(index) = lower.find("<head>") {
        let insert_at = index + "<head>".len();
        let mut html = String::with_capacity(raw.len() + head_injection.len());
        html.push_str(&raw[..insert_at]);
        html.push_str(&head_injection);
        html.push_str(&raw[insert_at..]);
        Ok(html)
    } else if let Some(index) = lower.find("<html") {
        let html_tag_end = raw[index..]
            .find('>')
            .ok_or_else(|| "fixture html tag is missing closing `>`".to_string())?
            + index
            + 1;
        Ok(format!(
            "{}<head>{head_injection}</head>{}",
            &raw[..html_tag_end],
            &raw[html_tag_end..]
        ))
    } else if lower.starts_with("<!doctype") {
        let doctype_end = raw
            .find('>')
            .ok_or_else(|| "fixture doctype is missing closing `>`".to_string())?
            + 1;
        Ok(format!(
            "{}<html><head>{head_injection}</head><body>{}</body></html>",
            &raw[..doctype_end],
            &raw[doctype_end..]
        ))
    } else {
        Ok(format!(
            "<html><head>{head_injection}</head><body>{raw}</body></html>"
        ))
    }
}

fn browser_document_write_script(html: &str) -> String {
    let html = serde_json::to_string(html).expect("serializing HTML should not fail");
    format!(
        "(() => {{ document.open(); document.write({html}); document.close(); return true; }})()"
    )
}

async fn wait_for_fixture_dom(page: &chromiumoxide::Page, fixture: &Path) -> Result<(), String> {
    let deadline = Instant::now() + BROWSER_NAVIGATION_TIMEOUT;
    let readiness_script = fixture_dom_readiness_script();
    let mut last_error = None;
    while Instant::now() < deadline {
        match page.evaluate_expression(readiness_script).await {
            Ok(value) => {
                let state = value.into_value::<String>().map_err(|error| {
                    format!(
                        "failed to read DOM readiness for {}: {error}",
                        fixture.display()
                    )
                })?;
                let ready = fixture_dom_is_ready(&state);
                if ready {
                    return Ok(());
                }
                last_error = Some(state);
            }
            Err(error) => {
                last_error = Some(error.to_string());
            }
        }
        tokio::time::sleep(BROWSER_NAVIGATION_POLL_INTERVAL).await;
    }

    let detail = last_error
        .map(|error| format!("; last readiness error: {error}"))
        .unwrap_or_default();
    Err(format!(
        "failed to open {}: timed out waiting for DOM readiness{detail}",
        fixture.display()
    ))
}

fn fixture_dom_readiness_script() -> &'static str {
    "(() => JSON.stringify({ href: window.location.href, readyState: document.readyState, hasBody: !!document.body, ready: document.readyState !== 'loading' && !!document.body }))()"
}

fn fixture_dom_is_ready(state: &str) -> bool {
    serde_json::from_str::<Value>(state)
        .ok()
        .and_then(|value| value["ready"].as_bool())
        .unwrap_or(false)
}

async fn ensure_test_helper(page: &chromiumoxide::Page, fixture: &Path) -> Result<(), String> {
    let has_helper: bool = page
        .evaluate_expression("typeof getTestData === 'function'")
        .await
        .map_err(|error| {
            format!(
                "failed to inspect measurement helper for {}: {error}",
                fixture.display()
            )
        })?
        .into_value()
        .map_err(|error| {
            format!(
                "failed to read measurement helper state for {}: {error}",
                fixture.display()
            )
        })?;
    if !has_helper {
        page.evaluate_expression(TEST_HELPER_SOURCE)
            .await
            .map_err(|error| {
                format!(
                    "failed to inject measurement helper for {}: {error}",
                    fixture.display()
                )
            })?;
    }
    page.evaluate_expression(GRID_TEMPLATE_AREA_CAPTURE_SCRIPT)
        .await
        .map_err(|error| {
            format!(
                "failed to inject grid template area capture for {}: {error}",
                fixture.display()
            )
        })?;
    Ok(())
}

fn fixture_url(fixture: &Path) -> Result<url::Url, String> {
    let path = if fixture.is_absolute() {
        fixture.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?
            .join(fixture)
    };
    url::Url::from_file_path(&path)
        .map_err(|()| format!("failed to create file URL for {}", fixture.display()))
}

fn fixture_base_url(fixture: &Path) -> Result<url::Url, String> {
    let parent = fixture
        .parent()
        .ok_or_else(|| format!("fixture {} has no parent directory", fixture.display()))?;
    let mut url = fixture_url(parent)?;
    if !url.as_str().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }
    Ok(url)
}

fn collect_html(root: &Path, filter: Option<&str>) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_html_into(root, &mut files)?;
    if let Some(filter) = filter {
        files.retain(|file| fixture_matches_filter(root, file, filter));
    }
    files.sort();
    Ok(files)
}

fn collect_constrained_fixtures_for_generation(
    config: &Config,
    report: &mut GenerationReport,
) -> Result<Vec<PathBuf>, String> {
    let mut fixtures = collect_html(&config.html_root, config.filter.as_deref())?;
    let mut excluded = BTreeSet::new();
    let manifest = read_corpus_manifest(config)?;
    for case in manifest
        .cases
        .iter()
        .filter(|case| case.source_root == "surgeist")
    {
        validate_surgeist_constrained_case(case)?;
        let source = validate_relative_path("Surgeist constrained case source", &case.source)?;
        let fixture = config.html_root.join(&source);
        if config
            .filter
            .as_deref()
            .is_some_and(|filter| !fixture_matches_filter(&config.html_root, &fixture, filter))
        {
            continue;
        }
        let report_source = format!("html/{}", source.to_string_lossy().replace('\\', "/"));
        match case.status.as_str() {
            "active" => {}
            "expected-fail" => {
                report.record_expected_fail(
                    case.id.clone(),
                    report_source,
                    case.reason
                        .clone()
                        .unwrap_or_else(|| "manifest marks case expected-fail".to_string()),
                );
            }
            "unsupported" => {
                report.record_unsupported(
                    case.id.clone(),
                    report_source,
                    "manifest".to_string(),
                    case.reason
                        .clone()
                        .unwrap_or_else(|| "manifest marks case unsupported".to_string()),
                );
                prune_constrained_status_case_outputs(config, &fixture)?;
                excluded.insert(fixture);
            }
            "quarantined" => {
                report.record_quarantined(
                    case.id.clone(),
                    report_source,
                    case.reason
                        .clone()
                        .unwrap_or_else(|| "manifest marks case quarantined".to_string()),
                );
                prune_constrained_status_case_outputs(config, &fixture)?;
                excluded.insert(fixture);
            }
            other => {
                return Err(format!(
                    "Surgeist constrained case {} uses unsupported status `{other}`",
                    case.id
                ));
            }
        }
    }
    fixtures.retain(|fixture| !excluded.contains(fixture));
    Ok(fixtures)
}

fn prune_constrained_status_case_outputs(config: &Config, fixture: &Path) -> Result<(), String> {
    for path in output_paths_for_fixture(&config.html_root, &config.xml_root, fixture)? {
        if path.exists() {
            fs::remove_file(&path).map_err(|error| {
                format!("failed to remove stale XML {}: {error}", path.display())
            })?;
        }
    }
    Ok(())
}

fn fixture_matches_filter(root: &Path, fixture: &Path, filter: &str) -> bool {
    let filter = filter.trim_matches('/');
    if filter.is_empty() {
        return true;
    }
    let Ok(rel) = fixture.strip_prefix(root) else {
        return false;
    };
    let rel_no_ext = rel.with_extension("");
    rel.starts_with(filter)
        || rel_no_ext.starts_with(filter)
        || rel_no_ext.to_string_lossy().starts_with(filter)
}

fn collect_html_into(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_html_into(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("html") {
            files.push(path);
        }
    }
    Ok(())
}

fn write_fixture_goldens(
    config: &Config,
    browser_path: &Path,
    fixture: &Path,
    desc: &Value,
    report: &mut GenerationReport,
) -> Result<(), String> {
    let rel = fixture.strip_prefix(&config.html_root).map_err(|error| {
        format!(
            "failed to make fixture path relative to {}: {error}",
            config.html_root.display()
        )
    })?;
    let group = rel.parent().unwrap_or_else(|| Path::new(""));
    let stem = fixture
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| format!("fixture has no UTF-8 stem: {}", fixture.display()))?;
    let source = rel.to_string_lossy().replace('\\', "/");
    let source_sha256 = sha256_file(fixture)?;
    let helper_sha256 = sha256_bytes(TEST_HELPER_SOURCE.as_bytes());
    let browser = browser_provenance(config, browser_path);
    let output_dir = config.xml_root.join(group);
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;

    let mut planned = Vec::new();
    for (variant, key) in fixture_cases() {
        let data = desc
            .get(key)
            .ok_or_else(|| format!("measurement JSON missing {key}"))?;
        let name = format!("{stem}__{variant}");
        let output_file = output_dir.join(format!("{name}.xml"));
        let report_source = format!("html/{source}");
        if let Some(reason) = unsupported_fixture_reason(data) {
            planned.push(PlannedGoldenOutput::Unsupported {
                name,
                source: report_source,
                output_file,
                variant: variant.to_string(),
                reason: reason.to_string(),
            });
            continue;
        }
        let provenance = GeneratedProvenance {
            source: report_source.clone(),
            source_sha256: source_sha256.clone(),
            linked_resources: Vec::new(),
            linked_resources_recorded: false,
            helper_sha256: helper_sha256.clone(),
            browser: browser.clone(),
        };
        planned.push(PlannedGoldenOutput::Generated {
            name: name.clone(),
            source: report_source,
            output: root_relative_source(&config.root, &output_file)?,
            output_file,
            variant: variant.to_string(),
            xml: generate_xml_with_provenance(&name, data, Some(&provenance)),
        });
    }

    commit_planned_golden_outputs(
        output_paths_for_fixture(&config.html_root, &config.xml_root, fixture)?,
        &planned,
    )?;
    for output in planned {
        match output {
            PlannedGoldenOutput::Unsupported {
                name,
                source,
                variant,
                reason,
                ..
            } => {
                report.record_unsupported(name.clone(), source, variant, reason.clone());
                eprintln!("reporting unsupported browser parity fixture {name}: {reason}");
            }
            PlannedGoldenOutput::Generated {
                name,
                source,
                output,
                variant,
                ..
            } => {
                report.record_generated(name, source, output, variant);
            }
        }
    }
    Ok(())
}

enum PlannedGoldenOutput {
    Generated {
        name: String,
        source: String,
        output: String,
        output_file: PathBuf,
        variant: String,
        xml: String,
    },
    Unsupported {
        name: String,
        source: String,
        output_file: PathBuf,
        variant: String,
        reason: String,
    },
}

impl PlannedGoldenOutput {
    fn output_file(&self) -> &Path {
        match self {
            PlannedGoldenOutput::Generated { output_file, .. }
            | PlannedGoldenOutput::Unsupported { output_file, .. } => output_file,
        }
    }
}

fn commit_planned_golden_outputs(
    stale_paths: Vec<PathBuf>,
    planned: &[PlannedGoldenOutput],
) -> Result<(), String> {
    commit_planned_golden_outputs_with_hook(stale_paths, planned, || Ok(()))
}

fn commit_planned_golden_outputs_with_hook<F>(
    stale_paths: Vec<PathBuf>,
    planned: &[PlannedGoldenOutput],
    before_install: F,
) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    let mut pending = write_planned_temp_files(planned)?;
    let mut replace_paths = stale_paths;
    replace_paths.extend(
        planned
            .iter()
            .map(|output| output.output_file().to_path_buf()),
    );
    replace_paths.sort();
    replace_paths.dedup();

    let mut backups = Vec::new();
    if let Err(error) = backup_existing_outputs(&replace_paths, &mut backups) {
        remove_temp_files(&pending);
        restore_backups(&mut backups);
        return Err(error);
    }

    if let Err(error) = before_install() {
        remove_temp_files(&pending);
        restore_backups(&mut backups);
        return Err(error);
    }

    let mut installed = Vec::new();
    while let Some(write) = pending.pop() {
        if let Err(error) = fs::rename(&write.temp_path, &write.output_file) {
            cleanup_partial_install(&write, &pending, &installed, &mut backups);
            return Err(format!(
                "failed to install generated XML {}: {error}",
                write.output_file.display()
            ));
        }
        installed.push(write.output_file);
    }

    remove_backups(backups)
}

struct PendingGoldenWrite {
    output_file: PathBuf,
    temp_path: PathBuf,
}

fn write_planned_temp_files(
    planned: &[PlannedGoldenOutput],
) -> Result<Vec<PendingGoldenWrite>, String> {
    let mut pending = Vec::new();
    for (index, output) in planned.iter().enumerate() {
        let PlannedGoldenOutput::Generated {
            output_file, xml, ..
        } = output
        else {
            continue;
        };
        let temp_path = sibling_temp_path(output_file, "tmp", index);
        if let Err(error) = write_temp_xml(&temp_path, xml) {
            remove_temp_files(&pending);
            return Err(error);
        }
        pending.push(PendingGoldenWrite {
            output_file: output_file.clone(),
            temp_path,
        });
    }
    Ok(pending)
}

fn write_temp_xml(temp_path: &Path, xml: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp_path)
        .map_err(|error| {
            format!(
                "failed to create temporary XML {}: {error}",
                temp_path.display()
            )
        })?;
    if let Err(error) = file.write_all(xml.as_bytes()) {
        fs::remove_file(temp_path).ok();
        return Err(format!(
            "failed to write temporary XML {}: {error}",
            temp_path.display()
        ));
    }
    Ok(())
}

fn backup_existing_outputs(
    paths: &[PathBuf],
    backups: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), String> {
    for (index, path) in paths.iter().enumerate() {
        if !path.exists() {
            continue;
        }
        let backup = sibling_temp_path(path, "bak", index);
        fs::rename(path, &backup).map_err(|error| {
            format!(
                "failed to back up existing XML {} to {}: {error}",
                path.display(),
                backup.display()
            )
        })?;
        backups.push((path.clone(), backup));
    }
    Ok(())
}

fn sibling_temp_path(path: &Path, suffix: &str, index: usize) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("generated.xml");
    parent.join(format!(
        ".{file_name}.{}.{}.{}",
        process::id(),
        index,
        suffix
    ))
}

fn cleanup_partial_install(
    failed_write: &PendingGoldenWrite,
    pending: &[PendingGoldenWrite],
    installed: &[PathBuf],
    backups: &mut Vec<(PathBuf, PathBuf)>,
) {
    fs::remove_file(&failed_write.temp_path).ok();
    remove_temp_files(pending);
    for path in installed {
        fs::remove_file(path).ok();
    }
    restore_backups(backups);
}

fn remove_temp_files(pending: &[PendingGoldenWrite]) {
    for write in pending {
        fs::remove_file(&write.temp_path).ok();
    }
}

fn restore_backups(backups: &mut Vec<(PathBuf, PathBuf)>) {
    while let Some((original, backup)) = backups.pop() {
        if original.exists() {
            fs::remove_file(&original).ok();
        }
        fs::rename(backup, original).ok();
    }
}

fn remove_backups(backups: Vec<(PathBuf, PathBuf)>) -> Result<(), String> {
    for (_, backup) in backups {
        fs::remove_file(&backup)
            .map_err(|error| format!("failed to remove backup {}: {error}", backup.display()))?;
    }
    Ok(())
}

fn prune_stale_generated_xml_outputs(
    config: &Config,
    report: &GenerationReport,
) -> Result<(), String> {
    if !config.xml_root.is_dir() {
        return Ok(());
    }

    let generated_outputs = report
        .generated
        .iter()
        .map(|entry| entry.output.as_str())
        .collect::<BTreeSet<_>>();
    let mut xml_files = Vec::new();
    collect_generated_xml_files(&config.xml_root, &mut xml_files)?;

    for path in xml_files {
        let source = root_relative_source(&config.root, &path)?;
        if generated_outputs.contains(source.as_str()) {
            continue;
        }
        fs::remove_file(&path)
            .map_err(|error| format!("failed to remove stale XML {}: {error}", path.display()))?;
    }
    Ok(())
}

fn prune_stale_generated_xml_outputs_after_success(
    config: &Config,
    report: &GenerationReport,
) -> Result<(), String> {
    if config.filter.is_some() || report.summary.failed_to_generate > 0 {
        return Ok(());
    }
    prune_stale_generated_xml_outputs(config, report)
}

fn collect_generated_xml_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|error| format!("failed to read {}: {error}", dir.display()))?
    {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()) == Some("generation-reports") {
            continue;
        }
        if path.is_dir() {
            collect_generated_xml_files(&path, files)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("xml") {
            files.push(path);
        }
    }
    Ok(())
}

fn record_failed_generation_job(
    config: &Config,
    job: &GenerationJob,
    report: &mut GenerationReport,
    reason: String,
) {
    let (name, source) = generation_job_report_identity(config, job);
    report.record_failed_to_generate(name, source, reason);
}

fn generation_job_report_identity(config: &Config, job: &GenerationJob) -> (String, String) {
    match job {
        GenerationJob::ConstrainedHtml(fixture) => {
            let name = fixture
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("unknown")
                .to_string();
            let source = fixture
                .strip_prefix(&config.html_root)
                .map(|rel| format!("html/{}", rel.to_string_lossy().replace('\\', "/")))
                .unwrap_or_else(|_| fixture.to_string_lossy().replace('\\', "/"));
            (name, source)
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
struct GenerationReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<GenerationReportMetadata>,
    filter: Option<String>,
    summary: GenerationReportSummary,
    generated: Vec<GeneratedReportEntry>,
    unsupported: Vec<UnsupportedReportEntry>,
    expected_fail: Vec<StatusReportEntry>,
    quarantined: Vec<StatusReportEntry>,
    failed_to_generate: Vec<StatusReportEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct GenerationReportMetadata {
    schema_version: u32,
    generator: &'static str,
    browser_source: &'static str,
    browser_version: Option<String>,
    helper_sha256: String,
    corpus_manifest_sha256: String,
    taffy_commit: &'static str,
}

impl GenerationReport {
    fn has_entries(&self) -> bool {
        self.summary.generated > 0
            || self.summary.unsupported > 0
            || self.summary.expected_fail > 0
            || self.summary.quarantined > 0
            || self.summary.failed_to_generate > 0
    }

    fn record_generated(&mut self, name: String, source: String, output: String, variant: String) {
        self.summary.generated += 1;
        self.generated.push(GeneratedReportEntry {
            name,
            source,
            output,
            variant,
        });
    }

    fn record_unsupported(
        &mut self,
        name: String,
        source: String,
        variant: String,
        reason: String,
    ) {
        self.summary.unsupported += 1;
        self.unsupported.push(UnsupportedReportEntry {
            name,
            source,
            variant,
            reason,
        });
    }

    fn record_expected_fail(&mut self, name: String, source: String, reason: String) {
        self.summary.expected_fail += 1;
        self.expected_fail.push(StatusReportEntry {
            name,
            source,
            reason,
        });
    }

    fn record_quarantined(&mut self, name: String, source: String, reason: String) {
        self.summary.quarantined += 1;
        self.quarantined.push(StatusReportEntry {
            name,
            source,
            reason,
        });
    }

    fn record_failed_to_generate(&mut self, name: String, source: String, reason: String) {
        self.summary.failed_to_generate += 1;
        self.failed_to_generate.push(StatusReportEntry {
            name,
            source,
            reason,
        });
    }
}

#[derive(Clone, Debug, Default, Serialize)]
struct GenerationReportSummary {
    generated: usize,
    unsupported: usize,
    expected_fail: usize,
    quarantined: usize,
    failed_to_generate: usize,
}

#[derive(Clone, Debug, Serialize)]
struct GeneratedReportEntry {
    name: String,
    source: String,
    output: String,
    variant: String,
}

#[derive(Clone, Debug, Serialize)]
struct UnsupportedReportEntry {
    name: String,
    source: String,
    variant: String,
    reason: String,
}

#[derive(Clone, Debug, Serialize)]
struct StatusReportEntry {
    name: String,
    source: String,
    reason: String,
}

fn write_generation_report(config: &Config, report: &GenerationReport) -> Result<(), String> {
    write_generation_report_with_hook(config, report, || Ok(()))
}

fn write_generation_report_with_hook<F>(
    config: &Config,
    report: &GenerationReport,
    before_install: F,
) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    let path = generation_report_path(config);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let mut report = report.clone();
    report.metadata = Some(generation_report_metadata(config)?);
    let raw = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("failed to serialize generation report: {error}"))?;
    commit_generated_report_atomically(&path, &format!("{raw}\n"), before_install)
}

fn commit_generated_report_atomically<F>(
    path: &Path,
    content: &str,
    before_install: F,
) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    let temp_path = sibling_temp_path(path, "tmp", 0);
    write_temp_report(&temp_path, content)?;
    if let Err(error) = before_install() {
        fs::remove_file(&temp_path).ok();
        return Err(error);
    }

    replace_generated_report(&temp_path, path)
}

fn write_temp_report(temp_path: &Path, content: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp_path)
        .map_err(|error| {
            format!(
                "failed to create temporary generation report {}: {error}",
                temp_path.display()
            )
        })?;
    if let Err(error) = file.write_all(content.as_bytes()) {
        fs::remove_file(temp_path).ok();
        return Err(format!(
            "failed to write temporary generation report {}: {error}",
            temp_path.display()
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn replace_generated_report(temp_path: &Path, path: &Path) -> Result<(), String> {
    fs::rename(temp_path, path).map_err(|error| {
        fs::remove_file(temp_path).ok();
        format!(
            "failed to install generation report {} atomically: {error}",
            path.display()
        )
    })
}

#[cfg(not(unix))]
fn replace_generated_report(temp_path: &Path, path: &Path) -> Result<(), String> {
    let backup_path = sibling_temp_path(path, "bak", 0);
    let had_existing = path.exists();
    if had_existing {
        if let Err(error) = fs::rename(path, &backup_path) {
            fs::remove_file(temp_path).ok();
            return Err(format!(
                "failed to back up generation report {} to {}: {error}",
                path.display(),
                backup_path.display()
            ));
        }
    }

    if let Err(error) = fs::rename(temp_path, path) {
        fs::remove_file(temp_path).ok();
        if had_existing {
            fs::rename(&backup_path, path).ok();
        }
        return Err(format!(
            "failed to install generation report {}: {error}",
            path.display()
        ));
    }

    if had_existing {
        fs::remove_file(backup_path).map_err(|error| {
            format!(
                "failed to remove generation report backup for {}: {error}",
                path.display()
            )
        })?;
    }
    Ok(())
}

fn generation_report_metadata(config: &Config) -> Result<GenerationReportMetadata, String> {
    Ok(GenerationReportMetadata {
        schema_version: 1,
        generator: "surgeist-layout-generate",
        browser_source: generation_report_browser_source(config),
        browser_version: generation_report_browser_version(config),
        helper_sha256: sha256_bytes(TEST_HELPER_SOURCE.as_bytes()),
        corpus_manifest_sha256: sha256_file(&config.root.join("corpus.toml"))?,
        taffy_commit: TAFFY_COMMIT,
    })
}

fn generation_report_browser_source(config: &Config) -> &'static str {
    if config.browser_path.is_some() {
        "custom-path"
    } else {
        "chrome-for-testing"
    }
}

fn generation_report_browser_version(config: &Config) -> Option<String> {
    if config.browser_path.is_some() {
        None
    } else {
        config.browser_version.clone()
    }
}

fn validate_generation_report_freshness(config: &Config) -> Result<(), String> {
    let report_dir = config.xml_root.join("generation-reports");
    if !report_dir.is_dir() {
        return Err(format!(
            "missing generation report directory {}; regenerate browser parity XML",
            report_dir.display()
        ));
    }
    let expected = generation_report_metadata(config)?;
    let path = report_dir.join("all.json");
    if !path.is_file() {
        return Err(format!(
            "missing generation report {}; regenerate browser parity XML",
            path.display()
        ));
    }
    validate_generation_report_metadata(&path, None, &expected)?;
    Ok(())
}

fn validate_generation_report_metadata(
    path: &Path,
    expected_filter: Option<&str>,
    expected: &GenerationReportMetadata,
) -> Result<(), String> {
    let raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read generation report {}: {error}",
            path.display()
        )
    })?;
    let json = serde_json::from_str::<Value>(&raw).map_err(|error| {
        format!(
            "failed to parse generation report {}: {error}",
            path.display()
        )
    })?;
    match expected_filter {
        Some(filter) => {
            if json["filter"].as_str() != Some(filter) {
                return Err(format!(
                    "{} report filter is {:?}, expected {:?}",
                    path.display(),
                    json["filter"].as_str(),
                    filter
                ));
            }
        }
        None => {
            if !json["filter"].is_null() {
                return Err(format!(
                    "{} report filter is {:?}, expected full-corpus null",
                    path.display(),
                    json["filter"]
                ));
            }
        }
    }
    let metadata = json
        .get("metadata")
        .ok_or_else(|| format!("{} is missing generation report metadata", path.display()))?;
    let checks = [
        (
            "schema_version",
            metadata["schema_version"]
                .as_u64()
                .map(|value| value.to_string()),
            Some(expected.schema_version.to_string()),
        ),
        (
            "generator",
            metadata["generator"].as_str().map(str::to_string),
            Some(expected.generator.to_string()),
        ),
        (
            "browser_source",
            metadata["browser_source"].as_str().map(str::to_string),
            Some(expected.browser_source.to_string()),
        ),
        (
            "browser_version",
            metadata["browser_version"].as_str().map(str::to_string),
            expected.browser_version.clone(),
        ),
        (
            "helper_sha256",
            metadata["helper_sha256"].as_str().map(str::to_string),
            Some(expected.helper_sha256.clone()),
        ),
        (
            "corpus_manifest_sha256",
            metadata["corpus_manifest_sha256"]
                .as_str()
                .map(str::to_string),
            Some(expected.corpus_manifest_sha256.clone()),
        ),
        (
            "taffy_commit",
            metadata["taffy_commit"].as_str().map(str::to_string),
            Some(expected.taffy_commit.to_string()),
        ),
    ];
    for (key, actual, expected) in checks {
        if actual != expected {
            return Err(format!(
                "{} generation report metadata `{key}` is {:?}, expected {:?}; regenerate browser parity XML",
                path.display(),
                actual,
                expected
            ));
        }
    }
    validate_generation_report_body(path, &json)?;
    Ok(())
}

fn validate_generation_report_body(path: &Path, json: &Value) -> Result<(), String> {
    if json.get("skipped").is_some() || json["summary"].get("skipped").is_some() {
        return Err(format!(
            "{} generation report uses a generic skipped bucket; use explicit unsupported, expected_fail, quarantined, or failed_to_generate buckets",
            path.display()
        ));
    }
    for bucket in GENERATION_REPORT_BUCKETS {
        let entries = json[bucket].as_array().ok_or_else(|| {
            format!(
                "{} generation report `{bucket}` bucket must be an array",
                path.display()
            )
        })?;
        let summary = json["summary"][bucket].as_u64().ok_or_else(|| {
            format!(
                "{} generation report `{bucket}` summary must be a number",
                path.display()
            )
        })? as usize;
        if summary != entries.len() {
            return Err(format!(
                "{} generation report {bucket} summary is {summary} but bucket has {} entries; regenerate browser parity XML",
                path.display(),
                entries.len()
            ));
        }
    }
    Ok(())
}

fn validate_xml_provenance_freshness(config: &Config) -> Result<(), String> {
    if !config.xml_root.is_dir() {
        return Err(format!(
            "missing generated XML directory {}; regenerate browser parity XML",
            config.xml_root.display()
        ));
    }
    let expected_helper_sha256 = sha256_bytes(TEST_HELPER_SOURCE.as_bytes());
    let expected_browser = expected_xml_browser_provenance(config);
    let mut linked_resource_cache =
        BTreeMap::<PathBuf, Vec<GeneratedLinkedResourceProvenance>>::new();
    for rel in collect_relative_files(&config.xml_root)? {
        if rel.extension().and_then(|extension| extension.to_str()) != Some("xml") {
            continue;
        }
        let path = config.xml_root.join(&rel);
        let raw = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read generated XML {}: {error}", path.display()))?;
        let provenance = parse_generated_provenance_comment(&raw).map_err(|error| {
            format!(
                "{} has invalid surgeist-layout-generate provenance: {error}",
                path.display()
            )
        })?;
        let source_rel = local_source_from_provenance(&provenance.source).map_err(|error| {
            format!("{} has invalid provenance source: {error}", path.display())
        })?;
        let source_path = config.root.join(&source_rel);
        let expected_source_sha256 = sha256_file(&source_path)?;
        if provenance.source_sha256 != expected_source_sha256 {
            return Err(format!(
                "{} generated XML source-sha256 is {}, expected {}; regenerate browser parity XML",
                path.display(),
                provenance.source_sha256,
                expected_source_sha256
            ));
        }
        let expected_linked_resources = if let Some(cached) = linked_resource_cache.get(&source_rel)
        {
            cached.clone()
        } else {
            let expected = expected_linked_resource_provenance(config, &source_rel, &source_path)?;
            linked_resource_cache.insert(source_rel.clone(), expected.clone());
            expected
        };
        if provenance.linked_resources_recorded
            && provenance.linked_resources != expected_linked_resources
        {
            return Err(format!(
                "{} generated XML linked-resource-sha256 is {}, expected {}; linked support resource provenance is stale; regenerate browser parity XML",
                path.display(),
                render_linked_resource_provenance(&provenance.linked_resources),
                render_linked_resource_provenance(&expected_linked_resources)
            ));
        }
        if provenance.helper_sha256 != expected_helper_sha256 {
            return Err(format!(
                "{} generated XML helper-sha256 is {}, expected {}; regenerate browser parity XML",
                path.display(),
                provenance.helper_sha256,
                expected_helper_sha256
            ));
        }
        validate_xml_browser_provenance(&path, &provenance.browser, expected_browser.as_ref())?;
    }
    Ok(())
}

fn expected_xml_browser_provenance(config: &Config) -> Option<XmlBrowserProvenanceExpectation> {
    if let Some(path) = &config.browser_path {
        return Some(XmlBrowserProvenanceExpectation::Exact(
            path.display().to_string(),
        ));
    }
    config
        .browser_version
        .as_ref()
        .map(|version| XmlBrowserProvenanceExpectation::Prefix {
            expected: format!("chrome-for-testing/{version} ("),
            description: format!("chrome-for-testing/{version}"),
        })
}

fn validate_xml_browser_provenance(
    path: &Path,
    browser: &str,
    expected: Option<&XmlBrowserProvenanceExpectation>,
) -> Result<(), String> {
    if browser.trim().is_empty() {
        return Err(format!(
            "{} generated XML browser provenance is empty; regenerate browser parity XML",
            path.display()
        ));
    }
    match expected {
        Some(XmlBrowserProvenanceExpectation::Exact(expected)) if browser != expected => {
            Err(format!(
                "{} generated XML browser provenance is {browser:?}, expected {expected:?}; regenerate browser parity XML",
                path.display()
            ))
        }
        Some(XmlBrowserProvenanceExpectation::Prefix {
            expected,
            description,
        }) if !browser.starts_with(expected) => Err(format!(
            "{} generated XML browser provenance is {browser:?}, expected {description}; regenerate browser parity XML",
            path.display()
        )),
        _ => Ok(()),
    }
}

fn parse_generated_provenance_comment(raw: &str) -> Result<GeneratedProvenance, String> {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("<!-- generated-by: surgeist-layout-generate ") {
        return Err("missing generated-by comment".to_string());
    }
    let end = trimmed
        .find("-->")
        .ok_or_else(|| "unterminated generated-by comment".to_string())?;
    let comment = &trimmed[..end];
    let linked_resource_attr = provenance_attr_optional(comment, "linked-resource-sha256")?;
    let linked_resources_recorded = linked_resource_attr.is_some();
    Ok(GeneratedProvenance {
        source: provenance_attr(comment, "source")?,
        source_sha256: provenance_attr(comment, "source-sha256")?,
        linked_resources: parse_linked_resource_provenance(&linked_resource_attr)?,
        linked_resources_recorded,
        helper_sha256: provenance_attr(comment, "helper-sha256")?,
        browser: provenance_attr(comment, "browser")?,
    })
}

fn provenance_attr_optional(comment: &str, key: &str) -> Result<Option<String>, String> {
    let marker = format!("{key}=\"");
    let Some(start) = comment.find(&marker).map(|start| start + marker.len()) else {
        return Ok(None);
    };
    let value = &comment[start..];
    let end = value
        .find('"')
        .ok_or_else(|| format!("unterminated `{key}` attribute"))?;
    Ok(Some(unescape_attr(&value[..end])))
}

fn provenance_attr(comment: &str, key: &str) -> Result<String, String> {
    let marker = format!("{key}=\"");
    let start = comment
        .find(&marker)
        .ok_or_else(|| format!("missing `{key}` attribute"))?
        + marker.len();
    let value = &comment[start..];
    let end = value
        .find('"')
        .ok_or_else(|| format!("unterminated `{key}` attribute"))?;
    Ok(unescape_attr(&value[..end]))
}

fn unescape_attr(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

fn local_source_from_provenance(source: &str) -> Result<PathBuf, String> {
    let local = source
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .replace('\\', "/");
    if local.is_empty() {
        return Err("empty source".to_string());
    }
    let path = PathBuf::from(&local);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(format!("source `{local}` must be a local relative path"));
    }
    Ok(path)
}

fn expected_linked_resource_provenance(
    config: &Config,
    source_rel: &Path,
    source_path: &Path,
) -> Result<Vec<GeneratedLinkedResourceProvenance>, String> {
    let _ = (config, source_rel, source_path);
    Ok(Vec::new())
}

fn generation_report_path(config: &Config) -> PathBuf {
    let scope = config
        .filter
        .as_deref()
        .map(generation_report_scope)
        .unwrap_or_else(|| "all".to_string());
    config
        .xml_root
        .join("generation-reports")
        .join(format!("{scope}.json"))
}

fn generation_report_scope(filter: &str) -> String {
    let normalized = filter.trim_matches('/');
    if normalized.is_empty() {
        return "all".to_string();
    }
    normalized
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GeneratedProvenance {
    source: String,
    source_sha256: String,
    linked_resources: Vec<GeneratedLinkedResourceProvenance>,
    linked_resources_recorded: bool,
    helper_sha256: String,
    browser: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GeneratedLinkedResourceProvenance {
    path: String,
    sha256: String,
}

enum XmlBrowserProvenanceExpectation {
    Exact(String),
    Prefix {
        expected: String,
        description: String,
    },
}

fn output_paths_for_fixture(
    html_root: &Path,
    xml_root: &Path,
    fixture: &Path,
) -> Result<Vec<PathBuf>, String> {
    let rel = fixture.strip_prefix(html_root).map_err(|error| {
        format!(
            "failed to make fixture path relative to {}: {error}",
            html_root.display()
        )
    })?;
    let group = rel.parent().unwrap_or_else(|| Path::new(""));
    let stem = fixture
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| format!("fixture has no UTF-8 stem: {}", fixture.display()))?;
    Ok(fixture_cases()
        .into_iter()
        .map(|(variant, _)| xml_root.join(group).join(format!("{stem}__{variant}.xml")))
        .collect())
}

fn browser_provenance(config: &Config, browser_path: &Path) -> String {
    if config.browser_path.is_some() {
        return browser_path.display().to_string();
    }
    match &config.browser_version {
        Some(version) => format!("chrome-for-testing/{version} ({})", browser_path.display()),
        None => browser_path.display().to_string(),
    }
}

fn root_relative_source(root: &Path, source_path: &Path) -> Result<String, String> {
    let root = absolutize_path(root)?;
    let source_path = absolutize_path(source_path)?;
    let relative = source_path.strip_prefix(&root).map_err(|error| {
        format!(
            "failed to make source path {} relative to {}: {error}",
            source_path.display(),
            root.display()
        )
    })?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn parse_linked_resource_provenance(
    raw: &Option<String>,
) -> Result<Vec<GeneratedLinkedResourceProvenance>, String> {
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    raw.split(',')
        .map(|entry| {
            let (path, sha256) = entry
                .split_once('=')
                .ok_or_else(|| format!("invalid linked-resource-sha256 entry `{entry}`"))?;
            Ok(GeneratedLinkedResourceProvenance {
                path: path.to_string(),
                sha256: sha256.to_string(),
            })
        })
        .collect()
}

fn render_linked_resource_provenance(resources: &[GeneratedLinkedResourceProvenance]) -> String {
    resources
        .iter()
        .map(|resource| format!("{}={}", resource.path, resource.sha256))
        .collect::<Vec<_>>()
        .join(",")
}

fn absolutize_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))
            .map(|cwd| cwd.join(path))
    }
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let raw =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ok(sha256_bytes(&raw))
}

fn sha256_bytes(raw: &[u8]) -> String {
    let digest = Sha256::digest(raw);
    hex_digest(&digest)
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut text, "{byte:02x}").expect("writing to string should not fail");
    }
    text
}

fn unsupported_fixture_reason(node: &Value) -> Option<&str> {
    if let Some(reason) = node["unsupportedReason"].as_str() {
        return Some(reason);
    }

    node["children"]
        .as_array()
        .and_then(|children| children.iter().find_map(unsupported_fixture_reason))
}

fn fixture_cases() -> [(&'static str, &'static str); 4] {
    [
        ("border_box_ltr", "borderBoxLtrData"),
        ("content_box_ltr", "contentBoxLtrData"),
        ("border_box_rtl", "borderBoxRtlData"),
        ("content_box_rtl", "contentBoxRtlData"),
    ]
}

#[cfg(test)]
fn generate_xml(name: &str, node: &Value) -> String {
    generate_xml_with_provenance(name, node, None)
}

fn generate_xml_with_provenance(
    name: &str,
    node: &Value,
    provenance: Option<&GeneratedProvenance>,
) -> String {
    let mut lines = Vec::new();
    if let Some(provenance) = provenance {
        let linked_resources = &provenance.linked_resources;
        let linked_resources = if provenance.linked_resources.is_empty() {
            String::new()
        } else {
            format!(
                " linked-resource-sha256=\"{}\"",
                escape_attr(render_linked_resource_provenance(linked_resources))
            )
        };
        lines.push(format!(
            "<!-- generated-by: surgeist-layout-generate schema=1 source=\"{}\" source-sha256=\"{}\"{} helper-sha256=\"{}\" browser=\"{}\" -->",
            escape_attr(&provenance.source),
            escape_attr(&provenance.source_sha256),
            linked_resources,
            escape_attr(&provenance.helper_sha256),
            escape_attr(&provenance.browser),
        ));
    }
    let use_rounding = bool_field(node, "useRounding");
    lines.push(format!(
        "<test name=\"{}\" use-rounding=\"{}\">",
        escape_attr(name),
        use_rounding
    ));
    let viewport = &node["viewport"];
    let root_context = viewport["rootContext"].as_str().unwrap_or("root");
    let root_context_attr = if root_context == "root" {
        String::new()
    } else {
        format!(" root-context=\"{}\"", escape_attr(root_context))
    };
    lines.push(format!(
        "  <viewport width=\"{}\" height=\"{}\"{}/>",
        dimension(&viewport["width"]).unwrap_or_default(),
        dimension(&viewport["height"]).unwrap_or_default(),
        root_context_attr
    ));
    lines.push("  <input>".to_string());
    write_input(&mut lines, node, 4, "horizontal-tb");
    lines.push("  </input>".to_string());
    lines.push("  <expectations>".to_string());
    write_expectation(
        &mut lines,
        node,
        ExpectationWriteContext {
            use_rounding,
            indent: 4,
            is_root: true,
            parent_abs_x: 0.0,
            parent_abs_y: 0.0,
        },
    );
    lines.push("  </expectations>".to_string());
    lines.push("</test>".to_string());
    format!("{}\n", lines.join("\n"))
}

fn write_input(lines: &mut Vec<String>, node: &Value, indent: usize, parent_writing_mode: &str) {
    let attrs = input_attrs_with_parent_writing_mode(node, parent_writing_mode);
    let writing_mode =
        string(&node["style"], "writingMode").unwrap_or_else(|| "horizontal-tb".to_string());
    let tag = if node.get("textContent").is_some() && !direct_text_requires_container(node) {
        "text"
    } else {
        "div"
    };
    let pad = " ".repeat(indent);
    let children = node["children"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    if children.is_empty() && node.get("textContent").is_none() {
        lines.push(format!("{pad}<{tag}{}/>", attr_text(&attrs)));
        return;
    }

    lines.push(format!("{pad}<{tag}{}>", attr_text(&attrs)));
    for child in children {
        write_input(lines, child, indent + 2, &writing_mode);
    }
    if let Some(text) = node["textContent"].as_str() {
        lines.push(format!(
            "{}{}",
            " ".repeat(indent + 2),
            escape_text(text.trim())
        ));
    }
    lines.push(format!("{pad}</{tag}>"));
}

fn direct_text_requires_container(node: &Value) -> bool {
    matches!(
        node["style"]["display"].as_str(),
        Some("grid" | "inline-grid" | "grid-lanes" | "inline-grid-lanes")
    )
}

#[derive(Clone, Copy, Debug)]
struct ExpectationWriteContext {
    use_rounding: bool,
    indent: usize,
    is_root: bool,
    parent_abs_x: f64,
    parent_abs_y: f64,
}

impl ExpectationWriteContext {
    fn child(self, abs_x: f64, abs_y: f64) -> Self {
        Self {
            indent: self.indent + 2,
            is_root: false,
            parent_abs_x: abs_x,
            parent_abs_y: abs_y,
            ..self
        }
    }
}

fn write_expectation(lines: &mut Vec<String>, node: &Value, context: ExpectationWriteContext) {
    let layout = if context.use_rounding {
        &node["smartRoundedLayout"]
    } else {
        &node["unroundedLayout"]
    };
    let abs_x = if context.is_root {
        0.0
    } else {
        context.parent_abs_x + number(&layout["x"])
    };
    let abs_y = if context.is_root {
        0.0
    } else {
        context.parent_abs_y + number(&layout["y"])
    };
    let mut attrs = vec![
        (
            "x",
            if context.is_root {
                "0".to_string()
            } else {
                layout_number_attr(&layout["x"])
            },
        ),
        (
            "y",
            if context.is_root {
                "0".to_string()
            } else {
                layout_number_attr(&layout["y"])
            },
        ),
        ("width", layout_number_attr(&layout["width"])),
        ("height", layout_number_attr(&layout["height"])),
    ];

    let overflow_x = node["style"]["overflowX"].as_str().unwrap_or_default();
    let overflow_y = node["style"]["overflowY"].as_str().unwrap_or_default();
    if ["hidden", "scroll", "auto"].contains(&overflow_x)
        || ["hidden", "scroll", "auto"].contains(&overflow_y)
    {
        let client = &node["naivelyRoundedLayout"];
        attrs.push((
            "scroll_width",
            layout_number_attr_value(
                (number(&layout["scrollWidth"]) - number(&client["clientWidth"])).max(0.0),
            ),
        ));
        attrs.push((
            "scroll_height",
            layout_number_attr_value(
                (number(&layout["scrollHeight"]) - number(&client["clientHeight"])).max(0.0),
            ),
        ));
    }

    let pad = " ".repeat(context.indent);
    let children = node["children"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if children.is_empty() {
        lines.push(format!("{pad}<node{}/>", attr_text(&attrs)));
        return;
    }

    lines.push(format!("{pad}<node{}>", attr_text(&attrs)));
    for child in children {
        write_expectation(lines, child, context.child(abs_x, abs_y));
    }
    lines.push(format!("{pad}</node>"));
}

#[cfg(test)]
fn input_attrs(node: &Value) -> Vec<(&'static str, String)> {
    input_attrs_with_parent_writing_mode(node, "horizontal-tb")
}

fn input_attrs_with_parent_writing_mode(
    node: &Value,
    parent_writing_mode: &str,
) -> Vec<(&'static str, String)> {
    let style = &node["style"];
    let mut attrs = Vec::new();
    maybe(&mut attrs, "source-tag", string(node, "tagName"), None);
    maybe(&mut attrs, "display", string(style, "display"), None);
    maybe(
        &mut attrs,
        "box-sizing",
        string(style, "boxSizing"),
        Some("border-box"),
    );
    maybe(&mut attrs, "direction", string(style, "direction"), None);
    if let Some(writing_mode) = writing_mode_attr(style, parent_writing_mode) {
        attrs.push(("writing-mode", writing_mode));
    }
    maybe(
        &mut attrs,
        "position",
        string(style, "position"),
        Some("relative"),
    );
    maybe(&mut attrs, "float", string(style, "cssFloat"), None);
    maybe(&mut attrs, "clear", string(style, "clear"), None);
    maybe(
        &mut attrs,
        "flex-direction",
        string(style, "flexDirection"),
        Some("row"),
    );
    maybe(
        &mut attrs,
        "flex-wrap",
        string(style, "flexWrap"),
        Some("nowrap"),
    );
    maybe(
        &mut attrs,
        "overflow-x",
        string(style, "overflowX"),
        Some("visible"),
    );
    maybe(
        &mut attrs,
        "overflow-y",
        string(style, "overflowY"),
        Some("visible"),
    );
    if non_default_overflow(style, "overflowX") || non_default_overflow(style, "overflowY") {
        maybe(
            &mut attrs,
            "scrollbar-width",
            number_string(style, "scrollbarWidth"),
            None,
        );
    }
    maybe(&mut attrs, "text-align", string(style, "textAlign"), None);
    maybe(
        &mut attrs,
        "vertical-align",
        string(style, "verticalAlign"),
        Some("baseline"),
    );
    maybe(&mut attrs, "font-family", font_family(style), Some("ahem"));
    maybe(
        &mut attrs,
        "font-size",
        dimension(&style["fontSize"]),
        Some("10px"),
    );
    maybe(
        &mut attrs,
        "line-height",
        dimension(&style["lineHeight"]),
        Some("10px"),
    );
    maybe(&mut attrs, "align-items", string(style, "alignItems"), None);
    maybe(&mut attrs, "align-self", string(style, "alignSelf"), None);
    maybe(
        &mut attrs,
        "justify-items",
        string(style, "justifyItems"),
        None,
    );
    maybe(
        &mut attrs,
        "justify-self",
        string(style, "justifySelf"),
        None,
    );
    maybe(
        &mut attrs,
        "align-content",
        string(style, "alignContent"),
        None,
    );
    maybe(
        &mut attrs,
        "justify-content",
        string(style, "justifyContent"),
        None,
    );
    maybe(
        &mut attrs,
        "flex-grow",
        number_string(style, "flexGrow"),
        Some("0"),
    );
    maybe(
        &mut attrs,
        "flex-shrink",
        number_string(style, "flexShrink"),
        Some("1"),
    );
    maybe(
        &mut attrs,
        "flex-basis",
        dimension(&style["flexBasis"]),
        Some("auto"),
    );
    maybe(
        &mut attrs,
        "width",
        dimension(&style["size"]["width"]),
        Some("auto"),
    );
    maybe(
        &mut attrs,
        "height",
        dimension(&style["size"]["height"]),
        Some("auto"),
    );
    maybe(
        &mut attrs,
        "min-width",
        dimension(&style["minSize"]["width"]),
        Some("auto"),
    );
    maybe(
        &mut attrs,
        "min-height",
        dimension(&style["minSize"]["height"]),
        Some("auto"),
    );
    maybe(
        &mut attrs,
        "max-width",
        dimension(&style["maxSize"]["width"]),
        Some("auto"),
    );
    maybe(
        &mut attrs,
        "max-height",
        dimension(&style["maxSize"]["height"]),
        Some("auto"),
    );
    maybe(
        &mut attrs,
        "aspect-ratio",
        number_string(style, "aspectRatio"),
        None,
    );
    maybe(&mut attrs, "row-gap", dimension(&style["gap"]["row"]), None);
    maybe(
        &mut attrs,
        "column-gap",
        dimension(&style["gap"]["column"]),
        None,
    );
    edge_attrs(
        &mut attrs,
        "margin",
        &style["margin"],
        ["top", "left", "bottom", "right"],
    );
    logical_inline_margin_attrs(&mut attrs, style);
    edge_attrs(
        &mut attrs,
        "padding",
        &style["padding"],
        ["top", "left", "bottom", "right"],
    );
    edge_attrs(
        &mut attrs,
        "border",
        &style["border"],
        ["top", "left", "bottom", "right"],
    );
    edge_attrs(
        &mut attrs,
        "",
        &style["inset"],
        ["top", "left", "bottom", "right"],
    );
    maybe(
        &mut attrs,
        "grid-auto-flow",
        grid_auto_flow(&style["gridAutoFlow"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-template-rows",
        dimension_list(&style["gridTemplateRows"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-template-columns",
        dimension_list(&style["gridTemplateColumns"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-template-areas",
        grid_template_areas(&style["gridTemplateAreas"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-auto-rows",
        dimension_list(&style["gridAutoRows"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-auto-columns",
        dimension_list(&style["gridAutoColumns"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-row-start",
        grid_position(&style["gridRowStart"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-row-end",
        grid_position(&style["gridRowEnd"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-column-start",
        grid_position(&style["gridColumnStart"]),
        None,
    );
    maybe(
        &mut attrs,
        "grid-column-end",
        grid_position(&style["gridColumnEnd"]),
        None,
    );
    attrs
}

fn writing_mode_attr(style: &Value, parent_writing_mode: &str) -> Option<String> {
    let writing_mode = string(style, "writingMode").unwrap_or_else(|| "horizontal-tb".to_string());
    if writing_mode.starts_with("vertical-")
        || (writing_mode == "horizontal-tb" && parent_writing_mode.starts_with("vertical-"))
    {
        Some(writing_mode)
    } else {
        None
    }
}

fn maybe(
    attrs: &mut Vec<(&'static str, String)>,
    key: &'static str,
    value: Option<String>,
    elide: Option<&str>,
) {
    if let Some(value) = value
        && elide != Some(value.as_str())
    {
        attrs.push((key, value));
    }
}

fn edge_attrs(
    attrs: &mut Vec<(&'static str, String)>,
    prefix: &'static str,
    edges: &Value,
    names: [&'static str; 4],
) {
    for name in names {
        let key = match (prefix, name) {
            ("margin", "top") => "margin-top",
            ("margin", "right") => "margin-right",
            ("margin", "bottom") => "margin-bottom",
            ("margin", "left") => "margin-left",
            ("padding", "top") => "padding-top",
            ("padding", "right") => "padding-right",
            ("padding", "bottom") => "padding-bottom",
            ("padding", "left") => "padding-left",
            ("border", "top") => "border-top",
            ("border", "right") => "border-right",
            ("border", "bottom") => "border-bottom",
            ("border", "left") => "border-left",
            ("", "top") => "top",
            ("", "right") => "right",
            ("", "bottom") => "bottom",
            ("", "left") => "left",
            _ => continue,
        };
        maybe(attrs, key, dimension(&edges[name]), None);
    }
}

fn logical_inline_margin_attrs(attrs: &mut Vec<(&'static str, String)>, style: &Value) {
    let logical = &style["logicalMargin"];
    if logical.is_null() {
        return;
    }

    let (start_attr, end_attr) = if string(style, "direction").as_deref() == Some("rtl") {
        ("margin-right", "margin-left")
    } else {
        ("margin-left", "margin-right")
    };
    maybe_edge_attr(attrs, start_attr, dimension(&logical["inlineStart"]));
    maybe_edge_attr(attrs, end_attr, dimension(&logical["inlineEnd"]));
}

fn maybe_edge_attr(
    attrs: &mut Vec<(&'static str, String)>,
    key: &'static str,
    value: Option<String>,
) {
    if let Some(value) = value {
        if let Some((_, existing_value)) = attrs.iter_mut().find(|(existing, _)| *existing == key) {
            *existing_value = value;
        } else {
            attrs.push((key, value));
        }
    }
}

fn attr_text(attrs: &[(&str, String)]) -> String {
    if attrs.is_empty() {
        String::new()
    } else {
        format!(
            " {}",
            attrs
                .iter()
                .map(|(key, value)| format!("{key}=\"{}\"", escape_attr(value)))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

fn dimension(value: &Value) -> Option<String> {
    let unit = value.get("unit").and_then(Value::as_str)?;
    match unit {
        "auto" | "max-content" | "min-content" => Some(unit.to_string()),
        "px" => Some(format!("{}px", number_attr(&value["value"]))),
        "percent" => Some(format!(
            "{}%",
            number_attr_value(number(&value["value"]) * 100.0)
        )),
        "fraction" => Some(format!("{}fr", number_attr(&value["value"]))),
        "calc" => value
            .get("value")
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

fn dimension_list(values: &Value) -> Option<String> {
    let values = values.as_array()?;
    let serialized = values
        .iter()
        .filter_map(track_definition)
        .collect::<Vec<_>>();
    (!serialized.is_empty()).then(|| serialized.join(" "))
}

fn grid_template_areas(value: &Value) -> Option<String> {
    let rows = value.as_array()?;
    let serialized = rows
        .iter()
        .map(grid_template_area_row)
        .collect::<Option<Vec<_>>>()?;
    (!serialized.is_empty()).then(|| serialized.join(" / "))
}

fn grid_template_area_row(value: &Value) -> Option<String> {
    let cells = value.as_array()?;
    let serialized = cells
        .iter()
        .map(|cell| {
            if cell.is_null() {
                Some(".")
            } else {
                cell.as_str()
            }
        })
        .collect::<Option<Vec<_>>>()?;
    (!serialized.is_empty()).then(|| serialized.join(" "))
}

fn track_definition(value: &Value) -> Option<String> {
    match value.get("kind").and_then(Value::as_str) {
        Some("scalar") | None => dimension(value),
        Some("line-names") => line_names_track_definition(value),
        Some("subgrid") => Some(subgrid_track_definition(value)),
        Some("function") => {
            let name = value["name"].as_str()?;
            let arguments = value["arguments"].as_array()?;
            match name {
                "fit-content" => {
                    let limit = dimension(arguments.first()?)?;
                    Some(format!("fit-content({limit})"))
                }
                "minmax" => {
                    let min = dimension(arguments.first()?)?;
                    let max = dimension(arguments.get(1)?)?;
                    Some(format!("minmax({min},{max})"))
                }
                "repeat" => {
                    let repetition = repetition(arguments.first()?)?;
                    let tracks = arguments
                        .iter()
                        .skip(1)
                        .map(track_definition)
                        .collect::<Option<Vec<_>>>()?
                        .join(" ");
                    Some(format!("repeat({repetition}, {tracks})"))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn line_names_track_definition(value: &Value) -> Option<String> {
    let names = value.get("names")?.as_array()?;
    let names = names
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()?;
    Some(format!("[{}]", names.join(" ")))
}

fn subgrid_track_definition(value: &Value) -> String {
    let mut parts = vec!["subgrid".to_string()];
    if let Some(line_names) = value.get("lineNames").and_then(Value::as_array) {
        parts.extend(line_names.iter().filter_map(|names| {
            let names = names.as_array()?;
            let names = names
                .iter()
                .map(Value::as_str)
                .collect::<Option<Vec<_>>>()?;
            Some(format!("[{}]", names.join(" ")))
        }));
    }
    parts.join(" ")
}

fn repetition(value: &Value) -> Option<String> {
    match value["unit"].as_str()? {
        "auto-fill" => Some("auto-fill".to_string()),
        "auto-fit" => Some("auto-fit".to_string()),
        "integer" => Some(number_attr(&value["value"])),
        _ => None,
    }
}

fn grid_auto_flow(value: &Value) -> Option<String> {
    let direction = value["direction"].as_str()?;
    match value["algorithm"].as_str() {
        Some("dense") => Some(format!("{direction} dense")),
        _ => Some(direction.to_string()),
    }
}

fn grid_position(value: &Value) -> Option<String> {
    match value["kind"].as_str()? {
        "auto" => None,
        "span" => Some(format!("span {}", number_attr(&value["value"]))),
        "line" => Some(number_attr(&value["value"])),
        "named-line" => {
            let name = value["name"].as_str()?;
            let index = value["value"].as_f64()?;
            if index == 0.0 {
                Some(name.to_string())
            } else {
                Some(format!("{name} {}", number_attr(&value["value"])))
            }
        }
        "named-span" => {
            let name = value["name"].as_str()?;
            let span = value["value"].as_f64()?;
            if span == 0.0 {
                Some(format!("span {name}"))
            } else {
                Some(format!("span {} {name}", number_attr(&value["value"])))
            }
        }
        _ => None,
    }
}

fn string(value: &Value, key: &str) -> Option<String> {
    value[key].as_str().map(ToString::to_string)
}

fn font_family(value: &Value) -> Option<String> {
    let family = string(value, "fontFamily")?.replace('"', "");
    let primary = family.split(',').next()?.trim().to_ascii_lowercase();
    match primary.as_str() {
        "ahem" | "monospace" => Some(primary),
        _ => None,
    }
}

fn number_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(|value| {
        if value.is_null() {
            None
        } else {
            Some(number_attr(value))
        }
    })
}

fn non_default_overflow(style: &Value, key: &str) -> bool {
    string(style, key).is_some_and(|value| value != "visible")
}

fn bool_field(value: &Value, key: &str) -> bool {
    value[key].as_bool().unwrap_or(false)
}

fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or(0.0)
}

fn number_attr(value: &Value) -> String {
    number_attr_value(number(value))
}

fn number_attr_value(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn layout_number_attr(value: &Value) -> String {
    layout_number_attr_value(number(value))
}

fn layout_number_attr_value(value: f64) -> String {
    let value = value as f32;
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn escape_attr(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

fn escape_text(value: impl AsRef<str>) -> String {
    value.as_ref().replace('&', "&amp;").replace('<', "&lt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn unsupported_node(reason: &str) -> Value {
        json!({
            "unsupportedReason": reason,
            "useRounding": true,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {"display": "block"},
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 0, "height": 0, "scrollWidth": 0, "scrollHeight": 0},
            "unroundedLayout": {"x": 0, "y": 0, "width": 0, "height": 0, "scrollWidth": 0, "scrollHeight": 0},
            "naivelyRoundedLayout": {"clientWidth": 0, "clientHeight": 0},
            "children": []
        })
    }

    #[test]
    fn fixture_cases_match_browser_measurement_keys() {
        assert_eq!(
            fixture_cases(),
            [
                ("border_box_ltr", "borderBoxLtrData"),
                ("content_box_ltr", "contentBoxLtrData"),
                ("border_box_rtl", "borderBoxRtlData"),
                ("content_box_rtl", "contentBoxRtlData"),
            ]
        );
    }

    #[test]
    fn xml_generation_preserves_browser_parity_fixture_shape() {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {"display": "block", "direction": "ltr", "size": {"width": {"unit": "px", "value": 50}}},
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 50, "height": 20, "scrollWidth": 50, "scrollHeight": 20},
            "unroundedLayout": {"x": 0, "y": 0, "width": 50, "height": 20, "scrollWidth": 50, "scrollHeight": 20},
            "naivelyRoundedLayout": {"clientWidth": 50, "clientHeight": 20},
            "children": [
                {
                    "useRounding": true,
                    "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
                    "style": {"direction": "ltr", "size": {"height": {"unit": "px", "value": 10}}},
                    "smartRoundedLayout": {"x": 0, "y": 0, "width": 50, "height": 10, "scrollWidth": 50, "scrollHeight": 10},
                    "unroundedLayout": {"x": 0, "y": 0, "width": 50, "height": 10, "scrollWidth": 50, "scrollHeight": 10},
                    "naivelyRoundedLayout": {"clientWidth": 50, "clientHeight": 10},
                    "children": []
                }
            ]
        });

        let xml = generate_xml("block_basic__border_box_ltr", &node);

        assert!(xml.contains("<test name=\"block_basic__border_box_ltr\" use-rounding=\"true\">"));
        assert!(xml.contains("  <viewport width=\"max-content\" height=\"max-content\"/>"));
        assert!(xml.contains("    <div display=\"block\" direction=\"ltr\" width=\"50px\">"));
        assert!(xml.contains("      <div direction=\"ltr\" height=\"10px\"/>"));
        assert!(xml.contains("    <node x=\"0\" y=\"0\" width=\"50\" height=\"20\">"));
        assert!(xml.contains("      <node x=\"0\" y=\"0\" width=\"50\" height=\"10\"/>"));
    }

    #[test]
    fn xml_generation_normalizes_root_expectation_to_origin() {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {"display": "inline-grid"},
            "smartRoundedLayout": {"x": 7, "y": 4, "width": 50, "height": 20, "scrollWidth": 50, "scrollHeight": 20},
            "unroundedLayout": {"x": 7, "y": 4, "width": 50, "height": 20, "scrollWidth": 50, "scrollHeight": 20},
            "naivelyRoundedLayout": {"clientWidth": 50, "clientHeight": 20},
            "children": [
                {
                    "useRounding": true,
                    "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
                    "style": {"display": "block"},
                    "smartRoundedLayout": {"x": 1, "y": 2, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
                    "unroundedLayout": {"x": 1, "y": 2, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
                    "naivelyRoundedLayout": {"clientWidth": 10, "clientHeight": 10},
                    "children": []
                }
            ]
        });

        let xml = generate_xml("inline_grid__border_box_ltr", &node);

        assert!(xml.contains("    <node x=\"0\" y=\"0\" width=\"50\" height=\"20\">"));
        assert!(xml.contains("      <node x=\"1\" y=\"2\" width=\"10\" height=\"10\"/>"));
    }

    #[test]
    fn xml_generation_marks_viewport_flex_item_root_context() {
        let node = json!({
            "useRounding": true,
            "viewport": {
                "width": {"unit": "px", "value": 400},
                "height": {"unit": "max-content"},
                "rootContext": "flex-item"
            },
            "style": {"display": "grid"},
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 160, "height": 20, "scrollWidth": 160, "scrollHeight": 20},
            "unroundedLayout": {"x": 0, "y": 0, "width": 160, "height": 20, "scrollWidth": 160, "scrollHeight": 20},
            "naivelyRoundedLayout": {"clientWidth": 160, "clientHeight": 20},
            "children": []
        });

        let xml = generate_xml("grid_flex_item__border_box_ltr", &node);

        assert!(xml.contains(
            "  <viewport width=\"400px\" height=\"max-content\" root-context=\"flex-item\"/>"
        ));
    }

    #[test]
    fn xml_generation_keeps_grid_text_elements_as_containers() {
        for display in ["grid", "inline-grid", "grid-lanes", "inline-grid-lanes"] {
            let node = json!({
                "useRounding": true,
                "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
                "style": {"display": display, "direction": "ltr"},
                "textContent": "hello",
                "smartRoundedLayout": {"x": 0, "y": 0, "width": 50, "height": 10, "scrollWidth": 50, "scrollHeight": 10},
                "unroundedLayout": {"x": 0, "y": 0, "width": 50, "height": 10, "scrollWidth": 50, "scrollHeight": 10},
                "naivelyRoundedLayout": {"clientWidth": 50, "clientHeight": 10},
                "children": []
            });

            let xml = generate_xml("grid_text__border_box_ltr", &node);

            assert!(xml.contains(&format!(
                "    <div display=\"{display}\" direction=\"ltr\">"
            )));
            assert!(xml.contains("      hello"));
            assert!(!xml.contains(&format!("<text display=\"{display}\"")));
        }
    }

    #[test]
    fn xml_generation_preserves_explicit_grid_line_names() {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {
                "display": "grid-lanes",
                "gridTemplateColumns": [
                    {"kind": "line-names", "names": ["lane"]},
                    {"kind": "scalar", "unit": "px", "value": 20},
                    {"kind": "line-names", "names": ["lane"]},
                    {"kind": "scalar", "unit": "px", "value": 30},
                    {"kind": "line-names", "names": ["lane"]},
                    {"kind": "scalar", "unit": "px", "value": 40},
                    {"kind": "line-names", "names": ["lane"]}
                ]
            },
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 90, "height": 0, "scrollWidth": 90, "scrollHeight": 0},
            "unroundedLayout": {"x": 0, "y": 0, "width": 90, "height": 0, "scrollWidth": 90, "scrollHeight": 0},
            "naivelyRoundedLayout": {"clientWidth": 90, "clientHeight": 0},
            "children": []
        });

        let xml = generate_xml("grid_lanes_named_placement__border_box_ltr", &node);

        assert!(
            xml.contains("grid-template-columns=\"[lane] 20px [lane] 30px [lane] 40px [lane]\"")
        );
    }

    #[test]
    fn xml_generation_preserves_calc_lengths() {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "px", "value": 200}, "height": {"unit": "max-content"}},
            "style": {
                "display": "block",
                "size": {"width": {"unit": "calc", "value": "calc(50% + 20px)"}},
                "margin": {"left": {"unit": "calc", "value": "calc(10% - 4px)"}}
            },
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 120, "height": 10, "scrollWidth": 120, "scrollHeight": 10},
            "unroundedLayout": {"x": 0, "y": 0, "width": 120, "height": 10, "scrollWidth": 120, "scrollHeight": 10},
            "naivelyRoundedLayout": {"clientWidth": 120, "clientHeight": 10},
            "children": []
        });

        let xml = generate_xml("calc_lengths__border_box_ltr", &node);

        assert!(xml.contains(r#"width="calc(50% + 20px)""#));
        assert!(xml.contains(r#"margin-left="calc(10% - 4px)""#));
    }

    #[test]
    fn xml_generation_preserves_calc_grid_tracks() {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "px", "value": 240}, "height": {"unit": "max-content"}},
            "style": {
                "display": "grid",
                "gridTemplateColumns": [
                    {"kind": "scalar", "unit": "calc", "value": "calc(25% + 20px)"},
                    {"kind": "scalar", "unit": "px", "value": 80}
                ]
            },
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 240, "height": 10, "scrollWidth": 240, "scrollHeight": 10},
            "unroundedLayout": {"x": 0, "y": 0, "width": 240, "height": 10, "scrollWidth": 240, "scrollHeight": 10},
            "naivelyRoundedLayout": {"clientWidth": 240, "clientHeight": 10},
            "children": []
        });

        let xml = generate_xml("calc_grid_tracks__border_box_ltr", &node);

        assert!(xml.contains(r#"grid-template-columns="calc(25% + 20px) 80px""#));
    }

    #[test]
    fn xml_generation_preserves_grid_template_areas() {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {
                "display": "grid",
                "gridTemplateRows": [
                    {"kind": "scalar", "unit": "px", "value": 20},
                    {"kind": "scalar", "unit": "px", "value": 40}
                ],
                "gridTemplateColumns": [
                    {"kind": "scalar", "unit": "px", "value": 30},
                    {"kind": "scalar", "unit": "px", "value": 50}
                ],
                "gridTemplateAreas": [
                    ["head", "head"],
                    ["nav", "main"]
                ]
            },
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 80, "height": 60, "scrollWidth": 80, "scrollHeight": 60},
            "unroundedLayout": {"x": 0, "y": 0, "width": 80, "height": 60, "scrollWidth": 80, "scrollHeight": 60},
            "naivelyRoundedLayout": {"clientWidth": 80, "clientHeight": 60},
            "children": []
        });

        let xml = generate_xml(
            "grid_named_template_area_generated_names__border_box_ltr",
            &node,
        );

        assert!(xml.contains("grid-template-areas=\"head head / nav main\""));
    }

    #[test]
    fn xml_generation_preserves_non_default_font_size() {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {
                "display": "block",
                "fontSize": {"unit": "px", "value": 12},
            },
            "textContent": "x",
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 12, "height": 12, "scrollWidth": 12, "scrollHeight": 12},
            "unroundedLayout": {"x": 0, "y": 0, "width": 12, "height": 12, "scrollWidth": 12, "scrollHeight": 12},
            "naivelyRoundedLayout": {"clientWidth": 12, "clientHeight": 12},
            "children": []
        });

        let xml = generate_xml("font_size__border_box_ltr", &node);

        assert!(xml.contains("    <text display=\"block\" font-size=\"12px\">"));
    }

    #[test]
    fn xml_generation_preserves_non_default_font_family() {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {
                "display": "block",
                "fontFamily": "monospace",
            },
            "textContent": "x",
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
            "unroundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
            "naivelyRoundedLayout": {"clientWidth": 10, "clientHeight": 10},
            "children": []
        });

        let xml = generate_xml("font_family__border_box_ltr", &node);

        assert!(xml.contains("    <text display=\"block\" font-family=\"monospace\">"));
    }

    #[test]
    fn xml_generation_elides_default_ahem_font_size() {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {
                "display": "block",
                "fontSize": {"unit": "px", "value": 10},
            },
            "textContent": "x",
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
            "unroundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
            "naivelyRoundedLayout": {"clientWidth": 10, "clientHeight": 10},
            "children": []
        });

        let xml = generate_xml("font_size__border_box_ltr", &node);

        assert!(!xml.contains("font-size"));
    }

    #[test]
    fn xml_generation_preserves_non_default_line_height() {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {
                "display": "block",
                "lineHeight": {"unit": "px", "value": 0},
            },
            "textContent": "x",
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 10, "height": 0, "scrollWidth": 10, "scrollHeight": 0},
            "unroundedLayout": {"x": 0, "y": 0, "width": 10, "height": 0, "scrollWidth": 10, "scrollHeight": 0},
            "naivelyRoundedLayout": {"clientWidth": 10, "clientHeight": 0},
            "children": []
        });

        let xml = generate_xml("line_height__border_box_ltr", &node);

        assert!(xml.contains("    <text display=\"block\" line-height=\"0px\">"));
    }

    #[test]
    fn xml_generation_preserves_vertical_align_top() {
        let node = json!({
            "useRounding": true,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {
                "display": "inline-grid-lanes",
                "verticalAlign": "top",
            },
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
            "unroundedLayout": {"x": 0, "y": 0, "width": 10, "height": 10, "scrollWidth": 10, "scrollHeight": 10},
            "naivelyRoundedLayout": {"clientWidth": 10, "clientHeight": 10},
            "children": []
        });

        let xml = generate_xml("vertical_align__border_box_ltr", &node);

        assert!(xml.contains("vertical-align=\"top\""));
    }

    #[test]
    fn bundled_helper_falls_back_to_computed_min_size_units() {
        assert!(
            TEST_HELPER_SOURCE.contains("parseResolvedDimension(lengthStyleValue(\"minWidth\")")
        );
        assert!(
            TEST_HELPER_SOURCE.contains("parseResolvedDimension(lengthStyleValue(\"minHeight\")")
        );
    }

    #[test]
    fn bundled_helper_preserves_resolved_percentage_min_max_size_values() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-resolved-percent-min-max-size-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        let script_path = root.join("resolved-percent-min-max-size.js");
        let script = format!(
            r#"
const window = {{}};
{TEST_HELPER_SOURCE}

const resolved = parseResolvedDimension("10%", "10%");
const actual = JSON.stringify(parseSize({{ width: resolved, height: resolved }}));
const expected = JSON.stringify({{
  width: {{ unit: "percent", value: 0.1 }},
  height: {{ unit: "percent", value: 0.1 }},
}});
if (actual !== expected) {{
  throw new Error(`resolved percentage min/max size should not be reparsed; got ${{actual}}`);
}}
"#
        );
        fs::write(&script_path, script).expect("script");

        let output = Command::new("node")
            .arg(&script_path)
            .output()
            .expect("node should run resolved percentage min/max size smoke test");

        assert!(
            output.status.success(),
            "node resolved percentage min/max size smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn bundled_helper_uses_computed_font_size() {
        assert!(TEST_HELPER_SOURCE.contains("fontSize: parseDimension(computedStyle.fontSize)"));
    }

    #[test]
    fn bundled_helper_uses_computed_display() {
        assert!(TEST_HELPER_SOURCE.contains("display: parseEnum(computedStyle.display)"));
    }

    #[test]
    fn bundled_helper_uses_computed_writing_mode() {
        assert!(TEST_HELPER_SOURCE.contains("writingMode: parseEnum(computedStyle.writingMode)"));
    }

    #[test]
    fn bundled_helper_lowers_authored_logical_size_to_physical_size() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-logical-size-capture-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        let script_path = root.join("logical-size-capture.js");
        let script = format!(
            r#"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{ styleSheets: [] }};

{TEST_HELPER_SOURCE}

function capture(style, writingMode) {{
  return parseElementSize((property) => style[property] || "", {{ writingMode }});
}}
function assertEqual(name, actual, expected) {{
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {{
    throw new Error(`${{name}} expected ${{expectedJson}} but got ${{actualJson}}`);
  }}
}}

assertEqual("horizontal", capture({{ inlineSize: "24px", blockSize: "72px" }}, "horizontal-tb"), {{
  width: {{ unit: "px", value: 24 }},
  height: {{ unit: "px", value: 72 }},
}});
assertEqual("vertical", capture({{ inlineSize: "24px", blockSize: "72px" }}, "vertical-rl"), {{
  width: {{ unit: "px", value: 72 }},
  height: {{ unit: "px", value: 24 }},
}});
assertEqual("physical wins", capture({{
  width: "10px",
  height: "11px",
  inlineSize: "24px",
  blockSize: "72px",
}}, "vertical-rl"), {{
  width: {{ unit: "px", value: 10 }},
  height: {{ unit: "px", value: 11 }},
}});
"#
        );
        fs::write(&script_path, script).expect("script");

        let output = Command::new("node")
            .arg(&script_path)
            .output()
            .expect("node should run logical size capture smoke test");

        assert!(
            output.status.success(),
            "node logical size capture smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn bundled_helper_records_vertical_align() {
        assert!(
            TEST_HELPER_SOURCE.contains("verticalAlign: parseEnum(styleValue(\"verticalAlign\"))")
        );
    }

    #[test]
    fn bundled_helper_captures_stylesheet_logical_margins_as_effective_physical_edges() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-effective-margin-capture-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        let script_path = root.join("effective-margin-capture.js");
        let script = format!(
            r#"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{
  styleSheets: [
    {{
      cssRules: [
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".logical",
          style: {{
            0: "margin-inline-start",
            length: 1,
            getPropertyValue(property) {{
              return property === "margin-inline-start" ? "13px" : "";
            }},
          }},
        }},
      ],
    }},
  ],
}};
function getComputedStyle() {{
  throw new Error("unexpected getComputedStyle call");
}}

{TEST_HELPER_SOURCE}

const element = {{
  classList: {{ contains() {{ return false; }} }},
  matches(selector) {{ return selector === ".logical"; }},
  style: {{
    length: 0,
    getPropertyValue() {{ return ""; }},
  }},
}};
const margin = parseEffectiveMargin(element, {{
  marginLeft: "13px",
  marginRight: "0px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}});
const actual = JSON.stringify(margin);
const expected = JSON.stringify({{ left: {{ unit: "px", value: 13 }} }});
if (actual !== expected) {{
  throw new Error(`unexpected effective margin ${{actual}}`);
}}
"#
        );
        fs::write(&script_path, script).expect("script");

        let output = Command::new("node")
            .arg(&script_path)
            .output()
            .expect("node should run effective margin capture smoke test");

        assert!(
            output.status.success(),
            "node effective margin capture smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn bundled_helper_preserves_authored_percentage_margin_values() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-percentage-margin-capture-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        let script_path = root.join("percentage-margin-capture.js");
        let script = format!(
            r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
function styleDeclaration(entries) {{
  const declaration = {{
    ...Object.fromEntries(entries.flatMap(([property, value]) => [
      [property, value],
      [property.replace(/-([a-z])/g, (_, c) => c.toUpperCase()), value],
    ])),
    length: entries.length,
    getPropertyValue(property) {{
      const entry = entries.find(([name]) => name === property);
      return entry ? entry[1] : "";
    }},
  }};
  entries.forEach(([property], index) => {{
    declaration[index] = property;
  }});
  return declaration;
}}
const document = {{
  styleSheets: [
    {{
      cssRules: [
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".shorthand",
          style: styleDeclaration([["margin", "5% 10% 15% 20%"]]),
        }},
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".logical",
          style: styleDeclaration([["margin-inline", "20% 10%"]]),
        }},
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".inline",
          style: styleDeclaration([["margin-left", "10px"]]),
        }},
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: "#specific",
          style: styleDeclaration([["margin-left", "10px"]]),
        }},
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".specific",
          style: styleDeclaration([["margin-left", "20%"]]),
        }},
      ],
    }},
  ],
}};
function getComputedStyle() {{
  throw new Error("unexpected getComputedStyle call");
}}

{TEST_HELPER_SOURCE}

function assertMargin(name, element, computedStyle, expected) {{
  const actual = JSON.stringify(parseEffectiveMargin(element, computedStyle));
  const expectedJson = JSON.stringify(expected);
  if (actual !== expectedJson) {{
    throw new Error(`${{name}} expected ${{expectedJson}} but got ${{actual}}`);
  }}
}}
function unit(value, unit) {{
  return {{ value, unit }};
}}
function elementFor(selectors, style, typedOmValues) {{
  const selectorSet = new Set(Array.isArray(selectors) ? selectors : [selectors]);
  return {{
    classList: {{ contains() {{ return false; }} }},
    matches(candidate) {{ return selectorSet.has(candidate); }},
    computedStyleMap() {{
      return {{
        get(property) {{
          return typedOmValues[property] || unit(0, "px");
        }},
      }};
    }},
    style,
  }};
}}

assertMargin("inline longhands", elementFor("", styleDeclaration([
  ["margin-left", "20%"],
  ["margin-right", "10%"],
]), {{
  "margin-left": unit(20, "percent"),
  "margin-right": unit(10, "percent"),
}}), {{
  marginLeft: "20px",
  marginRight: "10px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}}, {{
  left: {{ unit: "percent", value: 0.2 }},
  right: {{ unit: "percent", value: 0.1 }},
}});

assertMargin("stylesheet shorthand", elementFor(".shorthand", styleDeclaration([]), {{
  "margin-left": unit(20, "percent"),
  "margin-right": unit(10, "percent"),
  "margin-top": unit(5, "percent"),
  "margin-bottom": unit(15, "percent"),
}}), {{
  marginLeft: "20px",
  marginRight: "10px",
  marginTop: "5px",
  marginBottom: "15px",
  direction: "ltr",
}}, {{
  left: {{ unit: "percent", value: 0.2 }},
  right: {{ unit: "percent", value: 0.1 }},
  top: {{ unit: "percent", value: 0.05 }},
  bottom: {{ unit: "percent", value: 0.15 }},
}});

assertMargin("stylesheet margin-inline", elementFor(".logical", styleDeclaration([]), {{
  "margin-left": unit(20, "percent"),
  "margin-right": unit(10, "percent"),
}}), {{
  marginLeft: "20px",
  marginRight: "10px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}}, {{
  left: {{ unit: "percent", value: 0.2 }},
  right: {{ unit: "percent", value: 0.1 }},
}});

assertMargin("inline beats stylesheet", elementFor(".inline", styleDeclaration([
  ["margin-left", "20%"],
]), {{
  "margin-left": unit(20, "percent"),
}}), {{
  marginLeft: "20px",
  marginRight: "0px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}}, {{
  left: {{ unit: "percent", value: 0.2 }},
}});

assertMargin("defeated stylesheet percent falls back to computed", elementFor(["#specific", ".specific"], styleDeclaration([]), {{
  "margin-left": unit(10, "px"),
}}), {{
  marginLeft: "10px",
  marginRight: "0px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}}, {{
  left: {{ unit: "px", value: 10 }},
}});
"##
        );
        fs::write(&script_path, script).expect("script");

        let output = Command::new("node")
            .arg(&script_path)
            .output()
            .expect("node should run percentage margin capture smoke test");

        assert!(
            output.status.success(),
            "node percentage margin capture smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn bundled_helper_preserves_calc_dimension_values() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-calc-dimension-capture-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        let script_path = root.join("calc-dimension-capture.js");
        let script = format!(
            r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{ styleSheets: [] }};
function getComputedStyle() {{
  throw new Error("unexpected getComputedStyle call");
}}

{TEST_HELPER_SOURCE}

function assertEqual(name, actual, expected) {{
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {{
    throw new Error(`${{name}} expected ${{expectedJson}} but got ${{actualJson}}`);
  }}
}}

// Live Chrome 149 probe: computedStyleMap().get("width") for
// width: calc(50% + 20px) returned CSSMathSum, operator "sum",
// values [CSSUnitValue percent 50, CSSUnitValue px 20], and toString()
// reconstructed "calc(50% + 20px)". margin-left subtraction also returned
// CSSMathSum with the negative px term represented by CSSMathNegate.
assertEqual("typed om calc", parseDimension({{
  toString() {{ return "calc(50% + 20px)"; }},
}}), {{ unit: "calc", value: "calc(50% + 20px)" }});

assertEqual(
  "inline authored calc fallback",
  parseDimension("calc(10% - 4px)"),
  {{ unit: "calc", value: "calc(10% - 4px)" }}
);
"##
        );
        fs::write(&script_path, script).expect("script");

        let output = Command::new("node")
            .arg(&script_path)
            .output()
            .expect("node should run calc dimension capture smoke test");

        assert!(
            output.status.success(),
            "node calc dimension capture smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn bundled_helper_preserves_calc_grid_tracks() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-calc-grid-track-capture-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        let script_path = root.join("calc-grid-track-capture.js");
        let script = format!(
            r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{ styleSheets: [] }};

{TEST_HELPER_SOURCE}

const actual = JSON.stringify(parseGridTrackDefinitions("calc(25% + 20px) 80px"));
const expected = JSON.stringify([
  {{ kind: "scalar", unit: "calc", value: "calc(25% + 20px)" }},
  {{ kind: "scalar", unit: "px", value: 80 }},
]);
if (actual !== expected) {{
  throw new Error(`unexpected calc grid tracks ${{actual}}`);
}}
"##
        );
        fs::write(&script_path, script).expect("script");

        let output = Command::new("node")
            .arg(&script_path)
            .output()
            .expect("node should run calc grid track capture smoke test");

        assert!(
            output.status.success(),
            "node calc grid track capture smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn bundled_helper_preserves_single_calc_gap_shorthand() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-calc-gap-capture-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        let script_path = root.join("calc-gap-capture.js");
        let script = format!(
            r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{ styleSheets: [] }};

{TEST_HELPER_SOURCE}

const actual = JSON.stringify(parseGaps((property) => {{
  if (property === "gap") return {{ unit: "calc", value: "calc(5% + 2px)" }};
  return "";
}}));
const expected = JSON.stringify({{
  row: {{ unit: "calc", value: "calc(5% + 2px)" }},
  column: {{ unit: "calc", value: "calc(5% + 2px)" }},
}});
if (actual !== expected) {{
  throw new Error(`unexpected calc gap ${{actual}}`);
}}
"##
        );
        fs::write(&script_path, script).expect("script");

        let output = Command::new("node")
            .arg(&script_path)
            .output()
            .expect("node should run calc gap capture smoke test");

        assert!(
            output.status.success(),
            "node calc gap capture smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn bundled_helper_preserves_inline_shorthand_calc_margin_with_typed_om() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-inline-calc-margin-capture-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        let script_path = root.join("inline-calc-margin-capture.js");
        let script = format!(
            r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
const document = {{ styleSheets: [] }};
function styleDeclaration(entries) {{
  const declaration = {{
    ...Object.fromEntries(entries.flatMap(([property, value]) => [
      [property, value],
      [property.replace(/-([a-z])/g, (_, c) => c.toUpperCase()), value],
    ])),
    length: entries.length,
    getPropertyValue(property) {{
      const entry = entries.find(([name]) => name === property);
      return entry ? entry[1] : "";
    }},
  }};
  entries.forEach(([property], index) => {{
    declaration[index] = property;
  }});
  return declaration;
}}

{TEST_HELPER_SOURCE}

const element = {{
  classList: {{ contains() {{ return false; }} }},
  matches() {{ return false; }},
  style: styleDeclaration([["margin", "calc(10% - 4px) 0px"]]),
  computedStyleMap() {{
    return {{
      get(property) {{
        if (property === "margin-top") return {{ toString() {{ return "calc(10% - 4px)"; }} }};
        return {{ unit: "px", value: 0 }};
      }},
    }};
  }},
}};
const margin = parseEffectiveMargin(element, {{
  marginLeft: "0px",
  marginRight: "0px",
  marginTop: "16px",
  marginBottom: "0px",
  direction: "ltr",
}});
const actual = JSON.stringify(margin);
const expected = JSON.stringify({{
  top: {{ unit: "calc", value: "calc(10% - 4px)" }},
}});
if (actual !== expected) {{
  throw new Error(`unexpected inline shorthand calc margin ${{actual}}`);
}}
"##
        );
        fs::write(&script_path, script).expect("script");

        let output = Command::new("node")
            .arg(&script_path)
            .output()
            .expect("node should run inline calc margin capture smoke test");

        assert!(
            output.status.success(),
            "node inline calc margin capture smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn bundled_helper_does_not_preserve_stylesheet_scanned_calc_lengths() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-stylesheet-calc-capture-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        let script_path = root.join("stylesheet-calc-capture.js");
        let script = format!(
            r##"
const window = {{ innerWidth: 800 }};
const Node = {{ ELEMENT_NODE: 1, TEXT_NODE: 3 }};
const CSSRule = {{ STYLE_RULE: 1 }};
function styleDeclaration(entries) {{
  const declaration = {{
    ...Object.fromEntries(entries.flatMap(([property, value]) => [
      [property, value],
      [property.replace(/-([a-z])/g, (_, c) => c.toUpperCase()), value],
    ])),
    length: entries.length,
    getPropertyValue(property) {{
      const entry = entries.find(([name]) => name === property);
      return entry ? entry[1] : "";
    }},
  }};
  entries.forEach(([property], index) => {{
    declaration[index] = property;
  }});
  return declaration;
}}
const document = {{
  styleSheets: [
    {{
      cssRules: [
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".stylesheet-calc",
          style: styleDeclaration([
            ["width", "calc(50% + 20px)"],
            ["height", "10px"],
            ["grid-template-columns", "calc(25% + 20px) 80px"],
          ]),
        }},
      ],
    }},
  ],
  createElement() {{
    return {{
      style: {{}},
      offsetWidth: 0,
      clientWidth: 0,
      remove() {{}},
    }};
  }},
  body: {{ appendChild() {{}} }},
}};

{TEST_HELPER_SOURCE}

const parent = {{
  getBoundingClientRect() {{ return {{ x: 0, y: 0, width: 200, height: 10, right: 200, left: 0, bottom: 10, top: 0 }}; }},
  classList: {{ contains() {{ return false; }} }},
  clientLeft: 0,
  clientTop: 0,
}};
const element = {{
  tagName: "DIV",
  classList: {{ contains() {{ return false; }} }},
  matches(selector) {{ return selector === ".stylesheet-calc"; }},
  style: styleDeclaration([]),
  computedStyleMap() {{
    return {{
      get(property) {{
        if (property === "width") return {{ toString() {{ return "calc(50% + 20px)"; }} }};
        if (property === "height") return {{ unit: "px", value: 10 }};
        if (property === "grid-template-columns") return {{
          toString() {{ return "calc(25% + 20px) 80px"; }},
        }};
        return undefined;
      }},
    }};
  }},
  parentNode: parent,
  childNodes: [],
  childElementCount: 0,
  textContent: "",
  getBoundingClientRect() {{ return {{ x: 0, y: 0, width: 120, height: 10, right: 120, left: 0, bottom: 10, top: 0 }}; }},
  scrollWidth: 120,
  scrollHeight: 10,
  clientWidth: 120,
  clientHeight: 10,
  offsetWidth: 120,
  offsetHeight: 10,
  offsetLeft: 0,
  offsetTop: 0,
  getAttribute() {{ return null; }},
}};
function getComputedStyle() {{
  return {{
    display: "block",
    boxSizing: "border-box",
    direction: "ltr",
    writingMode: "horizontal-tb",
    fontFamily: "ahem",
    fontSize: "10px",
    lineHeight: "10px",
    width: "120px",
    height: "10px",
    minWidth: "0px",
    minHeight: "0px",
    maxWidth: "none",
    maxHeight: "none",
    marginLeft: "0px",
    marginRight: "0px",
    marginTop: "0px",
    marginBottom: "0px",
  }};
}}

const data = describeElement(element, {{}});
const actual = JSON.stringify(data.style.size);
const expected = JSON.stringify({{ height: {{ unit: "px", value: 10 }} }});
if (actual !== expected) {{
  throw new Error(`stylesheet calc should not be preserved; got ${{actual}}`);
}}
if (data.style.gridTemplateColumns !== undefined) {{
  throw new Error(`stylesheet compound calc should not be preserved; got ${{JSON.stringify(data.style.gridTemplateColumns)}}`);
}}
"##
        );
        fs::write(&script_path, script).expect("script");

        let output = Command::new("node")
            .arg(&script_path)
            .output()
            .expect("node should run stylesheet calc capture smoke test");

        assert!(
            output.status.success(),
            "node stylesheet calc capture smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn bundled_helper_does_not_guess_stylesheet_auto_margin_when_specificity_defeats_it() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-defeated-auto-margin-capture-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        let script_path = root.join("defeated-auto-margin-capture.js");
        let script = format!(
            r##"
const window = {{}};
const CSSRule = {{ STYLE_RULE: 1 }};
function styleDeclaration(entries) {{
  const declaration = {{
    ...Object.fromEntries(entries.map(([property, value]) => [property.replace(/-([a-z])/g, (_, c) => c.toUpperCase()), value])),
    ...Object.fromEntries(entries.map(([property, value]) => [property, value])),
    length: entries.length,
    getPropertyValue(property) {{
      const entry = entries.find(([name]) => name === property);
      return entry ? entry[1] : "";
    }},
  }};
  entries.forEach(([property], index) => {{
    declaration[index] = property;
  }});
  return declaration;
}}
const document = {{
  styleSheets: [
    {{
      cssRules: [
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: "#item",
          style: styleDeclaration([["margin-left", "10px"]]),
        }},
        {{
          type: CSSRule.STYLE_RULE,
          selectorText: ".item",
          style: styleDeclaration([["margin-left", "auto"]]),
        }},
      ],
    }},
  ],
}};
function getComputedStyle() {{
  throw new Error("unexpected getComputedStyle call");
}}

{TEST_HELPER_SOURCE}

const element = {{
  classList: {{ contains() {{ return false; }} }},
  matches(selector) {{ return selector === "#item" || selector === ".item"; }},
  style: {{
    length: 0,
    getPropertyValue() {{ return ""; }},
  }},
}};
const margin = parseEffectiveMargin(element, {{
  marginLeft: "10px",
  marginRight: "0px",
  marginTop: "0px",
  marginBottom: "0px",
  direction: "ltr",
}});
const actual = JSON.stringify(margin);
const expected = JSON.stringify({{ left: {{ unit: "px", value: 10 }} }});
if (actual !== expected) {{
  throw new Error(`unexpected effective margin ${{actual}}`);
}}
"##
        );
        fs::write(&script_path, script).expect("script");

        let output = Command::new("node")
            .arg(&script_path)
            .output()
            .expect("node should run defeated auto margin capture smoke test");

        assert!(
            output.status.success(),
            "node defeated auto margin capture smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn generator_injected_capture_records_grid_template_areas() {
        assert!(GRID_TEMPLATE_AREA_CAPTURE_SCRIPT.contains("gridTemplateAreas"));
        assert!(GRID_TEMPLATE_AREA_CAPTURE_SCRIPT.contains("originalDescribeElement"));
        assert!(GRID_TEMPLATE_AREA_CAPTURE_SCRIPT.contains("authoredStyleValue"));
        assert!(GRID_TEMPLATE_AREA_CAPTURE_SCRIPT.contains("computedStyle.gridTemplateAreas"));
    }

    #[test]
    fn generator_injected_capture_reads_local_authored_grid_template_areas() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-grid-template-area-capture-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        let script_path = root.join("grid-template-area-capture.js");
        let script = format!(
            r#"
const window = {{}};
const element = {{ style: {{ gridTemplateAreas: "" }} }};
function getComputedStyle() {{
  return {{ gridTemplateAreas: "none" }};
}}
function authoredStyleValue(element, property) {{
  if (property !== "gridTemplateAreas") {{
    throw new Error(`unexpected property ${{property}}`);
  }}
  return '"head head" "nav main"';
}}
function describeElement() {{
  return {{ style: {{ display: "grid" }} }};
}}

{GRID_TEMPLATE_AREA_CAPTURE_SCRIPT}

const data = describeElement(element);
const actual = JSON.stringify(data.style.gridTemplateAreas);
const expected = JSON.stringify([["head", "head"], ["nav", "main"]]);
if (actual !== expected) {{
  throw new Error(`unexpected gridTemplateAreas ${{actual}}`);
}}
"#
        );
        fs::write(&script_path, script).expect("script");

        let output = Command::new("node")
            .arg(&script_path)
            .output()
            .expect("node should run grid template area capture smoke test");

        assert!(
            output.status.success(),
            "node grid template area capture smoke failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn bundled_helper_records_default_root_viewport_as_max_content() {
        assert!(TEST_HELPER_SOURCE.contains("width: rootFillsBrowserViewport"));
        assert!(TEST_HELPER_SOURCE.contains(": { unit: 'max-content' }"));
    }

    #[test]
    fn bundled_helper_records_viewport_width_when_root_fills_browser_viewport() {
        assert!(TEST_HELPER_SOURCE.contains("rootFillsBrowserViewport"));
        assert!(TEST_HELPER_SOURCE.contains("px(window.innerWidth)"));
        assert!(
            TEST_HELPER_SOURCE.contains("Math.round(boundingRect.width) === window.innerWidth")
        );
    }

    #[test]
    fn bundled_helper_rejects_br_instead_of_lowering_to_measured_size() {
        assert!(TEST_HELPER_SOURCE.contains("tagName: e.tagName.toLowerCase()"));
        assert!(TEST_HELPER_SOURCE.contains("unsupportedElementReason"));
        assert!(TEST_HELPER_SOURCE.contains("Unsupported <br>"));
        assert!(!TEST_HELPER_SOURCE.contains("width: px(boundingRect.width)"));
        assert!(!TEST_HELPER_SOURCE.contains("height: px(boundingRect.height)"));
    }

    #[test]
    fn bundled_helper_rejects_mixed_text_content_instead_of_synthesizing_margins() {
        assert!(TEST_HELPER_SOURCE.contains("unsupportedChildNodesReason"));
        assert!(TEST_HELPER_SOURCE.contains("Unsupported mixed text/element content"));
        assert!(TEST_HELPER_SOURCE.contains("isSignificantInlineWhitespace"));
        assert!(!TEST_HELPER_SOURCE.contains("addLeadingInlineWhitespaceMargin"));
        assert!(!TEST_HELPER_SOURCE.contains("style.margin.left = { unit: 'px', value: width }"));
    }

    #[test]
    fn bundled_helper_reports_missing_test_root_as_unsupported() {
        assert!(TEST_HELPER_SOURCE.contains("unsupportedTestData"));
        assert!(TEST_HELPER_SOURCE.contains("Unsupported missing #test-root fixture root"));
    }

    #[test]
    fn unsupported_browser_semantics_are_reported_without_xml_generation() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-generate-test-{}",
            std::process::id()
        ));
        let html_root = root.join("html");
        let xml_root = root.join("xml");
        let html_dir = html_root.join("inline");
        std::fs::create_dir_all(&html_dir).expect("temp html dir should be created");
        let fixture = html_dir.join("mixed.html");
        std::fs::write(&fixture, "").expect("fixture placeholder should be written");

        let config = Config {
            root,
            html_root: html_root.clone(),
            xml_root: xml_root.clone(),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };
        let desc = json!({
            "borderBoxLtrData": unsupported_node("Unsupported <br> line-break semantics"),
            "contentBoxLtrData": unsupported_node("Unsupported mixed text/element content"),
            "borderBoxRtlData": unsupported_node("Unsupported <br> line-break semantics"),
            "contentBoxRtlData": unsupported_node("Unsupported mixed text/element content")
        });

        let mut report = GenerationReport::default();
        write_fixture_goldens(
            &config,
            Path::new("/tmp/chrome"),
            &fixture,
            &desc,
            &mut report,
        )
        .expect("unsupported cases should be reported without XML");

        assert!(
            !xml_root.join("inline/mixed__border_box_ltr.xml").exists(),
            "unsupported fixture variants must not enter checked XML parity"
        );
        assert_eq!(report.unsupported.len(), 4);

        std::fs::remove_dir_all(config.root).ok();
    }

    #[test]
    fn xml_generation_applies_root_rounding_policy_to_descendants() {
        let node = json!({
            "useRounding": false,
            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
            "style": {"direction": "ltr"},
            "smartRoundedLayout": {"x": 0, "y": 0, "width": 200, "height": 42, "scrollWidth": 200, "scrollHeight": 42},
            "unroundedLayout": {"x": 0, "y": 0, "width": 200, "height": 42.15625, "scrollWidth": 200, "scrollHeight": 42},
            "naivelyRoundedLayout": {"clientWidth": 200, "clientHeight": 42},
            "children": [
                {
                    "useRounding": true,
                    "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
                    "style": {"direction": "ltr"},
                    "smartRoundedLayout": {"x": 8, "y": 8, "width": 97, "height": 26, "scrollWidth": 97, "scrollHeight": 26},
                    "unroundedLayout": {"x": 8, "y": 8, "width": 97, "height": 26.15625, "scrollWidth": 97, "scrollHeight": 26},
                    "naivelyRoundedLayout": {"clientWidth": 97, "clientHeight": 26},
                    "children": [
                        {
                            "useRounding": true,
                            "viewport": {"width": {"unit": "max-content"}, "height": {"unit": "max-content"}},
                            "style": {"direction": "ltr"},
                            "smartRoundedLayout": {"x": 10, "y": 10, "width": 38, "height": 6, "scrollWidth": 38, "scrollHeight": 6},
                            "unroundedLayout": {"x": 10.078125, "y": 10.078125, "width": 38.40625, "height": 6, "scrollWidth": 38, "scrollHeight": 6},
                            "naivelyRoundedLayout": {"clientWidth": 38, "clientHeight": 6},
                            "children": []
                        }
                    ]
                }
            ]
        });

        let xml = generate_xml("percentage_moderate_complexity__border_box_ltr", &node);

        assert!(xml.contains(
            "<test name=\"percentage_moderate_complexity__border_box_ltr\" use-rounding=\"false\">"
        ));
        assert!(xml.contains("    <node x=\"0\" y=\"0\" width=\"200\" height=\"42.15625\">"));
        assert!(xml.contains("      <node x=\"8\" y=\"8\" width=\"97\" height=\"26.15625\">"));
        assert!(xml.contains(
            "        <node x=\"10.078125\" y=\"10.078125\" width=\"38.40625\" height=\"6\"/>"
        ));
    }

    #[test]
    fn layout_xml_attrs_match_engine_scalar_formatting() {
        assert_eq!(layout_number_attr_value(137.203125), "137.20313");
        assert_eq!(layout_number_attr_value(42.15625), "42.15625");
        assert_eq!(layout_number_attr_value(10.0), "10");
        assert_eq!(number_attr_value(55.00000000000001), "55.00000000000001");
    }

    #[test]
    fn browser_cache_defaults_to_project_target_not_system_cache() {
        let config = Config::from_root(PathBuf::from(DEFAULT_ROOT))
            .expect("browser parity root should resolve");

        assert!(config.browser_cache.ends_with("target/surgeist-browser"));
        assert!(config.root.is_relative());
        assert!(
            config
                .html_root
                .ends_with("tests/layout/browser_parity/html")
        );
        assert!(config.xml_root.ends_with("tests/layout/browser_parity/xml"));
        assert!(config.browser_cache.is_relative());
        assert!(config.browser_path.is_none());
        assert_eq!(
            config.browser_version.as_deref(),
            Some(DEFAULT_BROWSER_VERSION)
        );
    }

    #[test]
    fn browser_fixture_batch_size_keeps_renderer_lifetime_bounded() {
        const {
            assert!(BROWSER_FIXTURE_BATCH_SIZE > 0);
            assert!(BROWSER_FIXTURE_BATCH_SIZE <= 50);
        }
    }

    #[test]
    fn retryable_generation_error_only_matches_browser_navigation_timeouts() {
        assert!(is_retryable_generation_error(
            "failed to open html/flex/example.html: Request timed out."
        ));
        assert!(is_retryable_generation_error(
            "failed to reset browser page for html/flex/example.html: Request timed out."
        ));
        assert!(is_retryable_generation_error(
            "failed to load html/flex/example.html into browser page: Request timed out."
        ));
        assert!(is_retryable_generation_error(
            "failed to open html/flex/example.html: timed out waiting for DOM readiness"
        ));
        assert!(!is_retryable_generation_error(
            "Unsupported mixed text/element content"
        ));
        assert!(!is_retryable_generation_error(
            "failed to measure html/flex/example.html: Request timed out."
        ));
    }

    #[test]
    fn browser_document_script_writes_json_escaped_html() {
        let script = browser_document_write_script("<!doctype html><p>quote\"</p>");

        assert!(script.contains("document.write"));
        assert!(script.contains(r#"quote\""#));
        assert!(!script.contains("Page.navigate"));
    }

    #[test]
    fn browser_fixture_document_inlines_base_style_only_when_authored_fixture_references_it() {
        let raw_without_style =
            "<!doctype html><html><head></head><body><div id=\"test-root\"></div></body></html>";
        let raw_with_style = r#"<!doctype html><html><head><link rel="stylesheet" href="../../scripts/gentest/test_base_style.css"></head><body><div></div></body></html>"#;
        let with_style = browser_fixture_document(raw_with_style, "file:///tmp/fixtures/flex/")
            .expect("fixture document");
        let without_style =
            browser_fixture_document(raw_without_style, "file:///tmp/fixtures/html/")
                .expect("fixture document");

        assert!(with_style.contains("<base href=\"file:///tmp/fixtures/flex/\">"));
        assert!(with_style.contains("#test-root"));
        assert!(without_style.contains("<base href=\"file:///tmp/fixtures/html/\">"));
        assert!(!without_style.contains("#test-root"));
    }

    #[test]
    fn browser_fixture_document_preserves_doctype_when_synthesizing_head() {
        let raw = "<!doctype html><div id=\"test-root\"></div>";
        let document =
            browser_fixture_document(raw, "file:///tmp/fixtures/html/").expect("fixture document");

        assert!(document.starts_with("<!doctype html><html><head>"));
        assert!(document.contains("<base href=\"file:///tmp/fixtures/html/\">"));
        assert!(document.contains("</head><body><div id=\"test-root\"></div></body></html>"));
    }

    #[test]
    fn browser_fixture_document_inserts_head_inside_no_head_html_document() {
        let raw = "<!doctype html><html><body><div id=\"test-root\"></div></body></html>";
        let document =
            browser_fixture_document(raw, "file:///tmp/fixtures/html/").expect("fixture document");

        assert!(document.starts_with("<!doctype html><html><head>"));
        assert!(document.contains("</head><body><div id=\"test-root\"></div></body></html>"));
    }

    #[test]
    fn browser_args_disable_background_browser_behavior() {
        let args = browser_args();

        assert!(args.contains(&"disable-background-networking"));
        assert!(args.contains(&"disable-component-update"));
        assert!(args.contains(&"disable-sync"));
        assert!(args.contains(&"no-first-run"));
    }

    #[test]
    fn browser_args_enable_grid_lanes_layout() {
        let args = browser_args();

        assert!(args.contains(&"enable-blink-features=IdleDetection,CSSGridLanesLayout"));
    }

    #[test]
    fn browser_args_prevent_macos_keychain_prompts() {
        let args = browser_args();

        assert!(args.contains(&"use-mock-keychain"));
    }

    #[test]
    fn browser_args_keep_scrollbars_visible_for_browser_parity() {
        let args = browser_args();

        assert!(args.contains(&"headless=new"));
        assert!(!args.iter().any(|arg| arg.contains("hide-scrollbars")));
        assert!(!args.iter().any(|arg| arg.starts_with("--")));
    }

    #[test]
    fn input_attrs_omits_horizontal_writing_mode_under_horizontal_parent() {
        let attrs = input_attrs(&json!({
            "style": {
                "writingMode": "horizontal-tb"
            }
        }));

        assert!(!attrs.iter().any(|(key, _)| *key == "writing-mode"));
    }

    #[test]
    fn input_attrs_emits_horizontal_writing_mode_to_reset_vertical_parent() {
        let attrs = input_attrs_with_parent_writing_mode(
            &json!({
                "style": {
                    "writingMode": "horizontal-tb"
                }
            }),
            "vertical-rl",
        );

        assert!(attrs.contains(&("writing-mode", "horizontal-tb".to_string())));
    }

    #[test]
    fn input_attrs_emits_computed_vertical_writing_mode() {
        let attrs = input_attrs(&json!({
            "style": {
                "writingMode": "vertical-rl"
            }
        }));

        assert!(attrs.contains(&("writing-mode", "vertical-rl".to_string())));
    }

    #[test]
    fn visible_overflow_does_not_emit_scrollbar_width() {
        let attrs = input_attrs(&json!({
            "style": {
                "direction": "ltr",
                "overflowX": "visible",
                "overflowY": "visible",
                "scrollbarWidth": 15,
                "size": {"width": {"unit": "auto"}, "height": {"unit": "auto"}},
                "minSize": {"width": {"unit": "auto"}, "height": {"unit": "auto"}},
                "maxSize": {"width": {"unit": "auto"}, "height": {"unit": "auto"}},
                "margin": {},
                "padding": {},
                "border": {},
                "inset": {},
                "gap": {},
                "gridAutoFlow": {},
                "gridTemplateRows": [],
                "gridTemplateColumns": [],
                "gridAutoRows": [],
                "gridAutoColumns": [],
                "gridRowStart": {"kind": "auto"},
                "gridRowEnd": {"kind": "auto"},
                "gridColumnStart": {"kind": "auto"},
                "gridColumnEnd": {"kind": "auto"}
            }
        }));

        assert!(!attrs.iter().any(|(key, _)| *key == "scrollbar-width"));
    }

    #[test]
    fn input_attrs_serializes_logical_inline_margins_as_physical_edges() {
        let attrs = input_attrs(&json!({
            "style": {
                "direction": "ltr",
                "overflowX": "visible",
                "overflowY": "visible",
                "size": {"width": {"unit": "auto"}, "height": {"unit": "auto"}},
                "minSize": {"width": {"unit": "auto"}, "height": {"unit": "auto"}},
                "maxSize": {"width": {"unit": "auto"}, "height": {"unit": "auto"}},
                "margin": {},
                "logicalMargin": {
                    "inlineStart": {"unit": "auto"},
                    "inlineEnd": {"unit": "px", "value": 12}
                },
                "padding": {},
                "border": {},
                "inset": {},
                "gap": {},
                "gridAutoFlow": {},
                "gridTemplateRows": [],
                "gridTemplateColumns": [],
                "gridAutoRows": [],
                "gridAutoColumns": [],
                "gridRowStart": {"kind": "auto"},
                "gridRowEnd": {"kind": "auto"},
                "gridColumnStart": {"kind": "auto"},
                "gridColumnEnd": {"kind": "auto"}
            }
        }));

        assert!(attrs.contains(&("margin-left", "auto".to_string())));
        assert!(attrs.contains(&("margin-right", "12px".to_string())));
    }

    #[test]
    fn input_attrs_lets_logical_margin_override_conflicting_physical_edge() {
        let attrs = input_attrs(&json!({
            "style": {
                "direction": "ltr",
                "overflowX": "visible",
                "overflowY": "visible",
                "size": {"width": {"unit": "auto"}, "height": {"unit": "auto"}},
                "minSize": {"width": {"unit": "auto"}, "height": {"unit": "auto"}},
                "maxSize": {"width": {"unit": "auto"}, "height": {"unit": "auto"}},
                "margin": {"left": {"unit": "px", "value": 3}},
                "logicalMargin": {
                    "inlineStart": {"unit": "px", "value": 9}
                },
                "padding": {},
                "border": {},
                "inset": {},
                "gap": {},
                "gridAutoFlow": {},
                "gridTemplateRows": [],
                "gridTemplateColumns": [],
                "gridAutoRows": [],
                "gridAutoColumns": [],
                "gridRowStart": {"kind": "auto"},
                "gridRowEnd": {"kind": "auto"},
                "gridColumnStart": {"kind": "auto"},
                "gridColumnEnd": {"kind": "auto"}
            }
        }));

        assert_eq!(
            attrs
                .iter()
                .filter(|(key, _)| *key == "margin-left")
                .collect::<Vec<_>>(),
            vec![&("margin-left", "9px".to_string())]
        );
    }

    #[test]
    fn fixture_url_accepts_relative_paths_for_browser_navigation() {
        let url = fixture_url(Path::new(
            "tests/layout/browser_parity/html/block/block_basic.html",
        ))
        .expect("relative fixture should become file URL");

        assert_eq!(url.scheme(), "file");
        assert!(
            url.as_str()
                .ends_with("tests/layout/browser_parity/html/block/block_basic.html")
        );
    }

    #[test]
    fn collect_html_filter_matches_fixture_folder() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-filter-folder-{}",
            std::process::id()
        ));
        let html_root = root.join("html");
        fs::create_dir_all(html_root.join("subgrid")).expect("subgrid dir should exist");
        fs::create_dir_all(html_root.join("grid")).expect("grid dir should exist");
        fs::write(html_root.join("subgrid/baseline.html"), "").expect("subgrid fixture");
        fs::write(html_root.join("grid/basic.html"), "").expect("grid fixture");

        let files = collect_html(&html_root, Some("subgrid")).expect("filtered fixtures");

        assert_eq!(files, [html_root.join("subgrid/baseline.html")]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn collect_html_filter_matches_single_fixture_without_extension() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-filter-file-{}",
            std::process::id()
        ));
        let html_root = root.join("html");
        fs::create_dir_all(html_root.join("grid-lanes")).expect("grid-lanes dir should exist");
        fs::write(html_root.join("grid-lanes/rows.html"), "").expect("rows fixture");
        fs::write(html_root.join("grid-lanes/columns.html"), "").expect("columns fixture");

        let files = collect_html(&html_root, Some("grid-lanes/rows")).expect("filtered fixture");

        assert_eq!(files, [html_root.join("grid-lanes/rows.html")]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn collect_html_filter_matches_fixture_stem_prefix_within_folder() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-filter-prefix-{}",
            std::process::id()
        ));
        let html_root = root.join("html");
        fs::create_dir_all(html_root.join("subgrid")).expect("subgrid dir should exist");
        fs::write(html_root.join("subgrid/subgrid_abspos_ltr.html"), "").expect("abspos fixture");
        fs::write(html_root.join("subgrid/subgrid_alignment_ltr.html"), "")
            .expect("alignment fixture");

        let files =
            collect_html(&html_root, Some("subgrid/subgrid_abspos")).expect("filtered fixtures");

        assert_eq!(files, [html_root.join("subgrid/subgrid_abspos_ltr.html")]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn collect_html_includes_x_prefixed_fixtures() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-filter-x-prefix-{}",
            std::process::id()
        ));
        let html_root = root.join("html");
        fs::create_dir_all(html_root.join("float")).expect("float dir should exist");
        fs::write(html_root.join("float/xfloat_min_content.html"), "").expect("x fixture");

        let files = collect_html(&html_root, Some("float")).expect("filtered fixtures");

        assert_eq!(files, [html_root.join("float/xfloat_min_content.html")]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn output_paths_for_fixture_include_all_variants() {
        let paths = output_paths_for_fixture(
            Path::new("html"),
            Path::new("xml"),
            Path::new("html/grid/basic.html"),
        )
        .expect("output paths should be computed");

        assert_eq!(
            paths,
            [
                PathBuf::from("xml/grid/basic__border_box_ltr.xml"),
                PathBuf::from("xml/grid/basic__content_box_ltr.xml"),
                PathBuf::from("xml/grid/basic__border_box_rtl.xml"),
                PathBuf::from("xml/grid/basic__content_box_rtl.xml"),
            ]
        );
    }

    #[test]
    fn import_taffy_clears_stale_managed_files_and_preserves_surgeist_owned_fixtures() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-taffy-import-{}",
            std::process::id()
        ));
        let source_root = root.join("source");
        let source_fixtures = source_root.join(TAFFY_SOURCE_DIR);
        let corpus_root = root.join("corpus");
        let html_root = corpus_root.join("html");
        fs::create_dir_all(source_fixtures.join("block")).expect("source block dir");
        fs::create_dir_all(source_fixtures.join("flex")).expect("source flex dir");
        fs::create_dir_all(source_fixtures.join("subgrid")).expect("source subgrid dir");
        fs::create_dir_all(html_root.join("block")).expect("target block dir");
        fs::create_dir_all(html_root.join("grid")).expect("target grid dir");
        fs::create_dir_all(html_root.join("subgrid")).expect("target subgrid dir");
        fs::write(
            corpus_root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]

[[cases]]
id = "grid/grid_named_repeated_line_names"
source_root = "surgeist"
source = "grid/grid_named_repeated_line_names.html"
generator = "constrained-html"
status = "active"

[[cases]]
id = "subgrid/local"
source_root = "surgeist"
source = "subgrid/local.html"
generator = "constrained-html"
status = "active"
"#
            ),
        )
        .expect("corpus manifest");
        fs::write(
            source_fixtures.join("block/basic.html"),
            "<!doctype html><p>basic</p>",
        )
        .expect("source basic");
        fs::write(
            source_fixtures.join("flex/keep.html"),
            "<!doctype html><p>keep</p>",
        )
        .expect("source keep");
        fs::write(
            source_fixtures.join("subgrid/upstream.html"),
            "<!doctype html><p>excluded source</p>",
        )
        .expect("source excluded");
        fs::write(
            html_root.join("block/stale.html"),
            "<!doctype html><p>stale</p>",
        )
        .expect("stale");
        fs::write(
            html_root.join("grid/grid_named_repeated_line_names.html"),
            "<!doctype html><p>surgeist extra</p>",
        )
        .expect("surgeist extra");
        fs::write(
            html_root.join("subgrid/local.html"),
            "<!doctype html><p>local subgrid</p>",
        )
        .expect("local subgrid");
        let config = Config {
            root: corpus_root,
            html_root: html_root.clone(),
            xml_root: root.join("corpus/xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };

        let plan =
            prepare_taffy_import_with_expected_count(&source_root, 3).expect("Taffy import plan");
        let expected_files = plan
            .fixtures
            .iter()
            .map(|(rel, _)| rel.clone())
            .collect::<BTreeSet<_>>();
        clear_taffy_import_outputs(&config, &expected_files).expect("clear stale Taffy files");
        for (rel, raw) in plan.fixtures {
            let target = html_root.join(rel);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).expect("target parent");
            }
            fs::write(target, raw).expect("write fixture");
        }

        assert!(
            !html_root.join("block/stale.html").exists(),
            "stale managed baseline fixture should be removed"
        );
        assert_eq!(
            fs::read_to_string(html_root.join("block/basic.html")).expect("basic"),
            "<!doctype html><p>basic</p>"
        );
        assert_eq!(
            fs::read_to_string(html_root.join("flex/keep.html")).expect("keep"),
            "<!doctype html><p>keep</p>"
        );
        assert_eq!(
            fs::read_to_string(html_root.join("grid/grid_named_repeated_line_names.html"))
                .expect("surgeist extra"),
            "<!doctype html><p>surgeist extra</p>"
        );
        assert_eq!(
            fs::read_to_string(html_root.join("subgrid/local.html")).expect("local subgrid"),
            "<!doctype html><p>local subgrid</p>"
        );
        assert!(
            !html_root.join("subgrid/upstream.html").exists(),
            "excluded Taffy domains should not be imported over Surgeist-owned suites"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn taffy_manifest_rejects_legacy_allowed_extra_paths() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-taffy-allowed-extra-{}",
            std::process::id()
        ));
        let corpus_root = root.join("corpus");
        let html_root = corpus_root.join("html");
        fs::create_dir_all(html_root.join("flex")).expect("flex dir");
        fs::write(
            corpus_root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
allowed_extra_paths = [
  "flex/local_extra.html",
]
excluded_destination_dirs = ["subgrid", "grid-lanes"]
"#
            ),
        )
        .expect("corpus manifest");
        fs::write(
            html_root.join("flex/local_extra.html"),
            "<!doctype html><p>local extra</p>",
        )
        .expect("local extra");
        fs::write(
            html_root.join("flex/stale.html"),
            "<!doctype html><p>stale</p>",
        )
        .expect("stale");
        let config = Config {
            root: corpus_root,
            html_root: html_root.clone(),
            xml_root: root.join("corpus/xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };

        let error = validate_taffy_manifest(&config)
            .expect_err("legacy allowed_extra_paths should require [[cases]] migration");

        assert!(error.contains("[imports.taffy].allowed_extra_paths"));
        assert!(error.contains("[[cases]]"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn import_taffy_preserves_surgeist_constrained_manifest_cases() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-taffy-surgeist-case-{}",
            std::process::id()
        ));
        let corpus_root = root.join("corpus");
        let html_root = corpus_root.join("html");
        fs::create_dir_all(html_root.join("grid")).expect("grid dir");
        fs::write(
            corpus_root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]

[[cases]]
id = "grid/local_named_lines"
source_root = "surgeist"
source = "grid/local_named_lines.html"
generator = "constrained-html"
status = "active"
"#
            ),
        )
        .expect("corpus manifest");
        fs::write(
            html_root.join("grid/local_named_lines.html"),
            "<!doctype html><p>local case</p>",
        )
        .expect("local case");
        fs::write(
            html_root.join("grid/stale.html"),
            "<!doctype html><p>stale</p>",
        )
        .expect("stale");
        let config = Config {
            root: corpus_root,
            html_root: html_root.clone(),
            xml_root: root.join("corpus/xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };
        let expected_files = BTreeSet::new();

        clear_taffy_import_outputs(&config, &expected_files).expect("clear stale Taffy files");

        assert_eq!(
            fs::read_to_string(html_root.join("grid/local_named_lines.html")).expect("local case"),
            "<!doctype html><p>local case</p>"
        );
        assert!(
            !html_root.join("grid/stale.html").exists(),
            "unmanifested local constrained fixture should still be removed"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn check_corpus_rejects_missing_surgeist_constrained_manifest_case() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-missing-surgeist-case-{}",
            std::process::id()
        ));
        let corpus_root = root.join("corpus");
        fs::create_dir_all(corpus_root.join("html/grid")).expect("html dir");
        fs::write(
            corpus_root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]

[[cases]]
id = "grid/missing"
source_root = "surgeist"
source = "grid/missing.html"
generator = "constrained-html"
status = "active"
"#
            ),
        )
        .expect("corpus manifest");
        let config = Config {
            root: corpus_root,
            html_root: root.join("corpus/html"),
            xml_root: root.join("corpus/xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };

        let error = validate_surgeist_constrained_case_files(&config)
            .expect_err("missing manifest-owned constrained fixture should fail");

        assert!(error.contains("missing Surgeist constrained case grid/missing"));
        assert!(error.contains("grid/missing.html"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn gentest_directory_is_helper_only() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-gentest-helper-only-{}",
            std::process::id()
        ));
        let corpus_root = root.join("corpus");
        let helper_root = corpus_root.join("scripts/gentest");
        fs::create_dir_all(&helper_root).expect("helper dir");
        fs::write(helper_root.join("test_helper.js"), "").expect("test helper");
        fs::write(helper_root.join("test_base_style.css"), "").expect("base style");
        let config = Config {
            root: corpus_root,
            html_root: root.join("corpus/html"),
            xml_root: root.join("corpus/xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };

        check_gentest_helper_only_assets(&config).expect("helper-only directory should pass");

        fs::write(helper_root.join("Cargo.toml"), "").expect("legacy generator manifest");
        let error = check_gentest_helper_only_assets(&config)
            .expect_err("legacy generator files should fail");

        assert!(error.contains("scripts/gentest"));
        assert!(error.contains("Cargo.toml"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn taffy_cleanup_removes_unmanifested_surgeist_owned_excluded_domain_fixtures() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-taffy-excluded-unmanifested-{}",
            std::process::id()
        ));
        let corpus_root = root.join("corpus");
        let html_root = corpus_root.join("html");
        fs::create_dir_all(html_root.join("subgrid")).expect("subgrid dir");
        fs::write(
            corpus_root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]
"#
            ),
        )
        .expect("corpus manifest");
        fs::write(
            html_root.join("subgrid/unmanifested.html"),
            "<!doctype html><p>local but undeclared</p>",
        )
        .expect("local case");
        let config = Config {
            root: corpus_root,
            html_root: html_root.clone(),
            xml_root: root.join("corpus/xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };
        let expected_files = BTreeSet::new();

        clear_taffy_import_outputs(&config, &expected_files)
            .expect("clear stale local files in excluded domains");

        assert!(
            !html_root.join("subgrid/unmanifested.html").exists(),
            "excluded-domain local fixtures must still be manifest-owned"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn constrained_manifest_unsupported_cases_are_reported_and_not_generated() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-constrained-status-{}",
            std::process::id()
        ));
        let corpus_root = root.join("corpus");
        let html_root = corpus_root.join("html");
        let xml_root = corpus_root.join("xml");
        fs::create_dir_all(html_root.join("grid")).expect("html dir");
        fs::create_dir_all(xml_root.join("grid")).expect("xml dir");
        fs::write(
            corpus_root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]

[[cases]]
id = "grid/local_unsupported"
source_root = "surgeist"
source = "grid/local_unsupported.html"
generator = "constrained-html"
status = "unsupported"
reason = "unsupported local fixture"
"#
            ),
        )
        .expect("corpus manifest");
        fs::write(
            html_root.join("grid/local_unsupported.html"),
            "<!doctype html><p>unsupported</p>",
        )
        .expect("unsupported fixture");
        fs::write(
            html_root.join("grid/local_active.html"),
            "<!doctype html><p>active</p>",
        )
        .expect("active fixture");
        let stale_xml = xml_root.join("grid/local_unsupported__border_box_ltr.xml");
        fs::write(&stale_xml, "<stale/>").expect("stale XML");
        let config = Config {
            root: corpus_root,
            html_root: html_root.clone(),
            xml_root,
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };
        let mut report = GenerationReport::default();

        let fixtures =
            collect_constrained_fixtures_for_generation(&config, &mut report).expect("fixtures");

        assert_eq!(fixtures, [html_root.join("grid/local_active.html")]);
        assert_eq!(report.summary.unsupported, 1);
        assert_eq!(report.unsupported[0].name, "grid/local_unsupported");
        assert_eq!(
            report.unsupported[0].source,
            "html/grid/local_unsupported.html"
        );
        assert_eq!(report.unsupported[0].variant, "manifest");
        assert_eq!(report.unsupported[0].reason, "unsupported local fixture");
        assert!(
            !stale_xml.exists(),
            "manifest-unsupported constrained fixtures should prune stale XML outputs"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn constrained_manifest_expected_fail_runs_and_quarantined_is_reported() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-constrained-statuses-{}",
            std::process::id()
        ));
        let corpus_root = root.join("corpus");
        let html_root = corpus_root.join("html");
        let xml_root = corpus_root.join("xml");
        fs::create_dir_all(html_root.join("grid")).expect("html dir");
        fs::create_dir_all(xml_root.join("grid")).expect("xml dir");
        fs::write(
            corpus_root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]

[[cases]]
id = "grid/local_expected"
source_root = "surgeist"
source = "grid/local_expected.html"
generator = "constrained-html"
status = "expected-fail"
reason = "known geometry mismatch"

[[cases]]
id = "grid/local_quarantined"
source_root = "surgeist"
source = "grid/local_quarantined.html"
generator = "constrained-html"
status = "quarantined"
reason = "unstable source fixture"
"#
            ),
        )
        .expect("corpus manifest");
        fs::write(
            html_root.join("grid/local_expected.html"),
            "<!doctype html><p>expected</p>",
        )
        .expect("expected fixture");
        fs::write(
            html_root.join("grid/local_quarantined.html"),
            "<!doctype html><p>quarantined</p>",
        )
        .expect("quarantined fixture");
        let stale_xml = xml_root.join("grid/local_quarantined__border_box_ltr.xml");
        fs::write(&stale_xml, "<stale/>").expect("stale XML");
        let config = Config {
            root: corpus_root,
            html_root: html_root.clone(),
            xml_root,
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };
        let mut report = GenerationReport::default();

        let fixtures =
            collect_constrained_fixtures_for_generation(&config, &mut report).expect("fixtures");

        assert_eq!(fixtures, [html_root.join("grid/local_expected.html")]);
        assert_eq!(report.summary.expected_fail, 1);
        assert_eq!(report.expected_fail[0].name, "grid/local_expected");
        assert_eq!(
            report.expected_fail[0].source,
            "html/grid/local_expected.html"
        );
        assert_eq!(report.expected_fail[0].reason, "known geometry mismatch");
        assert_eq!(report.summary.quarantined, 1);
        assert_eq!(report.quarantined[0].name, "grid/local_quarantined");
        assert_eq!(
            report.quarantined[0].source,
            "html/grid/local_quarantined.html"
        );
        assert_eq!(report.quarantined[0].reason, "unstable source fixture");
        assert!(
            !stale_xml.exists(),
            "manifest-quarantined constrained fixtures should prune stale XML outputs"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn taffy_manifest_rejects_unknown_and_duplicate_root_cases() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-invalid-root-cases-{}",
            std::process::id()
        ));
        let corpus_root = root.join("corpus");
        fs::create_dir_all(&corpus_root).expect("corpus dir");
        fs::write(
            corpus_root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]

[[cases]]
id = "grid/one"
source_root = "surgeits"
source = "grid/one.html"
generator = "constrained-html"
status = "active"
"#
            ),
        )
        .expect("corpus manifest");
        let config = Config {
            root: corpus_root.clone(),
            html_root: root.join("corpus/html"),
            xml_root: root.join("corpus/xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };

        let unknown_source_root =
            validate_taffy_manifest(&config).expect_err("unknown source_root should fail");

        assert!(unknown_source_root.contains("unsupported source_root `surgeits`"));

        fs::write(
            corpus_root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]

[[cases]]
id = "grid/one"
source_root = "surgeist"
source = "grid/one.html"
generator = "constrained-html"
status = "active"

[[cases]]
id = "grid/one"
source_root = "surgeist"
source = "grid/two.html"
generator = "constrained-html"
status = "active"
"#
            ),
        )
        .expect("duplicate id manifest");

        let duplicate_id =
            validate_taffy_manifest(&config).expect_err("duplicate case id should fail");

        assert!(duplicate_id.contains("duplicate root case id `grid/one`"));

        fs::write(
            corpus_root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]

[[cases]]
id = "grid/one"
source_root = "surgeist"
source = "grid/shared.html"
generator = "constrained-html"
status = "active"

[[cases]]
id = "grid/two"
source_root = "surgeist"
source = "grid/./shared.html"
generator = "constrained-html"
status = "active"
"#
            ),
        )
        .expect("duplicate source manifest");

        let duplicate_source =
            validate_taffy_manifest(&config).expect_err("duplicate case source should fail");

        assert!(duplicate_source.contains("duplicate root case source `grid/shared.html`"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn corpus_junk_check_rejects_root_ds_store() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-corpus-junk-{}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("html")).expect("html dir");
        fs::write(root.join(".DS_Store"), "").expect("junk file");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };

        let error = check_corpus_junk_files(&config).expect_err("root junk should fail");

        assert!(error.contains("unexpected non-fixture file in parity corpus"));
        assert!(error.contains(".DS_Store"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn ensure_git_source_recovers_initialized_cache_without_origin() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-source-cache-missing-origin-{}",
            std::process::id()
        ));
        let (repo, commit) = create_local_git_source(&root, "source");
        let cache = root.join("cache");
        fs::create_dir_all(&cache).expect("cache dir");
        run_git(&cache, ["init"]).expect("partial cache init");

        ensure_git_source(&cache, path_str(&repo), &commit).expect("source cache should recover");

        assert_eq!(git_head(&cache), commit);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn ensure_git_source_repairs_wrong_origin_url() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-source-cache-wrong-origin-{}",
            std::process::id()
        ));
        let (repo, commit) = create_local_git_source(&root, "source");
        let wrong_repo = root.join("wrong-source");
        fs::create_dir_all(&wrong_repo).expect("wrong source dir");
        let cache = root.join("cache");
        fs::create_dir_all(&cache).expect("cache dir");
        run_git(&cache, ["init"]).expect("partial cache init");
        run_git(&cache, ["remote", "add", "origin", path_str(&wrong_repo)]).expect("wrong origin");

        ensure_git_source(&cache, path_str(&repo), &commit)
            .expect("source cache should repair wrong origin");

        assert_eq!(git_head(&cache), commit);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn xml_generation_records_provenance_comment() {
        let node = serde_json::json!({
            "useRounding": true,
            "viewport": { "width": "max-content", "height": "max-content" },
            "style": { "display": "block" },
            "children": [],
            "smartRoundedLayout": { "x": 0, "y": 0, "width": 10, "height": 20 },
            "unroundedLayout": { "x": 0, "y": 0, "width": 10, "height": 20 },
            "naivelyRoundedLayout": { "clientWidth": 10, "clientHeight": 20 }
        });
        let provenance = GeneratedProvenance {
            source: "grid/basic.html".to_string(),
            source_sha256: "abc123".to_string(),
            linked_resources: Vec::new(),
            linked_resources_recorded: false,
            helper_sha256: "def456".to_string(),
            browser: "chrome-for-testing/149".to_string(),
        };

        let xml = generate_xml_with_provenance("basic__border_box_ltr", &node, Some(&provenance));

        assert!(xml.starts_with("<!-- generated-by: surgeist-layout-generate schema=1 "));
        assert!(xml.contains("source=\"grid/basic.html\""));
        assert!(xml.contains("source-sha256=\"abc123\""));
        assert!(xml.contains("helper-sha256=\"def456\""));
        assert!(xml.contains("browser=\"chrome-for-testing/149\""));
    }

    #[test]
    fn fixture_generation_failure_preserves_existing_outputs() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-fixture-preserve-{}",
            std::process::id()
        ));
        let html_root = root.join("html");
        let xml_root = root.join("xml");
        let fixture = html_root.join("block/example.html");
        let existing = xml_root.join("block/example__border_box_ltr.xml");
        fs::create_dir_all(fixture.parent().unwrap()).expect("fixture dir");
        fs::create_dir_all(existing.parent().unwrap()).expect("xml dir");
        fs::write(&fixture, "<!doctype html>").expect("fixture");
        fs::write(&existing, "<existing/>").expect("existing xml");
        let config = Config {
            root: root.clone(),
            html_root,
            xml_root,
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };
        let desc = serde_json::json!({
            "borderBoxLtrData": unsupported_node("planned unsupported before missing key")
        });
        let mut report = GenerationReport::default();

        let error = write_fixture_goldens(
            &config,
            Path::new("/tmp/chrome"),
            &fixture,
            &desc,
            &mut report,
        )
        .expect_err("missing fixture variants should fail");

        assert!(error.contains("measurement JSON missing contentBoxLtrData"));
        assert_eq!(
            fs::read_to_string(&existing).expect("existing xml should remain"),
            "<existing/>"
        );
        assert_eq!(report.summary.generated, 0);
        assert_eq!(report.summary.unsupported, 0);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn generation_report_records_failed_generation_attempts() {
        let mut report = GenerationReport::default();

        report.record_failed_to_generate(
            "case".to_string(),
            "html/grid/case.html".to_string(),
            "browser page crashed".to_string(),
        );

        assert!(report.has_entries());
        assert_eq!(report.summary.failed_to_generate, 1);
        assert_eq!(report.failed_to_generate.len(), 1);
        assert_eq!(report.failed_to_generate[0].name, "case");
        assert_eq!(report.failed_to_generate[0].source, "html/grid/case.html");
        assert_eq!(report.failed_to_generate[0].reason, "browser page crashed");
    }

    #[test]
    fn generation_report_metadata_validation_accepts_current_manifests() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-report-metadata-ok-{}",
            std::process::id()
        ));
        let report_dir = root.join("xml/generation-reports");
        fs::create_dir_all(&report_dir).expect("report dir");
        fs::write(
            root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]
"#
            ),
        )
        .expect("corpus manifest");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: Some(DEFAULT_BROWSER_VERSION.to_string()),
        };
        let metadata = generation_report_metadata(&config).expect("metadata");
        for (name, filter) in [("all.json", Value::Null)] {
            let raw = serde_json::json!({
                "metadata": metadata.clone(),
                "filter": filter,
                "summary": {
                    "generated": 0,
                    "unsupported": 0,
                    "expected_fail": 0,
                    "quarantined": 0,
                    "failed_to_generate": 0
                },
                "generated": [],
                "unsupported": [],
                "expected_fail": [],
                "quarantined": [],
                "failed_to_generate": []
            });
            fs::write(
                report_dir.join(name),
                serde_json::to_string_pretty(&raw).expect("json"),
            )
            .expect("report");
        }

        validate_generation_report_freshness(&config).expect("fresh reports");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn generation_report_validation_rejects_bucket_summary_drift() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-report-body-drift-{}",
            std::process::id()
        ));
        let report_dir = root.join("xml/generation-reports");
        fs::create_dir_all(&report_dir).expect("report dir");
        fs::write(
            root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]
"#
            ),
        )
        .expect("corpus manifest");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: Some(DEFAULT_BROWSER_VERSION.to_string()),
        };
        let metadata = generation_report_metadata(&config).expect("metadata");
        for (name, filter) in [("all.json", Value::Null)] {
            let raw = serde_json::json!({
                "metadata": metadata.clone(),
                "filter": filter,
                "summary": {
                    "generated": 1,
                    "unsupported": 0,
                    "expected_fail": 0,
                    "quarantined": 0,
                    "failed_to_generate": 0
                },
                "generated": [],
                "unsupported": [],
                "expected_fail": [],
                "quarantined": [],
                "failed_to_generate": []
            });
            fs::write(
                report_dir.join(name),
                serde_json::to_string_pretty(&raw).expect("json"),
            )
            .expect("report");
        }

        let error = validate_generation_report_freshness(&config)
            .expect_err("bucket summary drift should fail freshness validation");

        assert!(error.contains("generated summary is 1 but bucket has 0 entries"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn generation_report_metadata_validation_rejects_manifest_drift() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-report-metadata-drift-{}",
            std::process::id()
        ));
        let report_dir = root.join("xml/generation-reports");
        fs::create_dir_all(&report_dir).expect("report dir");
        fs::write(
            root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]
"#
            ),
        )
        .expect("corpus manifest");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: Some(DEFAULT_BROWSER_VERSION.to_string()),
        };
        let mut metadata = generation_report_metadata(&config).expect("metadata");
        metadata.corpus_manifest_sha256 = "stale".to_string();
        let raw = serde_json::json!({
            "metadata": metadata,
            "filter": null,
            "summary": {
                "generated": 0,
                "unsupported": 0,
                "expected_fail": 0,
                "quarantined": 0,
                "failed_to_generate": 0
            },
            "generated": [],
            "unsupported": [],
            "expected_fail": [],
            "quarantined": [],
            "failed_to_generate": []
        });
        fs::write(
            report_dir.join("all.json"),
            serde_json::to_string_pretty(&raw).expect("json"),
        )
        .expect("report");

        let error =
            validate_generation_report_freshness(&config).expect_err("stale metadata should fail");

        assert!(error.contains("corpus_manifest_sha256"));
        assert!(error.contains("regenerate browser parity XML"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn generation_report_metadata_validation_rejects_missing_report_directory() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-report-metadata-missing-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temp dir");
        fs::write(
            root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]
"#
            ),
        )
        .expect("corpus manifest");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: Some(DEFAULT_BROWSER_VERSION.to_string()),
        };

        let error = validate_generation_report_freshness(&config)
            .expect_err("missing report directory should fail");

        assert!(error.contains("missing generation report directory"));
        assert!(error.contains("regenerate browser parity XML"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn generation_report_metadata_validation_rejects_custom_browser_captures() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-report-metadata-custom-browser-{}",
            std::process::id()
        ));
        let report_dir = root.join("xml/generation-reports");
        fs::create_dir_all(&report_dir).expect("report dir");
        fs::write(
            root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]
"#
            ),
        )
        .expect("corpus manifest");
        let checked_in_config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: Some(DEFAULT_BROWSER_VERSION.to_string()),
        };
        let custom_browser_config = Config {
            browser_path: Some(PathBuf::from("/tmp/custom-chromium")),
            ..checked_in_config.clone()
        };
        let metadata = generation_report_metadata(&custom_browser_config).expect("metadata");
        let raw = serde_json::json!({
            "metadata": metadata,
            "filter": null,
            "summary": {
                "generated": 0,
                "unsupported": 0,
                "expected_fail": 0,
                "quarantined": 0,
                "failed_to_generate": 0
            },
            "generated": [],
            "unsupported": [],
            "expected_fail": [],
            "quarantined": [],
            "failed_to_generate": []
        });
        fs::write(
            report_dir.join("all.json"),
            serde_json::to_string_pretty(&raw).expect("json"),
        )
        .expect("report");

        let error = validate_generation_report_freshness(&checked_in_config)
            .expect_err("custom browser reports should fail freshness validation");

        assert!(error.contains("browser_source"));
        assert!(error.contains("custom-path"));
        assert!(error.contains("regenerate browser parity XML"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn failed_generation_jobs_use_corpus_relative_sources() {
        let root =
            std::env::temp_dir().join(format!("surgeist-layout-failed-job-{}", std::process::id()));
        let html_fixture = root.join("html/grid/basic.html");
        fs::create_dir_all(html_fixture.parent().unwrap()).expect("html dir");
        fs::write(&html_fixture, "<!doctype html>").expect("html source");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };
        let mut report = GenerationReport::default();

        record_failed_generation_job(
            &config,
            &GenerationJob::ConstrainedHtml(html_fixture),
            &mut report,
            "html failed".to_string(),
        );
        assert_eq!(report.summary.failed_to_generate, 1);
        assert_eq!(report.failed_to_generate[0].name, "basic");
        assert_eq!(report.failed_to_generate[0].source, "html/grid/basic.html");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn generation_report_path_is_scoped_by_filter() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-report-path-{}",
            std::process::id()
        ));
        let mut config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };

        assert_eq!(
            generation_report_path(&config),
            root.join("xml/generation-reports/all.json")
        );
        config.filter = Some("grid".to_string());
        assert_eq!(
            generation_report_path(&config),
            root.join("xml/generation-reports/grid.json")
        );
        config.filter = Some("grid/named".to_string());
        assert_eq!(
            generation_report_path(&config),
            root.join("xml/generation-reports/grid_named.json")
        );
    }

    #[test]
    fn generation_report_write_failure_preserves_existing_report() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-report-rollback-{}",
            std::process::id()
        ));
        let report_path = root.join("xml/generation-reports/all.json");
        fs::create_dir_all(report_path.parent().unwrap()).expect("report dir");
        fs::write(
            root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]
"#
            ),
        )
        .expect("corpus manifest");
        fs::write(&report_path, "old report").expect("old report");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: Some(DEFAULT_BROWSER_VERSION.to_string()),
        };

        let error =
            write_generation_report_with_hook(&config, &GenerationReport::default(), || {
                Err("injected report failure".to_string())
            })
            .expect_err("injected failure should abort report write");

        assert_eq!(error, "injected report failure");
        assert_eq!(
            fs::read_to_string(&report_path).expect("old report should remain"),
            "old report"
        );
        let leftovers = fs::read_dir(report_path.parent().unwrap())
            .expect("report dir should read")
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_string)
            })
            .filter(|name| name.starts_with('.'))
            .collect::<Vec<_>>();
        assert!(
            leftovers.is_empty(),
            "failed report write should clean temporary files, found {leftovers:?}"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn generation_report_successful_write_replaces_existing_report() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-report-replace-{}",
            std::process::id()
        ));
        let report_path = root.join("xml/generation-reports/all.json");
        fs::create_dir_all(report_path.parent().unwrap()).expect("report dir");
        fs::write(
            root.join("corpus.toml"),
            format!(
                r#"
[imports.taffy]
repo = "{TAFFY_REPO}"
commit = "{TAFFY_COMMIT}"
source_dir = "{TAFFY_SOURCE_DIR}"
destination = "html"
expected_count = {TAFFY_EXPECTED_COUNT}
excluded_destination_dirs = ["subgrid", "grid-lanes"]
"#
            ),
        )
        .expect("corpus manifest");
        fs::write(&report_path, "old report").expect("old report");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: Some(DEFAULT_BROWSER_VERSION.to_string()),
        };

        write_generation_report(&config, &GenerationReport::default()).expect("report write");

        let raw = fs::read_to_string(&report_path).expect("report should exist");
        assert!(raw.contains(r#""generator": "surgeist-layout-generate""#));
        let leftovers = fs::read_dir(report_path.parent().unwrap())
            .expect("report dir should read")
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_string)
            })
            .filter(|name| name.starts_with('.'))
            .collect::<Vec<_>>();
        assert!(
            leftovers.is_empty(),
            "successful report write should clean temporary files, found {leftovers:?}"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn successful_full_regeneration_prunes_stale_xml_from_report_outputs() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-prune-stale-{}",
            std::process::id()
        ));
        let xml_root = root.join("xml");
        let kept_xml = xml_root.join("grid/kept.xml");
        let stale_xml = xml_root.join("grid/stale.xml");
        let scoped_report = xml_root.join("generation-reports/scoped.json");
        fs::create_dir_all(kept_xml.parent().unwrap()).expect("kept dir");
        fs::create_dir_all(stale_xml.parent().unwrap()).expect("stale dir");
        fs::create_dir_all(scoped_report.parent().unwrap()).expect("report dir");
        fs::write(&kept_xml, "<kept/>").expect("kept xml");
        fs::write(&stale_xml, "<stale/>").expect("stale xml");
        fs::write(&scoped_report, "{}").expect("report");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: xml_root.clone(),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };
        let mut report = GenerationReport::default();
        report.record_generated(
            "kept".to_string(),
            "html/grid/kept.html".to_string(),
            "xml/grid/kept.xml".to_string(),
            "border_box_ltr".to_string(),
        );

        prune_stale_generated_xml_outputs(&config, &report).expect("stale prune");

        assert!(kept_xml.exists(), "reported XML should be kept");
        assert!(
            !stale_xml.exists(),
            "unreported XML should be pruned after successful full generation"
        );
        assert!(
            scoped_report.exists(),
            "scoped generation reports should not be pruned as XML"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn xml_provenance_freshness_rejects_stale_source_hash() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-xml-provenance-stale-{}",
            std::process::id()
        ));
        let source = root.join("html/grid/basic.html");
        let xml = root.join("xml/grid/basic__border_box_ltr.xml");
        fs::create_dir_all(source.parent().unwrap()).expect("source dir");
        fs::create_dir_all(xml.parent().unwrap()).expect("xml dir");
        fs::write(&source, "<!doctype html><div id=\"test-root\"></div>").expect("source");
        fs::write(
            &xml,
            format!(
                "<!-- generated-by: surgeist-layout-generate schema=1 source=\"html/grid/basic.html\" source-sha256=\"stale\" helper-sha256=\"{}\" browser=\"Chrome/149\" -->\n<test name=\"basic\"/>",
                sha256_bytes(TEST_HELPER_SOURCE.as_bytes())
            ),
        )
        .expect("xml");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };

        let error =
            validate_xml_provenance_freshness(&config).expect_err("stale source hash should fail");

        assert!(error.contains("source-sha256"));
        assert!(error.contains("regenerate browser parity XML"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn xml_provenance_freshness_rejects_stale_browser_version() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-xml-provenance-browser-{}",
            std::process::id()
        ));
        let source = root.join("html/grid/basic.html");
        let xml = root.join("xml/grid/basic__border_box_ltr.xml");
        fs::create_dir_all(source.parent().unwrap()).expect("source dir");
        fs::create_dir_all(xml.parent().unwrap()).expect("xml dir");
        fs::write(&source, "<!doctype html><div id=\"test-root\"></div>").expect("source");
        fs::write(
            &xml,
            format!(
                "<!-- generated-by: surgeist-layout-generate schema=1 source=\"html/grid/basic.html\" source-sha256=\"{}\" helper-sha256=\"{}\" browser=\"chrome-for-testing/148.0.0.0 (target/old-browser)\" -->\n<test name=\"basic\"/>",
                sha256_file(&source).expect("source hash"),
                sha256_bytes(TEST_HELPER_SOURCE.as_bytes())
            ),
        )
        .expect("xml");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: Some(DEFAULT_BROWSER_VERSION.to_string()),
        };

        let error = validate_xml_provenance_freshness(&config)
            .expect_err("stale browser version should fail");

        assert!(error.contains("browser provenance"));
        assert!(error.contains(DEFAULT_BROWSER_VERSION));
        assert!(error.contains("regenerate browser parity XML"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn xml_provenance_freshness_accepts_custom_browser_path() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-xml-provenance-custom-browser-{}",
            std::process::id()
        ));
        let source = root.join("html/grid/basic.html");
        let xml = root.join("xml/grid/basic__border_box_ltr.xml");
        let browser_path = root.join("custom-browser/chrome");
        fs::create_dir_all(source.parent().unwrap()).expect("source dir");
        fs::create_dir_all(xml.parent().unwrap()).expect("xml dir");
        fs::create_dir_all(browser_path.parent().unwrap()).expect("browser dir");
        fs::write(&source, "<!doctype html><div id=\"test-root\"></div>").expect("source");
        fs::write(
            &xml,
            format!(
                "<!-- generated-by: surgeist-layout-generate schema=1 source=\"html/grid/basic.html\" source-sha256=\"{}\" helper-sha256=\"{}\" browser=\"{}\" -->\n<test name=\"basic\"/>",
                sha256_file(&source).expect("source hash"),
                sha256_bytes(TEST_HELPER_SOURCE.as_bytes()),
                browser_path.display()
            ),
        )
        .expect("xml");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root: root.join("xml"),
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: Some(browser_path.clone()),
            browser_version: Some(DEFAULT_BROWSER_VERSION.to_string()),
        };

        assert_eq!(
            browser_provenance(&config, &browser_path),
            browser_path.display().to_string()
        );
        validate_xml_provenance_freshness(&config)
            .expect("custom browser provenance should remain self-consistent");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn failed_full_regeneration_preserves_existing_xml_outputs() {
        let root = std::env::temp_dir().join(format!(
            "surgeist-layout-preserve-on-failure-{}",
            std::process::id()
        ));
        let xml_root = root.join("xml");
        let existing_xml = xml_root.join("grid/existing.xml");
        fs::create_dir_all(existing_xml.parent().unwrap()).expect("xml dir");
        fs::write(&existing_xml, "<existing/>").expect("existing xml");
        let config = Config {
            root: root.clone(),
            html_root: root.join("html"),
            xml_root,
            filter: None,
            browser_cache: PathBuf::from("target/surgeist-browser"),
            browser_path: None,
            browser_version: None,
        };
        let mut report = GenerationReport::default();
        report.record_failed_to_generate(
            "failed".to_string(),
            "html/grid/failed.html".to_string(),
            "browser timeout".to_string(),
        );

        prune_stale_generated_xml_outputs_after_success(&config, &report)
            .expect("failed generation should not prune");

        assert!(
            existing_xml.exists(),
            "failed full generation must preserve existing XML for retry"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn planned_output_commit_failure_restores_existing_xml_outputs() {
        let root =
            std::env::temp_dir().join(format!("surgeist-layout-rollback-{}", std::process::id()));
        let output = root.join("xml/grid/example.xml");
        fs::create_dir_all(output.parent().unwrap()).expect("xml dir");
        fs::write(&output, "<old/>").expect("old xml");
        let planned = vec![PlannedGoldenOutput::Generated {
            name: "example".to_string(),
            source: "html/grid/example.html".to_string(),
            output: "xml/grid/example.xml".to_string(),
            output_file: output.clone(),
            variant: "border_box_ltr".to_string(),
            xml: "<new/>".to_string(),
        }];

        let error = commit_planned_golden_outputs_with_hook(vec![output.clone()], &planned, || {
            Err("injected install failure".to_string())
        })
        .expect_err("injected failure should abort commit");

        assert_eq!(error, "injected install failure");
        assert_eq!(
            fs::read_to_string(&output).expect("old xml should be restored"),
            "<old/>"
        );
        let leftovers = fs::read_dir(output.parent().unwrap())
            .expect("xml dir should read")
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_string)
            })
            .filter(|name| name.starts_with('.'))
            .collect::<Vec<_>>();
        assert!(
            leftovers.is_empty(),
            "rollback should clean temporary files, found {leftovers:?}"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn grid_position_serializes_named_lines_and_spans() {
        assert_eq!(
            grid_position(&serde_json::json!({
                "kind": "named-line",
                "name": "a",
                "value": 8
            })),
            Some("a 8".to_string())
        );
        assert_eq!(
            grid_position(&serde_json::json!({
                "kind": "named-span",
                "name": "a",
                "value": 0
            })),
            Some("span a".to_string())
        );
        assert_eq!(
            grid_position(&serde_json::json!({
                "kind": "named-span",
                "name": "a",
                "value": 2
            })),
            Some("span 2 a".to_string())
        );
    }

    #[test]
    fn track_definition_serializes_subgrid_line_names() {
        assert_eq!(
            track_definition(&serde_json::json!({
                "kind": "subgrid",
                "lineNames": [
                    [],
                    [],
                    [],
                    ["b"]
                ]
            })),
            Some("subgrid [] [] [] [b]".to_string())
        );
    }

    fn create_local_git_source(root: &Path, name: &str) -> (PathBuf, String) {
        let repo = root.join(name);
        fs::create_dir_all(&repo).expect("source repo dir");
        run_git(&repo, ["init"]).expect("source repo init");
        fs::write(repo.join("fixture.txt"), "source").expect("source file");
        run_git(&repo, ["add", "fixture.txt"]).expect("source repo add");
        let output = Command::new("git")
            .current_dir(&repo)
            .args([
                "-c",
                "user.name=Surgeist Test",
                "-c",
                "user.email=surgeist@example.invalid",
                "commit",
                "-m",
                "source",
            ])
            .output()
            .expect("source repo commit command");
        assert!(
            output.status.success(),
            "source repo commit failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let commit = git_head(&repo);
        (repo, commit)
    }

    fn git_head(repo: &Path) -> String {
        let output = Command::new("git")
            .current_dir(repo)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("git rev-parse command");
        assert!(
            output.status.success(),
            "git rev-parse failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn path_str(path: &Path) -> &str {
        path.to_str().expect("test paths should be utf-8")
    }
}
