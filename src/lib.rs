pub mod browser;
pub mod document;
pub mod markdown;

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs::{self, File, Permissions};
use std::io::{self, Write};

pub fn validate_output_file(path: &Utf8Path) -> Result<()> {
    output_permissions(path).map(|_| ())
}

fn output_permissions(path: &Utf8Path) -> Result<Option<Permissions>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                bail!("output path is a symlink; use a direct file path instead: {path}");
            }
            if !metadata.is_file() {
                bail!("output path is not a regular file: {path}");
            }
            let permissions = metadata.permissions();
            if permissions.readonly() {
                bail!("output file is read-only: {path}");
            }
            Ok(Some(permissions))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to inspect output path {path}")),
    }
}

pub fn write_output_atomic(path: &Utf8Path, bytes: &[u8]) -> Result<()> {
    write_output_atomic_with(path, |file| file.write_all(bytes))
}

fn write_output_atomic_with(
    path: &Utf8Path,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> Result<()> {
    let permissions = output_permissions(path)?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_str().is_empty())
        .unwrap_or_else(|| Utf8Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create output directory {parent}"))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temporary output in {parent}"))?;
    if permissions.is_some() {
        validate_replacement_security(path, &temporary)?;
    }
    write(temporary.as_file_mut()).with_context(|| format!("failed to write {path}"))?;
    temporary
        .as_file_mut()
        .flush()
        .with_context(|| format!("failed to flush {path}"))?;
    if let Some(permissions) = permissions {
        temporary
            .as_file()
            .set_permissions(permissions)
            .with_context(|| format!("failed to preserve permissions for {path}"))?;
    }
    temporary
        .as_file()
        .sync_all()
        .with_context(|| format!("failed to sync {path}"))?;
    temporary
        .persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("failed to replace output file {path}"))?;
    Ok(())
}

#[cfg(unix)]
fn validate_replacement_security(
    path: &Utf8Path,
    temporary: &tempfile::NamedTempFile,
) -> Result<()> {
    use std::os::unix::fs::MetadataExt;

    let original =
        File::open(path).with_context(|| format!("failed to inspect security for {path}"))?;
    let source = original.metadata()?;
    let replacement = temporary.as_file().metadata()?;
    if source.uid() != replacement.uid() || source.gid() != replacement.gid() {
        bail!("cannot atomically replace {path} without changing file ownership");
    }
    if source.mode() & 0o7000 != 0 {
        bail!("cannot atomically replace {path} with special permission bits");
    }
    validate_replacement_acl(path, &original, temporary)
}

#[cfg(target_os = "linux")]
fn validate_replacement_acl(
    path: &Utf8Path,
    original: &File,
    temporary: &tempfile::NamedTempFile,
) -> Result<()> {
    use xattr::FileExt;

    if original.get_xattr("system.posix_acl_access")?.is_some()
        || temporary
            .as_file()
            .get_xattr("system.posix_acl_access")?
            .is_some()
        || original.get_xattr("security.selinux")?
            != temporary.as_file().get_xattr("security.selinux")?
    {
        bail!("cannot atomically replace {path} without changing file access controls");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn validate_replacement_acl(
    path: &Utf8Path,
    _original: &File,
    temporary: &tempfile::NamedTempFile,
) -> Result<()> {
    if !exacl::getfacl(path, None)?.is_empty()
        || !exacl::getfacl(temporary.path(), None)?.is_empty()
    {
        bail!("cannot atomically replace {path} without changing file access controls");
    }
    Ok(())
}

#[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
fn validate_replacement_acl(
    path: &Utf8Path,
    _original: &File,
    _temporary: &tempfile::NamedTempFile,
) -> Result<()> {
    bail!("cannot verify file access controls for atomic replacement of {path}");
}

#[cfg(windows)]
fn validate_replacement_security(
    path: &Utf8Path,
    temporary: &tempfile::NamedTempFile,
) -> Result<()> {
    use std::os::windows::fs::MetadataExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_ENCRYPTED;

    let original =
        File::open(path).with_context(|| format!("failed to inspect security for {path}"))?;
    if original.metadata()?.file_attributes() & FILE_ATTRIBUTE_ENCRYPTED != 0 {
        bail!("cannot atomically replace encrypted output file {path}");
    }
    if security_descriptor(&original)? != security_descriptor(temporary.as_file())? {
        bail!("cannot atomically replace {path} without changing file access controls");
    }
    Ok(())
}

#[cfg(windows)]
fn security_descriptor(file: &File) -> io::Result<Vec<u32>> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
    use windows_sys::Win32::Security::{
        GetKernelObjectSecurity, ATTRIBUTE_SECURITY_INFORMATION, DACL_SECURITY_INFORMATION,
        GROUP_SECURITY_INFORMATION, LABEL_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
        SCOPE_SECURITY_INFORMATION,
    };

    let information = OWNER_SECURITY_INFORMATION
        | GROUP_SECURITY_INFORMATION
        | DACL_SECURITY_INFORMATION
        | LABEL_SECURITY_INFORMATION
        | ATTRIBUTE_SECURITY_INFORMATION
        | SCOPE_SECURITY_INFORMATION;
    let mut needed = 0;
    let result = unsafe {
        GetKernelObjectSecurity(
            file.as_raw_handle(),
            information,
            std::ptr::null_mut(),
            0,
            &mut needed,
        )
    };
    let error = io::Error::last_os_error();
    if result != 0 || error.raw_os_error() != Some(ERROR_INSUFFICIENT_BUFFER as i32) {
        return Err(error);
    }
    let mut descriptor = vec![0u32; (needed as usize).div_ceil(4)];
    let result = unsafe {
        GetKernelObjectSecurity(
            file.as_raw_handle(),
            information,
            descriptor.as_mut_ptr().cast(),
            (descriptor.len() * 4) as u32,
            &mut needed,
        )
    };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(descriptor)
}

pub fn default_output_path(input: &Utf8Path) -> Result<Utf8PathBuf> {
    if input.file_name().is_none() {
        bail!("input path must point to a Markdown file");
    }

    Ok(input.with_extension("pdf"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_output_preserves_existing_file_on_partial_write_failure() {
        let directory = tempfile::tempdir().unwrap();
        let path = Utf8Path::from_path(directory.path())
            .unwrap()
            .join("out.pdf");
        fs::write(&path, b"Original PDF").unwrap();

        let error = write_output_atomic_with(&path, |file| {
            file.write_all(b"Partial new PDF")?;
            Err(io::Error::other("injected write failure"))
        })
        .unwrap_err();

        assert!(format!("{error:#}").contains("injected write failure"));
        assert_eq!(fs::read(&path).unwrap(), b"Original PDF");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn atomic_output_cleans_up_failed_new_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = Utf8Path::from_path(directory.path())
            .unwrap()
            .join("out.pdf");

        assert!(write_output_atomic_with(&path, |file| {
            file.write_all(b"Partial PDF")?;
            Err(io::Error::other("injected write failure"))
        })
        .is_err());

        assert!(!path.exists());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
    }

    #[test]
    fn atomic_output_replaces_only_the_destination_hard_link() {
        let directory = tempfile::tempdir().unwrap();
        let parent = Utf8Path::from_path(directory.path()).unwrap();
        let path = parent.join("out.pdf");
        let backup = parent.join("backup.pdf");
        fs::write(&path, b"Original PDF").unwrap();
        fs::hard_link(&path, &backup).unwrap();

        write_output_atomic(&path, b"New PDF").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"New PDF");
        assert_eq!(fs::read(&backup).unwrap(), b"Original PDF");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[test]
    fn atomic_output_cleans_up_on_commit_failure() {
        let directory = tempfile::tempdir().unwrap();
        let path = Utf8Path::from_path(directory.path())
            .unwrap()
            .join("out.pdf");

        let error = write_output_atomic_with(&path, |file| {
            file.write_all(b"New PDF")?;
            fs::create_dir(&path)
        })
        .unwrap_err();

        assert!(error.to_string().contains("failed to replace output file"));
        assert!(path.is_dir());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn atomic_output_preserves_permissions_and_rejects_read_only_files() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let path = Utf8Path::from_path(directory.path())
            .unwrap()
            .join("out.pdf");
        fs::write(&path, b"Original PDF").unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o640)).unwrap();

        write_output_atomic(&path, b"New PDF").unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );

        fs::set_permissions(&path, Permissions::from_mode(0o440)).unwrap();
        assert!(write_output_atomic(&path, b"Unexpected PDF")
            .unwrap_err()
            .to_string()
            .contains("read-only"));
        assert_eq!(fs::read(&path).unwrap(), b"New PDF");
    }

    #[test]
    fn atomic_output_rejects_read_only_destination() {
        let directory = tempfile::tempdir().unwrap();
        let path = Utf8Path::from_path(directory.path())
            .unwrap()
            .join("out.pdf");
        fs::write(&path, b"Original PDF").unwrap();
        let permissions = fs::metadata(&path).unwrap().permissions();
        let mut read_only = permissions.clone();
        read_only.set_readonly(true);
        fs::set_permissions(&path, read_only).unwrap();

        let result = write_output_atomic(&path, b"Unexpected PDF");
        fs::set_permissions(&path, permissions).unwrap();

        assert!(result.unwrap_err().to_string().contains("read-only"));
        assert_eq!(fs::read(&path).unwrap(), b"Original PDF");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn atomic_output_rejects_extended_acl() {
        let directory = tempfile::tempdir().unwrap();
        let path = Utf8Path::from_path(directory.path())
            .unwrap()
            .join("out.pdf");
        fs::write(&path, b"Original PDF").unwrap();
        assert!(std::process::Command::new("chmod")
            .args(["+a", "everyone allow read"])
            .arg(&path)
            .status()
            .unwrap()
            .success());

        assert!(write_output_atomic(&path, b"Unexpected PDF")
            .unwrap_err()
            .to_string()
            .contains("access controls"));
        assert_eq!(fs::read(&path).unwrap(), b"Original PDF");
        assert!(!exacl::getfacl(&path, None).unwrap().is_empty());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn atomic_output_rejects_extended_acl() {
        use xattr::FileExt;

        let directory = tempfile::tempdir().unwrap();
        let path = Utf8Path::from_path(directory.path())
            .unwrap()
            .join("out.pdf");
        fs::write(&path, b"Original PDF").unwrap();
        let mut acl = 2u32.to_le_bytes().to_vec();
        for (tag, permissions, identifier) in [
            (1u16, 6u16, u32::MAX),
            (2, 4, 65534),
            (4, 0, u32::MAX),
            (16, 4, u32::MAX),
            (32, 0, u32::MAX),
        ] {
            acl.extend_from_slice(&tag.to_le_bytes());
            acl.extend_from_slice(&permissions.to_le_bytes());
            acl.extend_from_slice(&identifier.to_le_bytes());
        }
        let file = File::open(&path).unwrap();
        file.set_xattr("system.posix_acl_access", &acl).unwrap();

        assert!(write_output_atomic(&path, b"Unexpected PDF")
            .unwrap_err()
            .to_string()
            .contains("access controls"));
        assert_eq!(fs::read(&path).unwrap(), b"Original PDF");
        assert_eq!(
            file.get_xattr("system.posix_acl_access").unwrap(),
            Some(acl)
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn atomic_output_allows_parent_directory_symlink() {
        let directory = tempfile::tempdir().unwrap();
        let parent = Utf8Path::from_path(directory.path()).unwrap();
        let actual = parent.join("actual");
        let link = parent.join("link");
        fs::create_dir(&actual).unwrap();
        std::os::unix::fs::symlink(&actual, &link).unwrap();
        let path = link.join("out.pdf");

        write_output_atomic(&path, b"First PDF").unwrap();
        write_output_atomic(&path, b"New PDF").unwrap();

        assert_eq!(fs::read(actual.join("out.pdf")).unwrap(), b"New PDF");
        assert_eq!(fs::read_dir(&actual).unwrap().count(), 1);
    }

    #[cfg(windows)]
    #[test]
    fn atomic_output_rejects_encrypted_destination() {
        use std::os::windows::ffi::OsStrExt;
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::{EncryptFileW, FILE_ATTRIBUTE_ENCRYPTED};

        let directory = tempfile::tempdir().unwrap();
        let path = Utf8Path::from_path(directory.path())
            .unwrap()
            .join("out.pdf");
        fs::write(&path, b"Original PDF").unwrap();
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        assert_ne!(
            unsafe { EncryptFileW(wide.as_ptr()) },
            0,
            "{}",
            io::Error::last_os_error()
        );

        assert!(write_output_atomic(&path, b"Unexpected PDF")
            .unwrap_err()
            .to_string()
            .contains("encrypted"));
        assert_eq!(fs::read(&path).unwrap(), b"Original PDF");
        assert_ne!(
            fs::metadata(&path).unwrap().file_attributes() & FILE_ATTRIBUTE_ENCRYPTED,
            0
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn defaults_output_path_to_pdf_extension() {
        let output = default_output_path(Utf8Path::new("docs/guide.md")).unwrap();

        assert_eq!(output, Utf8PathBuf::from("docs/guide.pdf"));
    }
}
