use super::*;
#[test]
fn retired_entry_preserves_bytes_and_refuses_old_append_open() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("graph.log");
    std::fs::write(&path, b"original bytes\n").unwrap();
    let mut source = LegacyFile::load(&path, "signal-log/1", 1024)
        .unwrap()
        .unwrap();
    assert!(LegacyFile::load(&path, "signal-log/1", 1024).is_err());
    source.retire().unwrap();
    assert!(source.is_retired());
    assert!(OpenOptions::new().append(true).open(&path).is_err());
    assert_eq!(
        std::fs::read(path.join(SOURCE)).unwrap(),
        b"original bytes\n"
    );
    drop(source);
    let mut source = LegacyFile::load(&path, "signal-log/1", 1024)
        .unwrap()
        .unwrap();
    assert!(source.is_retired());
    source.retire().unwrap();
    drop(source);
    assert!(LegacyFile::load(&path, "different-kind/1", 1024).is_err());
    std::fs::write(path.join(SOURCE), b"changed").unwrap();
    assert!(LegacyFile::load(&path, "signal-log/1", 1024).is_err());
}
#[test]
fn each_exposed_filesystem_boundary_resumes_the_same_source() {
    for moved in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("particles.jsonl");
        std::fs::write(&path, b"source\n").unwrap();
        let original = LegacyFile::load(&path, "text-jsonl/1", 1024)
            .unwrap()
            .unwrap();
        std::fs::create_dir(&original.stage).unwrap();
        std::fs::write(
            original.stage.join(MARKER),
            serde_json::to_vec(&original.manifest).unwrap(),
        )
        .unwrap();
        if moved {
            rename_new(&path, &original.stage.join(SOURCE)).unwrap();
        }
        drop(original);
        let mut resumed = LegacyFile::load(&path, "text-jsonl/1", 1024)
            .unwrap()
            .unwrap();
        assert_eq!(resumed.bytes(), b"source\n");
        resumed.retire().unwrap();
        assert!(resumed.is_retired());
        assert_eq!(std::fs::read(path.join(SOURCE)).unwrap(), b"source\n");
    }
}
#[test]
fn no_replace_rename_preserves_a_competing_path_and_its_contents() {
    let root = tempfile::tempdir().unwrap();
    let from = root.path().join("from");
    let to = root.path().join("to");
    std::fs::create_dir(&from).unwrap();
    std::fs::create_dir(&to).unwrap();
    std::fs::write(to.join("sentinel"), b"competitor").unwrap();
    assert!(rename_new(&from, &to).is_err());
    assert_eq!(std::fs::read(to.join("sentinel")).unwrap(), b"competitor");
    assert!(from.is_dir());
}
