//! Vfs - 仮想ファイルシステム（インメモリ）。
//!
//! 方針:
//! - メモリ上のツリー構造を基本にし、YAML スナップショットへ保存できる。
//! - パスは Unix 風（`/` 区切り、絶対パスのみ）。
//! - ルート `/` は常に存在。
//!
//! ノード種別:
//! - Dir: 子ノードの Vec（順序保持）
//! - File: テキスト内容（String）
//!
//! スレッド安全性は今段階では不要（シングルスレッド駆動）。

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// ファイルシステムノード。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Node {
    Dir { children: BTreeMap<String, Node> },
    File { content: String },
}

impl Node {
    pub fn is_dir(&self) -> bool {
        matches!(self, Node::Dir { .. })
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Node::File { .. })
    }

    pub fn as_dir(&self) -> Option<&BTreeMap<String, Node>> {
        match self {
            Node::Dir { children } => Some(children),
            _ => None,
        }
    }

    pub fn as_dir_mut(&mut self) -> Option<&mut BTreeMap<String, Node>> {
        match self {
            Node::Dir { children } => Some(children),
            _ => None,
        }
    }

    pub fn as_file_content(&self) -> Option<&str> {
        match self {
            Node::File { content } => Some(content),
            _ => None,
        }
    }
}

/// 仮想ファイルシステム。
#[derive(Clone, Deserialize, Serialize)]
pub struct Vfs {
    root: Node,
}

/// VFS 操作のエラー。
#[derive(Debug)]
pub enum VfsError {
    NotFound,
    NotADirectory,
    NotAFile,
    AlreadyExists,
    /// ルートは削除/変更不可。
    RootProtected,
}

impl std::fmt::Display for VfsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VfsError::NotFound => write!(f, "not found"),
            VfsError::NotADirectory => write!(f, "not a directory"),
            VfsError::NotAFile => write!(f, "not a file"),
            VfsError::AlreadyExists => write!(f, "already exists"),
            VfsError::RootProtected => write!(f, "root is protected"),
        }
    }
}

impl Vfs {
    pub fn new() -> Self {
        Vfs {
            root: Node::Dir {
                children: BTreeMap::new(),
            },
        }
    }

    /// サンプル内容で初期化した Vfs を返す。
    pub fn with_samples() -> Self {
        let mut vfs = Self::new();

        // README
        let _ = vfs.create_file(
            "/README.txt",
            "# Harbour OS\n\n\
             Welcome to Harbour OS.\n\
             Files are stored in the Harbour VFS snapshot.\n\
             Use TextEditor and Files to write notes from the console.\n",
        );

        // notes/ ディレクトリ + サンプル
        let _ = vfs.create_dir("/notes");
        let _ = vfs.create_file(
            "/notes/welcome.txt",
            "First note.\n\n\
             Try creating files from the TextEditor (Save).\n\
             Open files from the File Browser.\n",
        );
        let _ = vfs.create_file(
            "/notes/todo.txt",
            "- [ ] File browser\n\
             - [ ] TextEditor save/open\n\
             - [ ] Japanese IME\n\
             - [ ] Chinese IME\n",
        );

        // docs/ ディレクトリ + サンプル
        let _ = vfs.create_dir("/docs");
        let _ = vfs.create_file(
            "/docs/manual.txt",
            "Harbour OS Manual\n\
             =================\n\n\
             ESC: quit current app (with confirm)\n\
             F11: toggle fullscreen\n\
             Ctrl+Q: shutdown\n",
        );

        vfs
    }

    /// 保存済みスナップショットを読み込む。存在しない場合はサンプル内容で初期化する。
    pub fn load_or_samples(path: impl AsRef<Path>) -> Self {
        match Self::load_from_file(path.as_ref()) {
            Ok(vfs) => vfs,
            Err(e) => {
                eprintln!("[vfs] using sample filesystem: {}", e);
                Self::with_samples()
            }
        }
    }

    /// YAML スナップショットから VFS を読み込む。
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
        let vfs: Vfs =
            serde_yaml::from_str(&content).map_err(|e| format!("{}: {}", path.display(), e))?;
        Ok(vfs)
    }

    /// VFS 全体を YAML スナップショットとして保存する。
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| format!("{}: {}", parent.display(), e))?;
            }
        }
        let content = serde_yaml::to_string(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| format!("{}: {}", path.display(), e))
    }

    /// VFS 内のファイル/ディレクトリをホスト側ディレクトリへ書き出す。
    pub fn export_to_host(
        &self,
        vfs_path: &str,
        host_root: impl AsRef<Path>,
    ) -> Result<(), String> {
        let node = self.get(vfs_path).map_err(|e| e.to_string())?;
        let host_root = host_root.as_ref();
        let relative = vfs_path.trim_matches('/');
        let out_path = if relative.is_empty() {
            host_root.to_path_buf()
        } else {
            host_root.join(relative)
        };
        export_node(node, &out_path)
    }

    /// ホスト側ディレクトリから UTF-8 テキストファイルを VFS に取り込む。
    pub fn import_host_dir(
        &mut self,
        host_root: impl AsRef<Path>,
        vfs_dest: &str,
    ) -> Result<usize, String> {
        let host_root = host_root.as_ref();
        if !host_root.is_dir() {
            return Err(format!("{} is not a directory", host_root.display()));
        }
        self.ensure_dir(vfs_dest).map_err(|e| e.to_string())?;
        import_dir_into_vfs(self, host_root, vfs_dest)
    }

    /// パスをコンポーネントに分割。`/` や `/a/b` → ["a","b"]。
    /// 空や `.` はスキップ。`..` は未対応（簡易FS）。
    fn split_path(path: &str) -> Vec<String> {
        path.split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .map(|s| s.to_string())
            .collect()
    }

    /// パスからノードを取得。
    pub fn get(&self, path: &str) -> Result<&Node, VfsError> {
        let comps = Self::split_path(path);
        if comps.is_empty() {
            return Ok(&self.root);
        }
        let mut cur = &self.root;
        for c in &comps {
            cur = cur
                .as_dir()
                .and_then(|ch| ch.get(c))
                .ok_or(VfsError::NotFound)?;
        }
        Ok(cur)
    }

    /// パスから mutable ノードを取得。
    fn get_mut(&mut self, path: &str) -> Result<&mut Node, VfsError> {
        let comps = Self::split_path(path);
        if comps.is_empty() {
            return Ok(&mut self.root);
        }
        let mut cur = &mut self.root;
        for c in &comps {
            cur = cur
                .as_dir_mut()
                .and_then(|ch| ch.get_mut(c))
                .ok_or(VfsError::NotFound)?;
        }
        Ok(cur)
    }

    /// 親ディレクトリと最終コンポーネント（名前）を分解。
    fn split_parent(path: &str) -> Result<(Vec<String>, String), VfsError> {
        let mut comps = Self::split_path(path);
        if comps.is_empty() {
            return Err(VfsError::RootProtected);
        }
        let name = comps.pop().unwrap();
        Ok((comps, name))
    }

    /// 親ディレクトリへの mutable 参照を取得。
    fn get_parent_mut(&mut self, path: &str) -> Result<(&mut Node, String), VfsError> {
        let (parent_comps, name) = Self::split_parent(path)?;
        let mut cur = &mut self.root;
        for c in &parent_comps {
            cur = cur
                .as_dir_mut()
                .and_then(|ch| ch.get_mut(c))
                .ok_or(VfsError::NotFound)?;
            if !cur.is_dir() {
                return Err(VfsError::NotADirectory);
            }
        }
        Ok((cur, name))
    }

    /// ファイルを作成。
    pub fn create_file(&mut self, path: &str, content: &str) -> Result<(), VfsError> {
        let (parent, name) = self.get_parent_mut(path)?;
        let children = parent.as_dir_mut().ok_or(VfsError::NotADirectory)?;
        if children.contains_key(&name) {
            return Err(VfsError::AlreadyExists);
        }
        children.insert(
            name,
            Node::File {
                content: content.to_string(),
            },
        );
        Ok(())
    }

    /// ディレクトリを作成。
    pub fn create_dir(&mut self, path: &str) -> Result<(), VfsError> {
        let (parent, name) = self.get_parent_mut(path)?;
        let children = parent.as_dir_mut().ok_or(VfsError::NotADirectory)?;
        if children.contains_key(&name) {
            return Err(VfsError::AlreadyExists);
        }
        children.insert(
            name,
            Node::Dir {
                children: BTreeMap::new(),
            },
        );
        Ok(())
    }

    /// ディレクトリがなければ親ごと作成する。
    pub fn ensure_dir(&mut self, path: &str) -> Result<(), VfsError> {
        let comps = Self::split_path(path);
        let mut cur_path = String::new();
        for comp in comps {
            cur_path.push('/');
            cur_path.push_str(&comp);
            match self.get(&cur_path) {
                Ok(node) if node.is_dir() => {}
                Ok(_) => return Err(VfsError::NotADirectory),
                Err(VfsError::NotFound) => self.create_dir(&cur_path)?,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// ファイル内容を読む。
    pub fn read_file(&self, path: &str) -> Result<&str, VfsError> {
        let node = self.get(path)?;
        node.as_file_content().ok_or(VfsError::NotAFile)
    }

    /// ファイル内容を上書き。存在しなければ作成。
    pub fn write_file(&mut self, path: &str, content: &str) -> Result<(), VfsError> {
        // 既存なら上書き
        if let Ok(node) = self.get_mut(path) {
            if let Node::File { content: c } = node {
                *c = content.to_string();
                return Ok(());
            }
            return Err(VfsError::NotAFile);
        }
        // 新規作成
        self.create_file(path, content)
    }

    /// ファイル・ディレクトリを削除。ルートは不可。
    pub fn remove(&mut self, path: &str) -> Result<(), VfsError> {
        let (parent, name) = self.get_parent_mut(path)?;
        let children = parent.as_dir_mut().ok_or(VfsError::NotADirectory)?;
        if children.remove(&name).is_none() {
            return Err(VfsError::NotFound);
        }
        Ok(())
    }

    /// 同じ親ディレクトリ内でファイル/ディレクトリ名を変更する。
    pub fn rename(&mut self, path: &str, new_name: &str) -> Result<String, VfsError> {
        if new_name.is_empty() || new_name.contains('/') || new_name == "." || new_name == ".." {
            return Err(VfsError::NotFound);
        }
        let (parent, old_name) = self.get_parent_mut(path)?;
        let children = parent.as_dir_mut().ok_or(VfsError::NotADirectory)?;
        if !children.contains_key(&old_name) {
            return Err(VfsError::NotFound);
        }
        if children.contains_key(new_name) {
            return Err(VfsError::AlreadyExists);
        }
        let node = children.remove(&old_name).ok_or(VfsError::NotFound)?;
        children.insert(new_name.to_string(), node);
        let parent_path = parent_path(path);
        Ok(join_vfs_path(&parent_path, new_name))
    }

    /// ディレクトリの子要素一覧（名前と種別）。ソート済み（BTreeMap 順）。
    pub fn list_dir(&self, path: &str) -> Result<Vec<(String, bool)>, VfsError> {
        let node = self.get(path)?;
        let children = node.as_dir().ok_or(VfsError::NotADirectory)?;
        Ok(children
            .iter()
            .map(|(name, n)| (name.clone(), n.is_dir()))
            .collect())
    }
}

fn export_node(node: &Node, out_path: &Path) -> Result<(), String> {
    match node {
        Node::File { content } => {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("{}: {}", parent.display(), e))?;
            }
            fs::write(out_path, content).map_err(|e| format!("{}: {}", out_path.display(), e))
        }
        Node::Dir { children } => {
            fs::create_dir_all(out_path).map_err(|e| format!("{}: {}", out_path.display(), e))?;
            for (name, child) in children {
                export_node(child, &out_path.join(name))?;
            }
            Ok(())
        }
    }
}

fn import_dir_into_vfs(vfs: &mut Vfs, host_dir: &Path, vfs_dest: &str) -> Result<usize, String> {
    let mut imported = 0usize;
    let entries = fs::read_dir(host_dir).map_err(|e| format!("{}: {}", host_dir.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let Some(name) = safe_host_name(&path) else {
            continue;
        };
        let vfs_path = join_vfs_path(vfs_dest, &name);
        if path.is_dir() {
            vfs.ensure_dir(&vfs_path).map_err(|e| e.to_string())?;
            imported += import_dir_into_vfs(vfs, &path, &vfs_path)?;
        } else if path.is_file() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    vfs.write_file(&vfs_path, &content)
                        .map_err(|e| e.to_string())?;
                    imported += 1;
                }
                Err(e) => eprintln!("[vfs] skipped import {}: {}", path.display(), e),
            }
        }
    }
    Ok(imported)
}

fn safe_host_name(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    if name.starts_with('.') || name.contains('/') || name.contains('\\') {
        None
    } else {
        Some(name.to_string())
    }
}

fn join_vfs_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", parent.trim_end_matches('/'), name)
    }
}

fn parent_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(0) | None => "/".to_string(),
        Some(pos) => trimmed[..pos].to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_read() {
        let mut vfs = Vfs::new();
        vfs.create_file("/hello.txt", "hi").unwrap();
        assert_eq!(vfs.read_file("/hello.txt").unwrap(), "hi");
    }

    #[test]
    fn nested_dirs() {
        let mut vfs = Vfs::new();
        vfs.create_dir("/a").unwrap();
        vfs.create_dir("/a/b").unwrap();
        vfs.create_file("/a/b/c.txt", "deep").unwrap();
        assert_eq!(vfs.read_file("/a/b/c.txt").unwrap(), "deep");
    }

    #[test]
    fn list_and_remove() {
        let vfs = Vfs::with_samples();
        let root = vfs.list_dir("/").unwrap();
        assert!(root.iter().any(|(n, _)| n == "README.txt"));
        assert!(root.iter().any(|(n, is_dir)| n == "notes" && *is_dir));

        let mut vfs = vfs;
        vfs.remove("/README.txt").unwrap();
        assert!(vfs.read_file("/README.txt").is_err());
    }

    #[test]
    fn write_overwrites() {
        let mut vfs = Vfs::new();
        vfs.write_file("/f.txt", "v1").unwrap();
        vfs.write_file("/f.txt", "v2").unwrap();
        assert_eq!(vfs.read_file("/f.txt").unwrap(), "v2");
    }

    #[test]
    fn save_and_load_snapshot() {
        let mut path = std::env::temp_dir();
        path.push(format!("harbour_vfs_test_{}.yaml", std::process::id()));

        let mut vfs = Vfs::new();
        vfs.create_dir("/notes").unwrap();
        vfs.write_file("/notes/persist.txt", "still here").unwrap();
        vfs.save_to_file(&path).unwrap();

        let loaded = Vfs::load_from_file(&path).unwrap();
        assert_eq!(
            loaded.read_file("/notes/persist.txt").unwrap(),
            "still here"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn export_and_import_host_directory() {
        let base = std::env::temp_dir().join(format!("harbour_vfs_io_test_{}", std::process::id()));
        let export_root = base.join("export");
        let import_root = base.join("import");
        std::fs::create_dir_all(&import_root).unwrap();
        std::fs::write(import_root.join("host.txt"), "from host").unwrap();
        std::fs::create_dir_all(import_root.join("sub")).unwrap();
        std::fs::write(import_root.join("sub").join("deep.txt"), "deep host").unwrap();

        let mut vfs = Vfs::new();
        vfs.create_dir("/notes").unwrap();
        vfs.write_file("/notes/vfs.txt", "from vfs").unwrap();
        vfs.export_to_host("/notes", &export_root).unwrap();
        assert_eq!(
            std::fs::read_to_string(export_root.join("notes").join("vfs.txt")).unwrap(),
            "from vfs"
        );

        let count = vfs.import_host_dir(&import_root, "/imports").unwrap();
        assert_eq!(count, 2);
        assert_eq!(vfs.read_file("/imports/host.txt").unwrap(), "from host");
        assert_eq!(vfs.read_file("/imports/sub/deep.txt").unwrap(), "deep host");

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn rename_entry() {
        let mut vfs = Vfs::new();
        vfs.create_dir("/notes").unwrap();
        vfs.write_file("/notes/old.txt", "rename me").unwrap();
        let new_path = vfs.rename("/notes/old.txt", "new.txt").unwrap();
        assert_eq!(new_path, "/notes/new.txt");
        assert!(vfs.read_file("/notes/old.txt").is_err());
        assert_eq!(vfs.read_file("/notes/new.txt").unwrap(), "rename me");
    }
}
