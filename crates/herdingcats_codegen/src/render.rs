use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostics::Diagnostic;

pub fn write_generated_module(
    out_dir: impl AsRef<Path>,
    file_name: &str,
    source: &str,
) -> Result<PathBuf, Diagnostic> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir).map_err(|err| {
        Diagnostic::io(format!(
            "failed creating output directory {}: {err}",
            out_dir.display()
        ))
    })?;

    let path = out_dir.join(file_name);
    fs::write(&path, source)
        .map_err(|err| Diagnostic::io(format!("failed writing {}: {err}", path.display())))?;
    Ok(path)
}
