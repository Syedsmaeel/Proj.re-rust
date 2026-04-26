use git2::{Repository, StatusOptions};

pub fn get_pulse(path: &str) -> Result<Vec<String>, git2::Error> {
    let repo = Repository::open(path)?;
    let mut statuses = repo.statuses(Some(StatusOptions::default().include_untracked(true)))?;
    
    let mut files = Vec::new();
    for entry in statuses.iter() {
        if let Some(path) = entry.path() {
            files.push(path.to_string());
        }
    }
    Ok(files)
}

pub fn commit_all(path: &str, message: &str) -> Result<(), git2::Error> {
    let repo = Repository::open(path)?;
    let mut index = repo.index()?;
    index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let sig = repo.signature()?;
    let parent_commit = repo.head()?.peel_to_commit()?;
    
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent_commit])?;
    Ok(())
}
