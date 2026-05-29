use std::{
    fs,
    io::Error,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::OnceLock,
};

#[derive(Debug)]
struct TestRepository {
    remote: PathBuf,
    pub local: PathBuf,
}
impl TestRepository {
    pub fn new() -> Self {
        let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

        let remote = base_dir.join("tests").join("repo").join("remote");
        let local = base_dir.join("tests").join("repo").join("local");

        Self { remote, local }
    }
}

static TEST_REPO: OnceLock<TestRepository> = OnceLock::new();
static EXEC_PATH: OnceLock<String> = OnceLock::new();

fn initialize() {
    EXEC_PATH.get_or_init(|| {
        let gft = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("target")
            .join("debug")
            .join("git-flow-rs");

        let gf = gft.parent().unwrap().join("git-flow-rs");

        gf.display().to_string()
    });

    TEST_REPO.get_or_init(|| {
        let inst = TestRepository::new();
        println!("Initializing test repository...");

        println!("Creating remote in {:?}...", &inst.remote);
        if inst.remote.exists() {
            fs::remove_dir_all(&inst.remote).unwrap();
        }
        fs::create_dir_all(&inst.remote).unwrap();
        if inst.local.exists() {
            fs::remove_dir_all(&inst.local).unwrap();
        }
        let script = format!(
            r#"
set -e

git init --bare "{remote}"
git clone "{remote}" "{local}"

cd "{local}"

git config user.email "test@example.com"
git config user.name "test"

touch obj-main
git add .
git commit -m "First commit"
git push

git switch -c develop
touch obj-develop
git add .
git commit -m "Sec commit"
git push --set-upstream origin develop
"#,
            remote = inst.remote.display(),
            local = inst.local.display()
        );
        if let Ok(r) = Command::new("bash").arg("-c").arg(script).output()
            && let Some(status) = r.status.code()
            && status == 0
        {
            return inst;
        }

        panic!("Could not generate repository");
    });
}

fn run_cmd(args: &[&str]) -> Result<Output, Error> {
    println!(
        "Running command '{} {}' in {:?}",
        EXEC_PATH.get().unwrap(),
        args.join(" "),
        &TEST_REPO.get().unwrap().local
    );
    let res = Command::new(EXEC_PATH.get().unwrap())
        .args(args)
        .current_dir(&TEST_REPO.get().unwrap().local)
        .output();

    println!("{}", String::from_utf8_lossy(&res.as_ref().unwrap().stdout));

    res
}

#[test]
fn test_01_features() {
    initialize();

    let res = run_cmd(&["feature", "test-features", "start"]).unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error creating feature branch: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }

    fs::write(TEST_REPO.get().unwrap().local.join("obj-feature"), "").unwrap();
    let script = r#"
set -e

git add .
git commit -m "Feature adds file"
"#;
    let res = Command::new("bash")
        .arg("-c")
        .arg(script)
        .current_dir(TEST_REPO.get().unwrap().local.display().to_string())
        .output()
        .unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error commiting file: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }

    let res = run_cmd(&["feature", "test-features", "finish"]).unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error finishing feature branch: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }
}

#[test]
fn test_02_bugfix() {
    initialize();

    let res = run_cmd(&["bugfix", "test-bugfix", "start"]).unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error creating bugfix branch: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }

    fs::write(TEST_REPO.get().unwrap().local.join("obj-bugfix"), "").unwrap();
    let script = r#"
set -e

git add .
git commit -m "Bugfix adds file"
"#;
    let res = Command::new("bash")
        .arg("-c")
        .arg(script)
        .current_dir(TEST_REPO.get().unwrap().local.display().to_string())
        .output()
        .unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error commiting file: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }

    let res = run_cmd(&["bugfix", "test-bugfix", "finish"]).unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error finishing bugfix branch: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }
}

#[test]
fn test_03_hotfix() {
    initialize();

    let res = run_cmd(&["hotfix", "test-hotfix", "start"]).unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error creating hotfix branch: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }

    fs::write(TEST_REPO.get().unwrap().local.join("obj-hotfix"), "").unwrap();
    let script = r#"
set -e

git add .
git commit -m "Hotfix adds file"
"#;
    let res = Command::new("bash")
        .arg("-c")
        .arg(script)
        .current_dir(TEST_REPO.get().unwrap().local.display().to_string())
        .output()
        .unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error commiting file: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }

    let res = run_cmd(&["hotfix", "test-hotfix", "finish"]).unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error finishing hotfix branch: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }
}

#[test]
fn test_04_release() {
    initialize();

    let res = run_cmd(&["release", "test-release", "start"]).unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error creating release branch: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }

    fs::write(TEST_REPO.get().unwrap().local.join("obj-release"), "").unwrap();
    let script = r#"
set -e

git add .
git commit -m "Release adds file"
"#;
    let res = Command::new("bash")
        .arg("-c")
        .arg(script)
        .current_dir(TEST_REPO.get().unwrap().local.display().to_string())
        .output()
        .unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error commiting file: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }

    let res = run_cmd(&["release", "test-release", "finish"]).unwrap();
    if res.status.code().unwrap() != 0 {
        panic!(
            "Error finishing release branch: {:?}:\n{}",
            res.status.code(),
            String::from_utf8_lossy(&res.stderr)
        );
    }
}
