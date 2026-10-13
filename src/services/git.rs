//! Phobos IDE - Git Service (git2)
use anyhow::Result;

pub struct GitInfo {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
}

impl Default for GitInfo {
    fn default() -> Self {
        Self { branch: "main".into(), ahead: 0, behind: 0, staged: 0, unstaged: 0, untracked: 0 }
    }
}

pub fn git_info(path: &str) -> GitInfo {
    let mut info = GitInfo::default();
    let repo = match git2::Repository::discover(path) {
        Ok(r) => r, Err(_) => return info,
    };
    if let Ok(head) = repo.head() {
        if let Some(name) = head.shorthand() { info.branch = name.to_string(); }
    }
    if let Ok(statuses) = repo.statuses(None) {
        for s in statuses.iter() {
            let st = s.status();
            if st.contains(git2::Status::INDEX_NEW) || st.contains(git2::Status::INDEX_MODIFIED) { info.staged += 1; }
            if st.contains(git2::Status::WT_MODIFIED) || st.contains(git2::Status::WT_DELETED) { info.unstaged += 1; }
            if st.contains(git2::Status::WT_NEW) { info.untracked += 1; }
        }
    }
    info
}

pub fn git_log(path: &str, n: usize) -> Vec<String> {
    let mut out = Vec::new();
    let repo = match git2::Repository::discover(path) { Ok(r) => r, Err(_) => return out };
    let mut revwalk = match repo.revwalk() { Ok(r) => r, Err(_) => return out };
    let _ = revwalk.push_head();
    for (i, oid) in revwalk.enumerate() {
        if i >= n { break; }
        if let Ok(oid) = oid {
            if let Ok(commit) = repo.find_commit(oid) {
                out.push(format!("{} {} {}", &oid.to_string()[..7],
                    commit.author().name().unwrap_or("?"),
                    commit.summary().unwrap_or("")));
            }
        }
    }
    out
}
