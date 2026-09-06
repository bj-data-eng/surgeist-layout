//! Narrow ownership adoption for the layout generator's schema-3 reports.
//!
//! Historical metadata is evidence of prior output ownership, never current
//! provenance. The engine independently captures and checks every returned path.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use surgeist_generator::browser::PriorOwnership;
use surgeist_generator::{CorpusLocation, RelativePath, Sha256Digest};

type Result<T> = std::result::Result<T, String>;

pub(super) fn prior_ownership(location: &CorpusLocation) -> Result<Option<PriorOwnership>> {
    let root = location.corpus_root();
    let manifest_path = path("corpus.toml")?;
    let manifest_bytes = read(root, &manifest_path)?;
    let manifest: OwnershipManifest = surgeist_generator::parse_manifest(
        std::str::from_utf8(&manifest_bytes).map_err(message)?,
        manifest_path.as_str(),
    )
    .map_err(message)?;
    let full_path = report_path(&manifest.generation_reports.full.file)?;
    if !full_path.join(root).try_exists().map_err(message)? {
        return Ok(None);
    }
    let full_bytes = read(root, &full_path)?;
    let value: serde_json::Value = serde_json::from_slice(&full_bytes).map_err(message)?;
    if value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        == Some(4)
        || value
            .get("metadata")
            .and_then(|metadata| metadata.get("schema_version"))
            .and_then(serde_json::Value::as_u64)
            == Some(4)
    {
        return Ok(None);
    }
    let full: LegacyReport = serde_json::from_value(value).map_err(message)?;
    let artifacts = validate_report(root, &full, None)?;
    if full.summary != manifest.generation_reports.full.counts {
        return Err("legacy full report accounting disagrees with its manifest".to_string());
    }
    let mut evidence = vec![
        (manifest_path, Sha256Digest::from_bytes(&manifest_bytes)),
        (full_path, Sha256Digest::from_bytes(&full_bytes)),
    ];
    let mut report_paths = evidence
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<BTreeSet<_>>();
    for declaration in &manifest.generation_reports.scoped {
        let scoped_path = report_path(&declaration.file)?;
        if !report_paths.insert(scoped_path.clone()) {
            return Err("duplicate legacy report declaration".to_string());
        }
        let filter = path(&declaration.filter)?;
        let bytes = read(root, &scoped_path)?;
        let scoped: LegacyReport = serde_json::from_slice(&bytes).map_err(message)?;
        validate_report(root, &scoped, Some(filter.as_str()))?;
        if scoped.summary.generated != declaration.generated
            || scoped.metadata != full.metadata
            || generated_map(&scoped) != generated_map_filtered(&full, &filter)
            || scoped.unsupported != filtered(&full.unsupported, &filter, |entry| &entry.source)
            || scoped.expected_fail != filtered(&full.expected_fail, &filter, |entry| &entry.source)
            || scoped.quarantined != filtered(&full.quarantined, &filter, |entry| &entry.source)
            || scoped.failed_to_generate
                != filtered(&full.failed_to_generate, &filter, |entry| &entry.source)
        {
            return Err(format!(
                "legacy scoped report {} disagrees with the full report",
                scoped_path.as_str()
            ));
        }
        evidence.push((scoped_path, Sha256Digest::from_bytes(&bytes)));
    }
    PriorOwnership::new(evidence, artifacts)
        .map(Some)
        .map_err(message)
}

fn validate_report(
    root: &Path,
    report: &LegacyReport,
    expected_filter: Option<&str>,
) -> Result<Vec<(RelativePath, Sha256Digest)>> {
    if report
        .metadata
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(3)
        || report
            .metadata
            .get("generator")
            .and_then(serde_json::Value::as_str)
            != Some("surgeist-layout-generate")
    {
        return Err("unsupported legacy report version or generator".to_string());
    }
    if report.filter.as_deref() != expected_filter {
        return Err("legacy report filter disagrees with its declaration".to_string());
    }
    let actual = Counts {
        generated: report.generated.len(),
        unsupported: report.unsupported.len(),
        expected_fail: report.expected_fail.len(),
        quarantined: report.quarantined.len(),
        failed_to_generate: report.failed_to_generate.len(),
    };
    if report.summary != actual || !report.failed_to_generate.is_empty() {
        return Err("legacy report accounting is incomplete or inconsistent".to_string());
    }
    let mut outputs = BTreeSet::new();
    let mut cases = BTreeSet::new();
    let mut artifacts = Vec::new();
    for entry in &report.generated {
        let output = RelativePath::with_extension(&entry.output, "xml").map_err(message)?;
        if !output.as_str().starts_with("xml/") || !outputs.insert(output.clone()) {
            return Err("duplicate or outside legacy artifact path".to_string());
        }
        if !cases.insert((&entry.source, &entry.name, &entry.variant)) {
            return Err("duplicate legacy case identity".to_string());
        }
        validate_source(&entry.source)?;
        if entry.name.is_empty() || entry.variant.is_empty() {
            return Err("empty legacy case identity".to_string());
        }
        let bytes = read(root, &output)?;
        if Sha256Digest::from_bytes(&bytes) != entry.xml_sha256 {
            return Err(format!(
                "legacy output digest mismatch: {}",
                output.as_str()
            ));
        }
        artifacts.push((output, entry.xml_sha256.clone()));
    }
    for entry in &report.unsupported {
        validate_source(&entry.source)?;
        if !cases.insert((&entry.source, &entry.name, &entry.variant)) {
            return Err("duplicate legacy case disposition".to_string());
        }
    }
    for entry in report
        .expected_fail
        .iter()
        .chain(&report.quarantined)
        .chain(&report.failed_to_generate)
    {
        validate_source(&entry.source)?;
    }
    Ok(artifacts)
}

fn validate_source(source: &str) -> Result<()> {
    let source = RelativePath::with_extension(source, "html").map_err(message)?;
    if !source.as_str().starts_with("html/") {
        return Err("legacy source is outside html".to_string());
    }
    Ok(())
}

fn generated_map(report: &LegacyReport) -> BTreeMap<&str, &GeneratedEntry> {
    report
        .generated
        .iter()
        .map(|entry| (entry.output.as_str(), entry))
        .collect()
}

fn generated_map_filtered<'a>(
    report: &'a LegacyReport,
    filter: &RelativePath,
) -> BTreeMap<&'a str, &'a GeneratedEntry> {
    report
        .generated
        .iter()
        .filter(|entry| matches_filter(&entry.source, filter))
        .map(|entry| (entry.output.as_str(), entry))
        .collect()
}

fn filtered<T: Clone + Ord>(
    entries: &[T],
    filter: &RelativePath,
    source: impl Fn(&T) -> &str,
) -> Vec<T> {
    let mut result = entries
        .iter()
        .filter(|entry| matches_filter(source(entry), filter))
        .cloned()
        .collect::<Vec<_>>();
    result.sort();
    result
}

fn matches_filter(source: &str, filter: &RelativePath) -> bool {
    let prefix = format!("html/{}", filter.as_str());
    source == prefix
        || source
            .strip_prefix(&prefix)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn report_path(file: &str) -> Result<RelativePath> {
    let file = RelativePath::with_extension(file, "json").map_err(message)?;
    path(format!("xml/generation-reports/{}", file.as_str()))
}
fn path(value: impl AsRef<str>) -> Result<RelativePath> {
    RelativePath::new(value).map_err(message)
}
fn message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn read(root: &Path, relative: &RelativePath) -> Result<Vec<u8>> {
    let mut current = PathBuf::from(root);
    for component in relative.as_str().split('/') {
        current.push(component);
        let metadata = fs::symlink_metadata(&current).map_err(message)?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "symlink in legacy evidence path: {}",
                relative.as_str()
            ));
        }
    }
    fs::read(current).map_err(message)
}

#[derive(Deserialize)]
struct OwnershipManifest {
    generation_reports: ReportDeclarations,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReportDeclarations {
    full: FullDeclaration,
    scoped: Vec<ScopedDeclaration>,
}
#[derive(Deserialize)]
struct FullDeclaration {
    file: String,
    #[serde(flatten)]
    counts: Counts,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopedDeclaration {
    file: String,
    filter: String,
    generated: usize,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct Counts {
    generated: usize,
    unsupported: usize,
    expected_fail: usize,
    quarantined: usize,
    failed_to_generate: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyReport {
    metadata: serde_json::Value,
    filter: Option<String>,
    summary: Counts,
    generated: Vec<GeneratedEntry>,
    unsupported: Vec<UnsupportedEntry>,
    expected_fail: Vec<StatusEntry>,
    quarantined: Vec<StatusEntry>,
    failed_to_generate: Vec<StatusEntry>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct GeneratedEntry {
    name: String,
    source: String,
    output: String,
    variant: String,
    source_sha256: Sha256Digest,
    linked_resources: Vec<ResourceEntry>,
    xml_sha256: Sha256Digest,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct ResourceEntry {
    path: RelativePath,
    sha256: Sha256Digest,
}
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
struct UnsupportedEntry {
    name: String,
    source: String,
    variant: String,
    reason: String,
}
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
struct StatusEntry {
    name: String,
    source: String,
    reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::fs;
    use std::path::PathBuf;
    use surgeist_generator::Sha256Digest;

    struct Corpus(PathBuf);
    impl Corpus {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let root = std::env::temp_dir()
                .join(format!("layout-legacy-{}-{nonce}-{id}", std::process::id()));
            fs::create_dir(&root).unwrap();
            fs::create_dir_all(root.join("xml/generation-reports")).unwrap();
            fs::create_dir_all(root.join("xml/block")).unwrap();
            fs::write(
                root.join("xml/block/case__border_box_ltr.xml"),
                b"original artifact\n",
            )
            .unwrap();
            fs::write(root.join("corpus.toml"), "[generation_reports]\nscoped = []\n[generation_reports.full]\nfile = \"all.json\"\ngenerated = 1\nunsupported = 0\nexpected_fail = 0\nquarantined = 0\nfailed_to_generate = 0\n").unwrap();
            Self(root)
        }
        fn location(&self) -> CorpusLocation {
            CorpusLocation::new(&self.0, &self.0).unwrap()
        }
        fn report(&self, value: &Value) {
            fs::write(
                self.0.join("xml/generation-reports/all.json"),
                serde_json::to_vec(value).unwrap(),
            )
            .unwrap();
        }
    }
    impl Drop for Corpus {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn report() -> Value {
        let hash = Sha256Digest::from_bytes(b"source").to_string();
        json!({
            "metadata": {
                "schema_version":3,"generator":"surgeist-layout-generate",
                "browser_source":"chrome-for-testing","browser_version":"149.0.7827.115",
                "browser":"historical cache location","launch_profile_sha256":hash,
                "helper_sha256":hash,"base_style_sha256":hash,"corpus_manifest_sha256":hash,
                "taffy_commit":"d1ff7e339b9ee35b33858779f8d7653197e93d92"
            },
            "filter":null,
            "summary":{"generated":1,"unsupported":0,"expected_fail":0,"quarantined":0,"failed_to_generate":0},
            "generated":[{"name":"case__border_box_ltr","source":"html/block/case.html",
                "output":"xml/block/case__border_box_ltr.xml","variant":"border_box_ltr",
                "source_sha256":hash,"linked_resources":[],
                "xml_sha256":Sha256Digest::from_bytes(b"original artifact\n").to_string()}],
            "unsupported":[],"expected_fail":[],"quarantined":[],"failed_to_generate":[]
        })
    }
    #[test]
    fn legacy_report_adopts_verified_outputs_without_claiming_current_metadata() {
        let corpus = Corpus::new();
        corpus.report(&report());
        assert!(prior_ownership(&corpus.location()).unwrap().is_some());
        assert_eq!(
            fs::read(corpus.0.join("xml/block/case__border_box_ltr.xml")).unwrap(),
            b"original artifact\n"
        );
    }
    #[test]
    fn legacy_adoption_is_absent_for_missing_or_current_reports() {
        let corpus = Corpus::new();
        assert!(prior_ownership(&corpus.location()).unwrap().is_none());
        corpus.report(&json!({"schema_version":4}));
        assert!(prior_ownership(&corpus.location()).unwrap().is_none());
    }
    #[test]
    fn legacy_adoption_rejects_changed_output_bytes() {
        let corpus = Corpus::new();
        corpus.report(&report());
        fs::write(
            corpus.0.join("xml/block/case__border_box_ltr.xml"),
            "changed",
        )
        .unwrap();
        assert!(
            prior_ownership(&corpus.location())
                .unwrap_err()
                .contains("digest")
        );
    }
    #[test]
    fn legacy_adoption_rejects_duplicate_and_escaping_outputs() {
        for duplicate in [false, true] {
            let corpus = Corpus::new();
            let mut value = report();
            if duplicate {
                let entry = value["generated"][0].clone();
                value["generated"].as_array_mut().unwrap().push(entry);
                value["summary"]["generated"] = json!(2);
                let p = corpus.0.join("corpus.toml");
                fs::write(
                    &p,
                    fs::read_to_string(&p)
                        .unwrap()
                        .replace("generated = 1", "generated = 2"),
                )
                .unwrap();
            } else {
                value["generated"][0]["output"] = json!("xml/../../outside.xml");
            }
            corpus.report(&value);
            assert!(prior_ownership(&corpus.location()).is_err());
        }
    }
    #[test]
    fn legacy_adoption_rejects_inconsistent_counts_and_unknown_versions() {
        for (field, value) in [("count", json!(2)), ("version", json!(5))] {
            let corpus = Corpus::new();
            let mut contents = report();
            if field == "count" {
                contents["summary"]["generated"] = value;
            } else {
                contents["metadata"]["schema_version"] = value;
            }
            corpus.report(&contents);
            assert!(prior_ownership(&corpus.location()).is_err());
        }
    }
    #[test]
    fn legacy_adoption_checks_scoped_entries_against_full_report() {
        let corpus = Corpus::new();
        corpus.report(&report());
        let mut scoped = report();
        scoped["filter"] = json!("block");
        let manifest = corpus.0.join("corpus.toml");
        let contents = fs::read_to_string(&manifest)
            .unwrap()
            .replace("scoped = []\n", "");
        fs::write(&manifest, format!("{contents}\n[[generation_reports.scoped]]\nfile = \"block.json\"\nfilter = \"block\"\ngenerated = 1\n")).unwrap();
        let path = corpus.0.join("xml/generation-reports/block.json");
        fs::write(&path, serde_json::to_vec(&scoped).unwrap()).unwrap();
        assert!(prior_ownership(&corpus.location()).unwrap().is_some());
        scoped["generated"][0]["name"] = json!("another_name");
        fs::write(&path, serde_json::to_vec(&scoped).unwrap()).unwrap();
        assert!(
            prior_ownership(&corpus.location())
                .unwrap_err()
                .contains("scoped")
        );
    }
    #[cfg(unix)]
    #[test]
    fn legacy_adoption_rejects_symlinked_evidence() {
        let corpus = Corpus::new();
        corpus.report(&report());
        let path = corpus.0.join("xml/generation-reports/all.json");
        let other = corpus.0.join("other.json");
        fs::rename(&path, &other).unwrap();
        std::os::unix::fs::symlink(&other, &path).unwrap();
        assert!(
            prior_ownership(&corpus.location())
                .unwrap_err()
                .contains("symlink")
        );
    }
}
