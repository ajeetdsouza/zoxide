use zoxide_engine::Store;

#[test]
fn test_add() {
    let path = "/foo/bar";
    let now = 946684800;

    let data_dir = tempfile::tempdir().unwrap();
    {
        let mut store = Store::open(data_dir.path()).unwrap();
        store.add(path, now);
        store.add(path, now);
    }
    {
        let store = Store::open(data_dir.path()).unwrap();
        assert_eq!(store.dirs.len(), 1);

        let dir = &store.dirs[0];
        assert_eq!(dir.path, path);
        assert_eq!(dir.last_accessed, now);
    }
}

#[test]
fn test_remove() {
    let path = "/foo/bar";
    let now = 946684800;

    let data_dir = tempfile::tempdir().unwrap();
    {
        let mut store = Store::open(data_dir.path()).unwrap();
        store.add(path, now);
    }
    {
        let mut store = Store::open(data_dir.path()).unwrap();
        assert!(store.remove(path));
    }
    {
        let mut store = Store::open(data_dir.path()).unwrap();
        assert!(store.dirs.is_empty());
        assert!(!store.remove(path));
    }
}
