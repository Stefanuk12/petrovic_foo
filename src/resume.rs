//! Serves `resume.tex` as a PDF, compiling it with latexmk on demand.
//!
//! Builds are keyed by the SHA-256 of the source, so an unchanged `.tex` is served from memory
//! (or from the on-disk cache after a restart) and only an edit triggers a recompile.

use std::path::PathBuf;

use axum::{
    body::Bytes,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};
use tokio::{process::Command, sync::Mutex};

pub struct Resume {
    tex: PathBuf,
    cache_dir: PathBuf,
    latest: Mutex<Option<(String, Bytes)>>,
}

impl Resume {
    /// Reads `resume.tex` from the site root, where cargo-leptos copies it from `public/`.
    pub fn from_env(site_root: &str) -> Self {
        let tex = std::env::var_os("RESUME_TEX")
            .map_or_else(|| PathBuf::from(site_root).join("resume.tex"), PathBuf::from);
        let cache_dir = std::env::var_os("RESUME_CACHE_DIR").map_or_else(
            || std::env::temp_dir().join("petrovic_foo-resume"),
            PathBuf::from,
        );

        Self {
            tex,
            cache_dir,
            latest: Mutex::new(None),
        }
    }

    pub async fn serve(&self, req_headers: HeaderMap) -> Response {
        let (hash, pdf) = match self.pdf().await {
            Ok(built) => built,
            Err(err) => {
                leptos::logging::error!("resume: {err}");
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build resume").into_response();
            }
        };

        let etag = HeaderValue::from_str(&format!("\"{hash}\"")).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(header::ETAG, etag.clone());
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));

        if req_headers.get(header::IF_NONE_MATCH) == Some(&etag) {
            return (StatusCode::NOT_MODIFIED, headers).into_response();
        }

        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/pdf"));
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_static("inline; filename=\"Stefan-Petrovic-Resume.pdf\""),
        );
        
        (headers, pdf).into_response()
    }

    async fn pdf(&self) -> Result<(String, Bytes), String> {
        let source = tokio::fs::read(&self.tex)
            .await
            .map_err(|e| format!("reading {}: {e}", self.tex.display()))?;
        let hash = format!("{:x}", Sha256::digest(&source));

        let mut latest = self.latest.lock().await;
        if let Some((cached, pdf)) = latest.as_ref()
            && *cached == hash
        {
            return Ok((hash, pdf.clone()));
        }

        let cached_pdf = self.cache_dir.join(format!("{hash}.pdf"));
        let pdf = match tokio::fs::read(&cached_pdf).await {
            Ok(pdf) => Bytes::from(pdf),
            Err(_) => {
                let pdf = Bytes::from(self.compile(&hash, &source).await?);
                tokio::fs::write(&cached_pdf, &pdf)
                    .await
                    .map_err(|e| format!("writing {}: {e}", cached_pdf.display()))?;
                pdf
            }
        };

        *latest = Some((hash.clone(), pdf.clone()));
        Ok((hash, pdf))
    }

    async fn compile(&self, hash: &str, source: &[u8]) -> Result<Vec<u8>, String> {
        // Compile the snapshot we hashed, not the live file, so the cache key matches the output.
        let build_dir = self.cache_dir.join(format!("build-{hash}"));
        tokio::fs::create_dir_all(&build_dir)
            .await
            .map_err(|e| format!("creating {}: {e}", build_dir.display()))?;
        tokio::fs::write(build_dir.join("resume.tex"), source)
            .await
            .map_err(|e| format!("writing source: {e}"))?;

        leptos::logging::log!("resume: compiling {}", &hash[..12]);
        let output = Command::new("latexmk")
            .args(["-pdf", "-interaction=nonstopmode", "-halt-on-error", "resume.tex"])
            .current_dir(&build_dir)
            .output()
            .await
            .map_err(|e| format!("running latexmk: {e}"))?;

        let pdf = if output.status.success() {
            tokio::fs::read(build_dir.join("resume.pdf"))
                .await
                .map_err(|e| format!("reading compiled pdf: {e}"))
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let tail: Vec<_> = stdout.lines().rev().take(20).collect();
            Err(format!(
                "latexmk failed:\n{}",
                tail.into_iter().rev().collect::<Vec<_>>().join("\n")
            ))
        };
        let _ = tokio::fs::remove_dir_all(&build_dir).await;
        pdf
    }
}
