use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GradleClasspath {
    pub modules: Vec<GradleModule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GradleModule {
    pub name: String,
    pub path: String,
    pub classpath: Vec<PathBuf>,
    pub source_roots: Vec<PathBuf>,
}

#[derive(Debug)]
pub enum GradleError {
    Io(std::io::Error),
    Json(serde_json::Error),
    CommandFailed(String),
    OutputMissing,
}

impl From<std::io::Error> for GradleError {
    fn from(err: std::io::Error) -> Self {
        GradleError::Io(err)
    }
}

impl From<serde_json::Error> for GradleError {
    fn from(err: serde_json::Error) -> Self {
        GradleError::Json(err)
    }
}

pub fn run_gradle_classpath(root: &Path) -> Result<GradleClasspath, GradleError> {
    let (init_script, output_file) = write_init_script()?;
    let gradle = find_gradle_command(root);
    let output_arg = format!("-PkotlinAnalyzerOutput={}", output_file.display());

    let status = Command::new(gradle)
        .current_dir(root)
        .arg("--no-daemon")
        .arg("-q")
        .arg("-I")
        .arg(&init_script)
        .arg(output_arg)
        .arg("kotlinAnalyzerClasspath")
        .status()?;

    if !status.success() {
        return Err(GradleError::CommandFailed(format!(
            "gradle exited with status {status}"
        )));
    }

    let json = fs::read_to_string(&output_file).map_err(|_| GradleError::OutputMissing)?;
    let parsed = serde_json::from_str::<GradleClasspath>(&json)?;
    Ok(parsed)
}

fn find_gradle_command(root: &Path) -> PathBuf {
    let wrapper = if cfg!(windows) { "gradlew.bat" } else { "gradlew" };
    let wrapper_path = root.join(wrapper);
    if wrapper_path.exists() {
        if cfg!(windows) {
            return wrapper_path;
        }
        return PathBuf::from(format!("./{wrapper}"));
    }
    PathBuf::from("gradle")
}

fn write_init_script() -> Result<(PathBuf, PathBuf), std::io::Error> {
    let temp_dir = env::temp_dir().join(format!(
        "kotlin-analyzer-gradle-{}",
        unique_suffix()
    ));
    fs::create_dir_all(&temp_dir)?;

    let init_script = temp_dir.join("init.gradle");
    let output_file = temp_dir.join("classpath.json");

    fs::write(&init_script, init_script_body())?;

    // Ensure the output path is absolute for Gradle.
    let output_file = output_file
        .canonicalize()
        .unwrap_or(output_file);

    Ok((init_script, output_file))
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn init_script_body() -> &'static str {
    r#"
import groovy.json.JsonOutput

def outputPath = null
if (hasProperty("kotlinAnalyzerOutput")) {
    outputPath = project.property("kotlinAnalyzerOutput")
}

def collectSourceRoots = { proj ->
    def roots = []
    def sourceSets = proj.extensions.findByName("sourceSets")
    if (sourceSets != null) {
        def main = sourceSets.findByName("main")
        if (main != null) {
            def kotlinExt = main.extensions.findByName("kotlin")
            if (kotlinExt != null && kotlinExt.hasProperty("srcDirs")) {
                roots.addAll(kotlinExt.srcDirs)
            }
            if (main.hasProperty("java")) {
                roots.addAll(main.java.srcDirs)
            }
            if (main.hasProperty("resources")) {
                roots.addAll(main.resources.srcDirs)
            }
        }
    }
    roots.findAll { it != null }.collect { it.absolutePath }
}

def collectClasspath = { proj ->
    def files = []
    def conf = proj.configurations.findByName("compileClasspath")
    if (conf != null) {
        try {
            files.addAll(conf.resolve())
        } catch (Exception ignored) {
        }
    }
    files.findAll { it != null }.collect { it.absolutePath }
}

allprojects { proj ->
    tasks.register("kotlinAnalyzerClasspath") {
        doLast {
            def modules = []
            rootProject.allprojects.each { p ->
                def moduleInfo = [
                    name: p.name,
                    path: p.path,
                    classpath: collectClasspath(p),
                    source_roots: collectSourceRoots(p),
                ]
                modules.add(moduleInfo)
            }

            def payload = [modules: modules]
            def json = JsonOutput.toJson(payload)

            if (outputPath == null || outputPath.toString().isEmpty()) {
                println(json)
            } else {
                def file = new File(outputPath.toString())
                file.parentFile.mkdirs()
                file.text = json
            }
        }
    }
}
"#
}
