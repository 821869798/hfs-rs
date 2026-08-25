//! Virtual File System — HFS2 `Tfile` inspired model.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Root,
    VirtualFolder,
    RealFolder,
    File,
    Link,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VfsNode {
    pub id: NodeId,
    pub name: String,
    pub kind: NodeKind,
    /// Real filesystem path for File/RealFolder, or URL for Link.
    pub resource: Option<PathBuf>,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub comment: String,
    pub hidden: bool,
    pub download_forbidden: bool,
}

impl VfsNode {
    pub fn is_folder(&self) -> bool {
        matches!(
            self.kind,
            NodeKind::Root | NodeKind::VirtualFolder | NodeKind::RealFolder
        )
    }

    pub fn display_name(&self) -> &str {
        if self.name.is_empty() && self.kind == NodeKind::Root {
            "Home"
        } else {
            &self.name
        }
    }
}

#[derive(Debug, Error)]
pub enum VfsError {
    #[error("node not found: {0:?}")]
    NotFound(NodeId),
    #[error("name already exists: {0}")]
    NameExists(String),
    #[error("not a folder")]
    NotAFolder,
    #[error("cannot modify root that way")]
    InvalidRootOp,
    #[error("invalid name")]
    InvalidName,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vfs {
    next_id: u64,
    root: NodeId,
    nodes: HashMap<NodeId, VfsNode>,
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

impl Vfs {
    pub fn new() -> Self {
        let root_id = NodeId(1);
        let mut nodes = HashMap::new();
        nodes.insert(
            root_id,
            VfsNode {
                id: root_id,
                name: String::new(),
                kind: NodeKind::Root,
                resource: None,
                parent: None,
                children: Vec::new(),
                comment: String::new(),
                hidden: false,
                download_forbidden: false,
            },
        );
        Self {
            next_id: 2,
            root: root_id,
            nodes,
        }
    }

    pub fn root_id(&self) -> NodeId {
        self.root
    }

    pub fn get(&self, id: NodeId) -> Option<&VfsNode> {
        self.nodes.get(&id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut VfsNode> {
        self.nodes.get_mut(&id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = &VfsNode> {
        self.nodes.values()
    }

    fn alloc_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    fn unique_child_name(&self, parent: NodeId, desired: &str) -> Result<String, VfsError> {
        let parent_node = self.get(parent).ok_or(VfsError::NotFound(parent))?;
        if !parent_node.is_folder() {
            return Err(VfsError::NotAFolder);
        }
        if desired.is_empty() || desired.contains('/') || desired.contains('\\') {
            return Err(VfsError::InvalidName);
        }
        let exists = parent_node.children.iter().any(|cid| {
            self.get(*cid)
                .map(|n| n.name.eq_ignore_ascii_case(desired))
                .unwrap_or(false)
        });
        if !exists {
            return Ok(desired.to_string());
        }
        for i in 2..10_000 {
            let candidate = format!("{desired} ({i})");
            let exists = parent_node.children.iter().any(|cid| {
                self.get(*cid)
                    .map(|n| n.name.eq_ignore_ascii_case(&candidate))
                    .unwrap_or(false)
            });
            if !exists {
                return Ok(candidate);
            }
        }
        Err(VfsError::NameExists(desired.to_string()))
    }

    pub fn add_file<P: AsRef<Path>>(
        &mut self,
        parent: NodeId,
        path: P,
    ) -> Result<NodeId, VfsError> {
        let path = path.as_ref().to_path_buf();
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .filter(|s| !s.is_empty())
            .ok_or(VfsError::InvalidName)?;
        let name = self.unique_child_name(parent, &name)?;
        let id = self.alloc_id();
        self.nodes.insert(
            id,
            VfsNode {
                id,
                name,
                kind: NodeKind::File,
                resource: Some(path),
                parent: Some(parent),
                children: Vec::new(),
                comment: String::new(),
                hidden: false,
                download_forbidden: false,
            },
        );
        self.nodes
            .get_mut(&parent)
            .ok_or(VfsError::NotFound(parent))?
            .children
            .push(id);
        Ok(id)
    }

    pub fn add_real_folder<P: AsRef<Path>>(
        &mut self,
        parent: NodeId,
        path: P,
    ) -> Result<NodeId, VfsError> {
        let path = path.as_ref().to_path_buf();
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let name = self.unique_child_name(parent, &name)?;
        let id = self.alloc_id();
        self.nodes.insert(
            id,
            VfsNode {
                id,
                name,
                kind: NodeKind::RealFolder,
                resource: Some(path),
                parent: Some(parent),
                children: Vec::new(),
                comment: String::new(),
                hidden: false,
                download_forbidden: false,
            },
        );
        self.nodes
            .get_mut(&parent)
            .ok_or(VfsError::NotFound(parent))?
            .children
            .push(id);
        Ok(id)
    }

    pub fn add_virtual_folder(&mut self, parent: NodeId, name: &str) -> Result<NodeId, VfsError> {
        let name = self.unique_child_name(parent, name)?;
        let id = self.alloc_id();
        self.nodes.insert(
            id,
            VfsNode {
                id,
                name,
                kind: NodeKind::VirtualFolder,
                resource: None,
                parent: Some(parent),
                children: Vec::new(),
                comment: String::new(),
                hidden: false,
                download_forbidden: false,
            },
        );
        self.nodes
            .get_mut(&parent)
            .ok_or(VfsError::NotFound(parent))?
            .children
            .push(id);
        Ok(id)
    }

    pub fn add_link(&mut self, parent: NodeId, name: &str, url: &str) -> Result<NodeId, VfsError> {
        let name = self.unique_child_name(parent, name)?;
        if url.trim().is_empty() {
            return Err(VfsError::InvalidName);
        }
        let id = self.alloc_id();
        self.nodes.insert(
            id,
            VfsNode {
                id,
                name,
                kind: NodeKind::Link,
                resource: Some(PathBuf::from(url)),
                parent: Some(parent),
                children: Vec::new(),
                comment: String::new(),
                hidden: false,
                download_forbidden: false,
            },
        );
        self.nodes
            .get_mut(&parent)
            .ok_or(VfsError::NotFound(parent))?
            .children
            .push(id);
        Ok(id)
    }

    pub fn remove(&mut self, id: NodeId) -> Result<(), VfsError> {
        if id == self.root {
            return Err(VfsError::InvalidRootOp);
        }
        let node = self.nodes.get(&id).ok_or(VfsError::NotFound(id))?.clone();
        // Remove children first (depth-first).
        for child in node.children.clone() {
            let _ = self.remove(child);
        }
        if let Some(parent) = node.parent {
            if let Some(p) = self.nodes.get_mut(&parent) {
                p.children.retain(|c| *c != id);
            }
        }
        self.nodes.remove(&id);
        Ok(())
    }

    pub fn rename(&mut self, id: NodeId, new_name: &str) -> Result<(), VfsError> {
        if id == self.root {
            return Err(VfsError::InvalidRootOp);
        }
        if new_name.is_empty() || new_name.contains('/') || new_name.contains('\\') {
            return Err(VfsError::InvalidName);
        }
        let parent = self
            .nodes
            .get(&id)
            .ok_or(VfsError::NotFound(id))?
            .parent
            .ok_or(VfsError::InvalidRootOp)?;
        // Ensure uniqueness among siblings excluding self.
        let conflict = self
            .get(parent)
            .ok_or(VfsError::NotFound(parent))?
            .children
            .iter()
            .filter(|cid| **cid != id)
            .any(|cid| {
                self.get(*cid)
                    .map(|n| n.name.eq_ignore_ascii_case(new_name))
                    .unwrap_or(false)
            });
        if conflict {
            return Err(VfsError::NameExists(new_name.to_string()));
        }
        self.nodes.get_mut(&id).ok_or(VfsError::NotFound(id))?.name = new_name.to_string();
        Ok(())
    }

    /// Depth-first flat list for UI: (depth, node_id)
    pub fn flat_tree(&self) -> Vec<(usize, NodeId)> {
        let mut out = Vec::new();
        self.walk(self.root, 0, &mut out);
        out
    }

    fn walk(&self, id: NodeId, depth: usize, out: &mut Vec<(usize, NodeId)>) {
        out.push((depth, id));
        if let Some(node) = self.get(id) {
            for child in &node.children {
                self.walk(*child, depth + 1, out);
            }
        }
    }

    pub fn url_path(&self, id: NodeId) -> String {
        let mut parts = Vec::new();
        let mut cur = Some(id);
        while let Some(cid) = cur {
            if cid == self.root {
                break;
            }
            if let Some(n) = self.get(cid) {
                parts.push(n.name.clone());
                cur = n.parent;
            } else {
                break;
            }
        }
        parts.reverse();
        if parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}/", parts.join("/")).replace("//", "/")
        }
    }

    /// Resolve a URL path to either a VFS node or a dynamic real path under a RealFolder.
    pub fn resolve(&self, raw_path: &str) -> ResolveResult {
        let path = normalize_url_path(raw_path);
        if path.is_empty() || path == "/" {
            return ResolveResult::Node(self.root);
        }
        let segments: Vec<&str> = path
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .collect();
        if segments.contains(&"..") {
            return ResolveResult::Forbidden;
        }

        let mut current = self.root;
        let mut rest: &[&str] = &segments;

        while !rest.is_empty() {
            let name = rest[0];
            let node = match self.get(current) {
                Some(n) => n,
                None => return ResolveResult::NotFound,
            };

            if let Some(&child_id) = node
                .children
                .iter()
                .find(|cid| self.get(**cid).map(|c| c.name == name).unwrap_or(false))
            {
                current = child_id;
                rest = &rest[1..];
                continue;
            }

            // Dynamic resolve inside real folder.
            if node.kind == NodeKind::RealFolder {
                if let Some(base) = &node.resource {
                    let mut disk = base.clone();
                    for seg in rest {
                        disk.push(seg);
                    }
                    if disk.exists() {
                        if disk.is_dir() {
                            return ResolveResult::DiskDir {
                                node_id: current,
                                path: disk,
                                url_suffix: rest.join("/"),
                            };
                        }
                        return ResolveResult::DiskFile {
                            node_id: current,
                            path: disk,
                        };
                    }
                }
            }
            return ResolveResult::NotFound;
        }

        ResolveResult::Node(current)
    }

    /// List directory entries for HTTP browsing.
    pub fn list_dir(&self, resolved: &ResolveResult) -> Vec<DirEntry> {
        match resolved {
            ResolveResult::Node(id) => {
                let Some(node) = self.get(*id) else {
                    return Vec::new();
                };
                match node.kind {
                    NodeKind::Root | NodeKind::VirtualFolder => node
                        .children
                        .iter()
                        .filter_map(|cid| self.get(*cid))
                        .filter(|n| !n.hidden)
                        .map(|n| DirEntry {
                            name: n.name.clone(),
                            is_dir: n.is_folder(),
                            size: n
                                .resource
                                .as_ref()
                                .and_then(|p| fs::metadata(p).ok().map(|m| m.len())),
                            mtime: n.resource.as_ref().and_then(|p| {
                                fs::metadata(p)
                                    .ok()
                                    .and_then(|m| m.modified().ok())
                                    .map(chrono_from_system)
                            }),
                        })
                        .collect(),
                    NodeKind::RealFolder => {
                        let mut entries = Vec::new();
                        // Explicit virtual children first.
                        for cid in &node.children {
                            if let Some(n) = self.get(*cid) {
                                if n.hidden {
                                    continue;
                                }
                                entries.push(DirEntry {
                                    name: n.name.clone(),
                                    is_dir: n.is_folder(),
                                    size: n
                                        .resource
                                        .as_ref()
                                        .and_then(|p| fs::metadata(p).ok().map(|m| m.len())),
                                    mtime: None,
                                });
                            }
                        }
                        if let Some(base) = &node.resource {
                            if let Ok(rd) = fs::read_dir(base) {
                                for ent in rd.flatten() {
                                    let name = ent.file_name().to_string_lossy().to_string();
                                    if name.starts_with('.') {
                                        continue;
                                    }
                                    // Skip names already covered by explicit children.
                                    if entries.iter().any(|e| e.name == name) {
                                        continue;
                                    }
                                    let meta = ent.metadata().ok();
                                    entries.push(DirEntry {
                                        name,
                                        is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                                        size: meta.as_ref().and_then(|m| {
                                            if m.is_file() { Some(m.len()) } else { None }
                                        }),
                                        mtime: meta
                                            .and_then(|m| m.modified().ok())
                                            .map(chrono_from_system),
                                    });
                                }
                            }
                        }
                        entries.sort_by(|a, b| {
                            b.is_dir
                                .cmp(&a.is_dir)
                                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                        });
                        entries
                    }
                    _ => Vec::new(),
                }
            }
            ResolveResult::DiskDir { path, .. } => {
                let mut entries = Vec::new();
                if let Ok(rd) = fs::read_dir(path) {
                    for ent in rd.flatten() {
                        let name = ent.file_name().to_string_lossy().to_string();
                        if name.starts_with('.') {
                            continue;
                        }
                        let meta = ent.metadata().ok();
                        entries.push(DirEntry {
                            name,
                            is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                            size: meta
                                .as_ref()
                                .and_then(|m| if m.is_file() { Some(m.len()) } else { None }),
                            mtime: meta.and_then(|m| m.modified().ok()).map(chrono_from_system),
                        });
                    }
                }
                entries.sort_by(|a, b| {
                    b.is_dir
                        .cmp(&a.is_dir)
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                });
                entries
            }
            _ => Vec::new(),
        }
    }

    /// Directory on disk that can accept uploads for a resolved path.
    /// Only RealFolder / nested disk dirs under a RealFolder are writable.
    pub fn upload_dir_for(&self, resolved: &ResolveResult) -> Option<PathBuf> {
        match resolved {
            ResolveResult::Node(id) => {
                let n = self.get(*id)?;
                if n.kind == NodeKind::RealFolder {
                    n.resource.clone()
                } else {
                    None
                }
            }
            ResolveResult::DiskDir { path, .. } => Some(path.clone()),
            _ => None,
        }
    }

    pub fn file_path_for(&self, resolved: &ResolveResult) -> Option<PathBuf> {
        match resolved {
            ResolveResult::Node(id) => {
                let n = self.get(*id)?;
                if n.kind == NodeKind::File {
                    n.resource.clone()
                } else {
                    None
                }
            }
            ResolveResult::DiskFile { path, .. } => Some(path.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolveResult {
    Node(NodeId),
    DiskFile {
        node_id: NodeId,
        path: PathBuf,
    },
    DiskDir {
        node_id: NodeId,
        path: PathBuf,
        url_suffix: String,
    },
    NotFound,
    Forbidden,
}

impl ResolveResult {
    pub fn is_dir(&self, vfs: &Vfs) -> bool {
        match self {
            ResolveResult::Node(id) => vfs.get(*id).map(|n| n.is_folder()).unwrap_or(false),
            ResolveResult::DiskDir { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub mtime: Option<String>,
}

fn normalize_url_path(raw: &str) -> String {
    let decoded = percent_encoding::percent_decode_str(raw)
        .decode_utf8_lossy()
        .to_string();
    let mut out = String::from("/");
    for seg in decoded.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if !out.ends_with('/') {
            out.push('/');
        }
        out.push_str(seg);
    }
    if out.len() > 1 && raw.ends_with('/') && !out.ends_with('/') {
        out.push('/');
    }
    out
}

fn chrono_from_system(t: std::time::SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Local> = t.into();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn add_and_resolve_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("hello.txt");
        fs::write(&file, b"hi").unwrap();

        let mut vfs = Vfs::new();
        let id = vfs.add_file(vfs.root_id(), &file).unwrap();
        let r = vfs.resolve("/hello.txt");
        match r {
            ResolveResult::Node(nid) => assert_eq!(nid, id),
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(vfs.file_path_for(&r).unwrap(), file);
    }

    #[test]
    fn real_folder_dynamic() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("sub");
        fs::create_dir(&nested).unwrap();
        let file = nested.join("a.bin");
        fs::write(&file, b"data").unwrap();

        let mut vfs = Vfs::new();
        let folder_name = dir
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        vfs.add_real_folder(vfs.root_id(), dir.path()).unwrap();
        let path = format!("/{folder_name}/sub/a.bin");
        let r = vfs.resolve(&path);
        match r {
            ResolveResult::DiskFile { path, .. } => assert_eq!(path, file),
            other => panic!("unexpected {other:?}"),
        }
    }
}
