mod support;
use support::*;
use zixcel_container_registry::{Hub, apply, plan};
#[test]
fn journal_symlinks_and_world_readable_directories_are_rejected() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let d = directory();
    let outside = directory();
    symlink(outside.path(), d.path().join("link")).unwrap();
    for path in [d.path().join("link"), outside.path().to_owned()] {
        if path == outside.path() {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut f = Fake::new();
        f.snapshot("one");
        f.snapshot("one");
        let mut h = Hub::new(f);
        let p = plan(&mut h, "example", &["one".into()], 100).unwrap();
        assert!(apply(&mut h, &p, 101, &digest(&p), &path).is_err());
    }
}
