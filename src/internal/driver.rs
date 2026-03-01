use {
    crate::data::Str,
    crate::internal::platform::{
        canonicalize, create_dir_all, var, Command, Error, ErrorKind, Path, PathBuf, Result,
    },
    crate::tracker::Location,
};

pub struct Driver;

impl Driver {
    const CLANG_CANDIDATES: [&'static str; 4] = ["clang-21", "clang-20", "clang-19", "clang"];
    const LINKER_CANDIDATES: [&'static str; 3] = ["ld.lld", "lld", "ld"];

    #[cfg(target_os = "macos")]
    fn sysroot() -> Option<String> {
        let output = Command::new("xcrun").arg("--show-sdk-path").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let root = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if root.is_empty() {
            None
        } else {
            Some(root)
        }
    }

    fn env_path(keys: &[&str]) -> Option<PathBuf> {
        for key in keys {
            if let Ok(value) = var(key) {
                if !value.trim().is_empty() {
                    return Some(PathBuf::from(value));
                }
            }
        }
        None
    }

    fn accepts(compiler: &Path) -> bool {
        let output = Command::new(compiler).arg("--version").output();
        match output {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
                text.contains("clang") || text.contains("llvm")
            }
            Err(_) => false,
        }
    }

    fn linker() -> Option<PathBuf> {
        let direct = Self::env_path(&["AXO_LINKER_PATH", "LD"]);
        if direct.is_some() {
            return direct;
        }

        Self::llvm_bindir()
            .and_then(|bindir| Self::first_existing_in(&bindir, &Self::LINKER_CANDIDATES))
    }

    fn linker_flag(linker: &Path) -> Option<String> {
        let name = linker
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if name.contains("lld") {
            return Some("lld".to_string());
        }

        if linker.is_absolute() {
            return Some(linker.to_string_lossy().to_string());
        }

        None
    }

    fn compile(compiler: &Path, code: &Path, binary: &Path, bootstrap: bool) -> Result<()> {
        let mut command = Command::new(compiler);
        command.arg(code);
        let compiler_name = compiler
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if let Some(linker) = Self::linker().as_deref().and_then(Self::linker_flag) {
            command.arg(format!("-fuse-ld={}", linker));
        }
        if bootstrap {
            command
                .arg("-nostdlib")
                .arg("-nodefaultlibs")
                .arg("-nostartfiles")
                .arg("-Wl,-e,_start");
        }
        #[cfg(target_os = "macos")]
        if let Some(root) = Self::sysroot() {
            command.arg("-isysroot").arg(root);
        }
        command.arg("-o").arg(binary);

        let output = command.output();

        match output {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(Error::new(
                ErrorKind::Other,
                format!(
                    "{} failed: {}",
                    compiler.display(),
                    String::from_utf8_lossy(&output.stderr)
                ),
            )),
            Err(error) => Err(error),
        }
    }

    fn binary(mut path: PathBuf) -> PathBuf {
        if cfg!(target_os = "windows") && path.extension().is_none() {
            path.set_extension("exe");
        }
        path
    }

    fn from_prefix() -> Option<PathBuf> {
        Self::llvm_bindir()
            .and_then(|bindir| Self::first_existing_in(&bindir, &Self::CLANG_CANDIDATES))
    }

    fn llvm_bindir() -> Option<PathBuf> {
        Self::env_path(&["AXO_LLVM_BINDIR"]).or_else(|| {
            Self::env_path(&[
                "AXO_LLVM_PREFIX",
                "AXO_LLVM19_PREFIX",
                "AXO_LLVM18_PREFIX",
                "LLVM_SYS_191_PREFIX",
                "LLVM_SYS_181_PREFIX",
            ])
            .map(|p| p.join("bin"))
        })
    }

    fn first_existing_in(dir: &Path, names: &[&str]) -> Option<PathBuf> {
        for name in names {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        None
    }

    pub fn paths<'driver>(
        target: Location<'driver>,
        module: &str,
        code: Option<Str<'driver>>,
        executable: Option<Str<'driver>>,
    ) -> (PathBuf, PathBuf) {
        let code = code
            .as_ref()
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        let executable = executable
            .as_ref()
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);

        let stem = if let Some(path) = target.to_path() {
            PathBuf::from(path.file_name().unwrap())
        } else {
            PathBuf::from(module)
        };
        let default_base = Path::new("lab").join(stem);

        let executable = match executable {
            Some(path) => Self::binary(path),
            None => match &code {
                Some(path) => {
                    let mut binary = path.clone();
                    binary.set_extension("");
                    if binary.as_os_str().is_empty() {
                        PathBuf::from(module)
                    } else {
                        Self::binary(binary)
                    }
                }
                None => Self::binary(default_base),
            },
        };

        let code = match code {
            Some(path) => path,
            None => {
                let mut code = executable.clone();
                code.set_extension("ll");
                code
            }
        };

        (code, executable)
    }

    pub fn link(code: &Path, binary: &Path, bootstrap: bool) -> Result<()> {
        if let Some(parent) = binary.parent() {
            if !parent.as_os_str().is_empty() {
                create_dir_all(parent)?;
            }
        }

        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Some(explicit) = Self::env_path(&["AXO_CLANG_PATH", "CC"]) {
            candidates.push(explicit);
        }
        if let Some(clang) = Self::from_prefix() {
            candidates.push(clang);
        }
        candidates.push(PathBuf::from("clang-19"));
        candidates.push(PathBuf::from("clang"));
        candidates.push(PathBuf::from("cc"));

        let mut failures: Vec<String> = Vec::new();

        for candidate in candidates {
            if !Self::accepts(&candidate) {
                continue;
            }

            match Self::compile(&candidate, code, binary, bootstrap) {
                Ok(()) => return Ok(()),
                Err(error) => failures.push(format!("{}: {}", candidate.display(), error)),
            }
        }

        if !failures.is_empty() {
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "failed to link generated code with available compilers:\n{}",
                    failures.join("\n")
                ),
            ));
        }

        Err(Error::new(
            ErrorKind::Other,
            "no compiler frontend found (`clang` not available). Install clang to link generated code.",
        ))
    }

    pub fn run(binary: &Path) -> Result<()> {
        let resolved = canonicalize(binary).unwrap_or_else(|_| {
            if binary.is_relative() {
                Path::new(".").join(binary)
            } else {
                binary.to_path_buf()
            }
        });

        let status = Command::new(&resolved).status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::Other,
                format!("binary exited with status: {}", status),
            ))
        }
    }
}
