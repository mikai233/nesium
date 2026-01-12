use std::path::{Path, PathBuf};

/// Return a default directory for storing QUIC cert/key material for the given application.
///
/// This is used by both the standalone `nesium-netd` binary and the embedded server inside
/// the Flutter client, so the path is parameterized by `app_name`.
pub fn default_quic_data_dir(app_name: &str) -> PathBuf {
    // Prefer OS-specific app data locations; fall back to current directory.
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(app_name)
            .join("quic")
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."))
            .join(app_name)
            .join("quic")
    }
}

pub fn ensure_quic_cert_pair(dir: &Path) -> anyhow::Result<(PathBuf, PathBuf)> {
    std::fs::create_dir_all(dir)?;
    let cert_path = dir.join("cert.pem");
    let key_path = dir.join("key.pem");

    if cert_path.exists() && key_path.exists() {
        return Ok((cert_path, key_path));
    }

    // Pinning mode does not rely on SAN/hostname verification, but keeping some reasonable SANs
    // makes the cert more portable if you switch to system-trust mode later.
    let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    let rcgen::CertifiedKey { cert, signing_key } =
        rcgen::generate_simple_self_signed(subject_alt_names)?;
    std::fs::write(&cert_path, cert.pem())?;
    std::fs::write(&key_path, signing_key.serialize_pem())?;

    Ok((cert_path, key_path))
}

pub fn build_quic_server_config(
    cert_path: &Path,
    key_path: &Path,
) -> anyhow::Result<quinn::ServerConfig> {
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};

    let certs: Vec<CertificateDer<'static>> =
        CertificateDer::pem_file_iter(cert_path)?.collect::<Result<Vec<_>, _>>()?;
    let key: PrivateKeyDer<'static> = PrivateKeyDer::from_pem_file(key_path)?;

    let mut server_config = quinn::ServerConfig::with_single_cert(certs, key)?;
    server_config.transport_config(std::sync::Arc::new(quinn::TransportConfig::default()));
    Ok(server_config)
}

pub fn sha256_fingerprint_from_pem(cert_path: &Path) -> anyhow::Result<String> {
    use rustls::pki_types::CertificateDer;
    use rustls::pki_types::pem::PemObject;

    let Some(cert) = CertificateDer::pem_file_iter(cert_path)?.next() else {
        anyhow::bail!("No certificate found in {}", cert_path.display());
    };
    let cert = cert?;

    let digest = ring::digest::digest(&ring::digest::SHA256, cert.as_ref());
    let hex = hex::encode(digest.as_ref());

    // Format as AA:BB:.. for readability.
    let mut out = String::with_capacity(32 * 3 - 1);
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        if i != 0 {
            out.push(':');
        }
        out.push_str(std::str::from_utf8(chunk)?);
    }
    Ok(out.to_uppercase())
}

/// Compute leaf certificate SHA-256 fingerprint, encoded as base64url (no padding).
///
/// This is shorter and copy-friendly (43 chars) while still representing the full 32 bytes.
pub fn sha256_fingerprint_base64url_from_pem(cert_path: &Path) -> anyhow::Result<String> {
    use base64::Engine as _;
    use rustls::pki_types::CertificateDer;
    use rustls::pki_types::pem::PemObject;

    let Some(cert) = CertificateDer::pem_file_iter(cert_path)?.next() else {
        anyhow::bail!("No certificate found in {}", cert_path.display());
    };
    let cert = cert?;

    let digest = ring::digest::digest(&ring::digest::SHA256, cert.as_ref());
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest.as_ref()))
}
