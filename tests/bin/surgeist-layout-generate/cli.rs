//! Command compatibility and layout corpus declarations for the shared engine.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use surgeist_generator::browser::{
    self, AcquisitionMode, BrowserCorpus, BrowserLaunch, BrowserLocation, BrowserSettings,
    CaseSpec, FixtureSpec, FixtureStatus, GenerationRequest, ReportScope, SourceImportSpec,
};
use surgeist_generator::{
    CorpusLocation, PinnedSource, RelativePath, Sha256Digest, SourceRevision,
};

use crate::adapter::{LayoutAdapter, fixture_cases};

const ROOT_ENV: &str = "SURGEIST_LAYOUT_BROWSER_PARITY_ROOT";
const FILTER_ENV: &str = "SURGEIST_LAYOUT_GENERATE_FILTER";
const BROWSER_PATH_ENV: &str = "SURGEIST_BROWSER_PATH";
const DEFAULT_ROOT: &str = "tests/layout/browser_parity";
const SOURCE_ATTESTATION: &str = "html/.surgeist-source.json";

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Command {
    Generate,
    GenerateExisting,
    CheckCorpus,
    CheckTaffyCorpus,
    ImportTaffy,
}

fn parse_command(arguments: impl IntoIterator<Item = String>) -> Result<Command> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    let [command] = arguments.as_slice() else {
        return Err("usage: surgeist-layout-generate <generate|generate-existing|check-corpus|check-taffy-corpus|import-taffy>".to_string());
    };
    match command.as_str() {
        "generate" => Ok(Command::Generate),
        "generate-existing" => Ok(Command::GenerateExisting),
        "check-corpus" => Ok(Command::CheckCorpus),
        "check-taffy-corpus" => Ok(Command::CheckTaffyCorpus),
        "import-taffy" => Ok(Command::ImportTaffy),
        _ => Err(format!(
            "unknown surgeist-layout-generate command {command:?}"
        )),
    }
}

pub(super) fn run_from_env() -> Result<()> {
    let command = parse_command(env::args().skip(1))?;
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = env::var_os(ROOT_ENV).map_or_else(|| PathBuf::from(DEFAULT_ROOT), PathBuf::from);
    let browser_location = BrowserLocation::new(owner, &root).map_err(message)?;
    let location = browser_location.location();
    let manifest_path = location.corpus_root().join("corpus.toml");
    let manifest_bytes = fs::read_to_string(&manifest_path).map_err(message)?;
    let manifest_digest = Sha256Digest::from_bytes(manifest_bytes.as_bytes());
    let manifest: Manifest =
        surgeist_generator::parse_manifest(&manifest_bytes, &manifest_path).map_err(message)?;
    manifest.validate()?;
    let import = manifest.source_import()?;
    match command {
        Command::ImportTaffy => {
            let source = browser::acquire_source(owner, import.pin(), AcquisitionMode::Managed)
                .map_err(message)?;
            let inputs = BTreeMap::from([(relative("corpus.toml")?, manifest_digest)]);
            browser::import_source_with_expected_inputs(
                location,
                &import,
                source.canonical_root(),
                &inputs,
            )
            .map_err(message)?;
            Ok(())
        }
        Command::CheckTaffyCorpus => {
            let source =
                browser::acquire_source(owner, import.pin(), AcquisitionMode::ExistingOnly)
                    .map_err(message)?;
            browser::verify_source_import(location, &import, source.canonical_root())
                .map_err(message)?;
            revalidate_manifest(&manifest_path, &manifest_digest)?;
            Ok(())
        }
        Command::CheckCorpus => {
            let attestation = browser::verify_import(location, &import).map_err(message)?;
            let inputs = expected_inputs(&manifest_digest, &attestation)?;
            let corpus = manifest
                .corpus(browser_location, attestation.fixture_paths())?
                .with_expected_inputs(inputs)
                .map_err(message)?
                .with_import_provenance(vec![attestation.provenance().map_err(message)?])
                .map_err(message)?;
            browser::check_corpus(&corpus, &LayoutAdapter).map_err(message)?;
            Ok(())
        }
        Command::Generate | Command::GenerateExisting => {
            let environment = GenerationEnvironment::capture(command)?;
            let attestation = browser::verify_import(location, &import).map_err(message)?;
            let fixture_paths = attestation.fixture_paths();
            let filter = environment.filter(location, &fixture_paths)?;
            let prior = if filter.is_none() {
                crate::legacy::prior_ownership(location)?
            } else {
                None
            };
            let inputs = expected_inputs(&manifest_digest, &attestation)?;
            let corpus = manifest
                .corpus(browser_location, fixture_paths)?
                .with_expected_inputs(inputs)
                .map_err(message)?
                .with_import_provenance(vec![attestation.provenance().map_err(message)?])
                .map_err(message)?;
            let executable = match environment.browser_path {
                Some(path) => path,
                None => browser::acquire_browser(owner, corpus.browser(), AcquisitionMode::Managed)
                    .map_err(message)?,
            };
            let request = GenerationRequest::new(corpus, executable, filter.clone(), prior)
                .map_err(message)?;
            browser::generate(request, LayoutAdapter).map_err(message)?;
            Ok(())
        }
    }
}

fn revalidate_manifest(path: &Path, expected: &Sha256Digest) -> Result<()> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to revalidate corpus manifest {}: {error}",
            path.display()
        )
    })?;
    if Sha256Digest::from_bytes(bytes) != *expected {
        return Err("corpus manifest changed during source verification".to_string());
    }
    Ok(())
}

fn expected_inputs(
    manifest_digest: &Sha256Digest,
    attestation: &browser::SourceAttestation,
) -> Result<BTreeMap<RelativePath, Sha256Digest>> {
    let mut inputs = attestation.verified_inputs();
    inputs.insert(relative("corpus.toml")?, manifest_digest.clone());
    Ok(inputs)
}

struct GenerationEnvironment {
    browser_path: Option<RelativePath>,
    filter: Option<String>,
}

impl GenerationEnvironment {
    fn capture(command: Command) -> Result<Self> {
        for variable in ["SURGEIST_BROWSER_CACHE", "SURGEIST_BROWSER_VERSION"] {
            if env::var_os(variable).is_some() {
                return Err(format!(
                    "{variable} is manifest-owned and must be unset for generation"
                ));
            }
        }
        Self::from_values(command, utf8_env(BROWSER_PATH_ENV)?, utf8_env(FILTER_ENV)?)
    }

    fn from_values(
        command: Command,
        browser_path: Option<String>,
        filter: Option<String>,
    ) -> Result<Self> {
        let filter = filter.filter(|value| !value.is_empty());
        if command == Command::Generate {
            if browser_path.is_some() {
                return Err(format!(
                    "{BROWSER_PATH_ENV} is only valid with generate-existing"
                ));
            }
            if filter.is_some() {
                return Err(format!(
                    "{FILTER_ENV} is only valid with generate-existing diagnostic runs"
                ));
            }
        } else if browser_path.as_deref().is_none_or(str::is_empty) {
            return Err(format!(
                "generate-existing requires a non-empty {BROWSER_PATH_ENV} relative to the repository owner"
            ));
        }
        let browser_path = browser_path.map(relative).transpose()?;
        Ok(Self {
            browser_path,
            filter,
        })
    }

    fn filter(
        &self,
        location: &CorpusLocation,
        fixtures: &[RelativePath],
    ) -> Result<Option<RelativePath>> {
        let Some(filter) = &self.filter else {
            return Ok(None);
        };
        let path = relative(filter)?;
        let source = Path::new(path.as_str());
        if source
            .extension()
            .is_some_and(|extension| extension != "html")
        {
            return Err(format!(
                "{FILTER_ENV} must name an HTML fixture or directory prefix"
            ));
        }
        let root = location.corpus_root().join("html");
        let resolved = path.resolve_existing(&root).map_err(message)?;
        let matches = if resolved.is_file() {
            source
                .extension()
                .is_some_and(|extension| extension == "html")
        } else {
            fixtures
                .iter()
                .any(|fixture| Path::new(fixture.as_str()).starts_with(source))
        };
        if !matches {
            return Err(format!("{FILTER_ENV} matched no HTML fixtures"));
        }
        relative(format!("html/{filter}")).map(Some)
    }
}

fn utf8_env(variable: &str) -> Result<Option<String>> {
    env::var_os(variable)
        .map(|value| {
            value
                .into_string()
                .map_err(|_| format!("{variable} must contain valid UTF-8"))
        })
        .transpose()
}

fn relative(value: impl AsRef<str>) -> Result<RelativePath> {
    RelativePath::new(value).map_err(message)
}
fn message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    browser: BrowserManifest,
    generation_reports: Reports,
    source_roots: SourceRoots,
    imports: Imports,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BrowserManifest {
    source: String,
    version: String,
    version_output: String,
    cache_root: String,
    provenance_format: String,
    launch: Launch,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Launch {
    batch_size: usize,
    navigation_timeout_ms: u64,
    dom_poll_interval_ms: u64,
    retry_count: usize,
    job_order: String,
    retry_error_class: String,
    profile_scope: String,
    page_scope: String,
    disable_default_args: bool,
    disable_cache: bool,
    arguments: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reports {
    full: FullReport,
    scoped: Vec<ScopedReport>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FullReport {
    file: String,
    generated: usize,
    unsupported: usize,
    expected_fail: usize,
    quarantined: usize,
    failed_to_generate: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopedReport {
    filter: String,
    file: String,
    generated: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRoots {
    taffy: SourceRoot,
    surgeist: SourceRoot,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRoot {
    kind: String,
    path: String,
    upstream_commit: Option<String>,
    description: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Imports {
    taffy: TaffyImport,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaffyImport {
    repo: String,
    commit: String,
    source_dir: String,
    destination: String,
    expected_count: usize,
    excluded_destination_dirs: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    source_root: CaseRoot,
    source: String,
    generator: CaseGenerator,
    status: CaseStatus,
    reason: Option<String>,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum CaseRoot {
    Taffy,
    Surgeist,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CaseGenerator {
    ConstrainedHtml,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CaseStatus {
    Active,
    ExpectedFail,
    Unsupported,
    Quarantined,
}

impl Manifest {
    fn validate(&self) -> Result<()> {
        if self.schema_version != 2 {
            return Err("corpus manifest schema_version must be 2".to_string());
        }
        let taffy = &self.source_roots.taffy;
        let surgeist = &self.source_roots.surgeist;
        if taffy.kind != "taffy"
            || taffy.path != "html"
            || taffy.upstream_commit.as_deref() != Some(self.imports.taffy.commit.as_str())
            || taffy.description.trim().is_empty()
            || surgeist.kind != "surgeist"
            || surgeist.path != "html"
            || surgeist.upstream_commit.is_some()
            || surgeist.description.trim().is_empty()
            || self.imports.taffy.destination != "html"
        {
            return Err(
                "manifest source roots disagree with the layout corpus declaration".to_string(),
            );
        }
        let launch = &self.browser.launch;
        if launch.retry_count != 1
            || launch.job_order != "sorted-sequential"
            || launch.retry_error_class != "open-load-reset-timeout"
            || launch.profile_scope != "per-batch-and-retry"
            || launch.page_scope != "per-job"
            || !launch.disable_default_args
            || !launch.disable_cache
        {
            return Err(
                "manifest browser lifecycle does not match the retained layout fixture protocol"
                    .to_string(),
            );
        }
        let mut ids = BTreeSet::new();
        let mut sources = BTreeSet::new();
        for case in &self.cases {
            if case.id.trim().is_empty() || !ids.insert(&case.id) {
                return Err("empty or duplicate corpus case identity".to_string());
            }
            let source = relative(&case.source)?;
            if !sources.insert(source.clone()) {
                return Err(format!("duplicate corpus case source {}", source.as_str()));
            }
            match case.generator {
                CaseGenerator::ConstrainedHtml => {}
            }
            if Path::new(source.as_str())
                .extension()
                .is_none_or(|extension| extension != "html")
            {
                return Err(format!("case {} must reference an HTML fixture", case.id));
            }
        }
        Ok(())
    }

    fn browser(&self) -> Result<BrowserSettings> {
        let launch = &self.browser.launch;
        let launch = BrowserLaunch::new(
            launch.batch_size,
            launch.navigation_timeout_ms,
            launch.dom_poll_interval_ms,
            launch.arguments.clone(),
        )
        .map_err(message)?;
        BrowserSettings::new(
            self.browser.source.clone(),
            self.browser.version.clone(),
            self.browser.version_output.clone(),
            relative(&self.browser.cache_root)?,
            self.browser.provenance_format.clone(),
            launch,
        )
        .map_err(message)
    }

    fn source_import(&self) -> Result<SourceImportSpec> {
        let import = &self.imports.taffy;
        let pin = PinnedSource::new(
            "taffy",
            &import.repo,
            SourceRevision::new(&import.commit).map_err(message)?,
            relative(&import.source_dir)?,
        )
        .map_err(message)?;
        SourceImportSpec::new(
            pin,
            relative(&import.destination)?,
            relative(".surgeist-source.json")?,
            "html".to_string(),
            import.expected_count,
            import
                .excluded_destination_dirs
                .iter()
                .map(relative)
                .collect::<Result<_>>()?,
            self.cases
                .iter()
                .filter(|case| case.source_root == CaseRoot::Surgeist)
                .map(|case| relative(&case.source))
                .collect::<Result<_>>()?,
        )
        .map_err(message)
    }

    fn corpus(
        &self,
        browser_location: BrowserLocation,
        files: Vec<RelativePath>,
    ) -> Result<BrowserCorpus> {
        let declarations = self
            .cases
            .iter()
            .filter(|case| case.source_root == CaseRoot::Surgeist)
            .map(|case| (case.source.as_str(), case))
            .collect::<BTreeMap<_, _>>();
        for source in declarations.keys() {
            if !files.iter().any(|file| file.as_str() == *source) {
                return Err(format!("missing declared layout fixture {source}"));
            }
        }
        let fixtures = files
            .into_iter()
            .map(|file| {
                let declaration = declarations.get(file.as_str()).copied();
                let path = Path::new(file.as_str());
                let stem = path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| format!("fixture {} has no UTF-8 stem", file.as_str()))?;
                let name = declaration.map_or_else(|| stem.to_string(), |case| case.id.clone());
                let status = declaration.map_or(FixtureStatus::Active, |case| match case.status {
                    CaseStatus::Active => FixtureStatus::Active,
                    CaseStatus::ExpectedFail => FixtureStatus::ExpectedFail {
                        reason: case
                            .reason
                            .clone()
                            .unwrap_or_else(|| "manifest marks case expected-fail".to_string()),
                    },
                    CaseStatus::Unsupported => FixtureStatus::Unsupported {
                        reason: case
                            .reason
                            .clone()
                            .unwrap_or_else(|| "manifest marks case unsupported".to_string()),
                    },
                    CaseStatus::Quarantined => FixtureStatus::Quarantined {
                        reason: case
                            .reason
                            .clone()
                            .unwrap_or_else(|| "manifest marks case quarantined".to_string()),
                    },
                });
                let cases = fixture_cases()
                    .into_iter()
                    .map(|(variant, _)| {
                        let id = format!("{stem}__{variant}");
                        let output = Path::new("xml")
                            .join(path.parent().unwrap_or_else(|| Path::new("")))
                            .join(format!("{id}.xml"));
                        CaseSpec::new(id, variant.to_string(), relative(output.to_string_lossy())?)
                            .map_err(message)
                    })
                    .collect::<Result<Vec<_>>>()?;
                FixtureSpec::new(
                    name,
                    relative(format!("html/{}", file.as_str()))?,
                    cases,
                    status,
                )
                .map_err(message)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut reports = vec![
            ReportScope::new(
                relative(format!(
                    "generation-reports/{}",
                    self.generation_reports.full.file
                ))?,
                None,
            )
            .map_err(message)?,
        ];
        for scope in &self.generation_reports.scoped {
            reports.push(
                ReportScope::new(
                    relative(format!("generation-reports/{}", scope.file))?,
                    Some(relative(format!("html/{}", scope.filter))?),
                )
                .map_err(message)?
                .with_expected_generated(scope.generated)
                .map_err(message)?,
            );
        }
        let corpus = BrowserCorpus::new(
            browser_location,
            relative("corpus.toml")?,
            "surgeist-layout-generate".to_string(),
            relative("xml")?,
            reports,
            self.browser()?,
            fixtures,
            vec![
                relative("scripts/gentest/test_helper.js")?,
                relative("scripts/gentest/test_base_style.css")?,
            ],
            vec![relative(SOURCE_ATTESTATION)?],
        )
        .map_err(message)?
        .with_expected_counts(self.expected_counts()?)
        .with_exact_input_roots(vec![relative("scripts/gentest")?])
        .map_err(message)?;
        Ok(corpus)
    }

    fn expected_counts(&self) -> Result<browser::BrowserReportSummary> {
        let expected = &self.generation_reports.full;
        browser::BrowserReportSummary::new(
            expected.generated,
            expected.unsupported,
            expected.expected_fail,
            expected.quarantined,
            expected.failed_to_generate,
        )
        .map_err(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_check_requires_the_captured_manifest_to_remain_present_and_unchanged() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "surgeist-layout-manifest-check-{}-{nonce}.toml",
            std::process::id()
        ));
        fs::File::create_new(&path).unwrap();
        let original = b"source_pin = 'captured'\n";
        fs::write(&path, original).unwrap();
        let expected = Sha256Digest::from_bytes(original);
        revalidate_manifest(&path, &expected).expect("unchanged manifest remains current");

        fs::write(&path, b"source_pin = 'changed'\n").unwrap();
        let changed = revalidate_manifest(&path, &expected)
            .expect_err("changed manifest cannot authorize a successful source check");
        assert!(changed.contains("manifest"), "{changed}");

        fs::remove_file(&path).unwrap();
        let missing = revalidate_manifest(&path, &expected)
            .expect_err("missing manifest cannot authorize a successful source check");
        assert!(missing.contains("manifest"), "{missing}");
    }

    #[test]
    fn existing_browser_keeps_the_full_owner_relative_path() {
        let path = "tmp/surgeist-browser/mac_arm-149.0.7827.115/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing";
        let environment = GenerationEnvironment::from_values(
            Command::GenerateExisting,
            Some(path.to_string()),
            None,
        )
        .unwrap();
        assert_eq!(environment.browser_path.unwrap().as_str(), path);
    }

    #[test]
    fn generation_commands_reject_incompatible_browser_and_filter_inputs() {
        assert!(
            GenerationEnvironment::from_values(
                Command::Generate,
                Some("tmp/browser".to_string()),
                None
            )
            .is_err()
        );
        assert!(
            GenerationEnvironment::from_values(Command::Generate, None, Some("grid".to_string()))
                .is_err()
        );
        assert!(GenerationEnvironment::from_values(Command::GenerateExisting, None, None).is_err());
        assert!(
            GenerationEnvironment::from_values(
                Command::GenerateExisting,
                Some("../browser".to_string()),
                None
            )
            .is_err()
        );
        assert!(
            GenerationEnvironment::from_values(
                Command::GenerateExisting,
                Some("/absolute/browser".to_string()),
                None
            )
            .is_err()
        );
        assert!(
            GenerationEnvironment::from_values(Command::Generate, None, Some(String::new()))
                .unwrap()
                .filter
                .is_none()
        );
    }

    #[test]
    fn only_the_five_existing_commands_are_accepted() {
        for (name, command) in [
            ("generate", Command::Generate),
            ("generate-existing", Command::GenerateExisting),
            ("check-corpus", Command::CheckCorpus),
            ("check-taffy-corpus", Command::CheckTaffyCorpus),
            ("import-taffy", Command::ImportTaffy),
        ] {
            assert_eq!(parse_command([name.to_string()]).unwrap(), command);
        }
        assert!(parse_command([]).is_err());
        assert!(parse_command(["check-corpus".to_string(), "extra".to_string()]).is_err());
        assert!(parse_command(["unknown".to_string()]).is_err());
    }

    #[test]
    fn declared_fixture_variants_keep_names_and_corpus_relative_outputs() {
        let mut manifest: Manifest = surgeist_generator::parse_manifest(
            include_str!("../../layout/browser_parity/corpus.toml"),
            "corpus.toml",
        )
        .unwrap();
        manifest.cases.clear();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let corpus = manifest
            .corpus(
                BrowserLocation::new(root, root).unwrap(),
                vec![RelativePath::new("grid/example.html").unwrap()],
            )
            .unwrap();
        let fixture = &corpus.fixtures()[0];
        assert_eq!(fixture.source().as_str(), "html/grid/example.html");
        let cases = fixture
            .cases()
            .iter()
            .map(|case| (case.id(), case.variant(), case.output().as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            cases,
            [
                (
                    "example__border_box_ltr",
                    "border_box_ltr",
                    "xml/grid/example__border_box_ltr.xml"
                ),
                (
                    "example__content_box_ltr",
                    "content_box_ltr",
                    "xml/grid/example__content_box_ltr.xml"
                ),
                (
                    "example__border_box_rtl",
                    "border_box_rtl",
                    "xml/grid/example__border_box_rtl.xml"
                ),
                (
                    "example__content_box_rtl",
                    "content_box_rtl",
                    "xml/grid/example__content_box_rtl.xml"
                ),
            ]
        );
    }
}
