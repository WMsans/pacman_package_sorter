use crate::packages::models::{Package, ShowMode, SortKey}; 
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterState {
    Include,
    Exclude,
    Ignore,
}

impl Default for FilterState {
    fn default() -> Self {
        FilterState::Ignore
    }
}

// Filter packages based on criteria
pub fn filter_packages(
    packages: &[Package],
    tag_filters: &HashMap<String, FilterState>,
    repo_filters: &HashMap<String, FilterState>,
    show_mode: ShowMode, 
    orphan_names: &[String], 
) -> Vec<Package> {
    let include_tags: Vec<_> = tag_filters
        .iter()
        .filter(|(_, v)| **v == FilterState::Include)
        .map(|(k, _)| k)
        .collect();
    let exclude_tags: Vec<_> = tag_filters
        .iter()
        .filter(|(_, v)| **v == FilterState::Exclude)
        .map(|(k, _)| k)
        .collect();
    let include_repos: Vec<_> = repo_filters
        .iter()
        .filter(|(_, v)| **v == FilterState::Include)
        .map(|(k, _)| k)
        .collect();
    let exclude_repos: Vec<_> = repo_filters
        .iter()
        .filter(|(_, v)| **v == FilterState::Exclude)
        .map(|(k, _)| k)
        .collect();

    packages
        .iter()
        .filter(|p| {
            if include_tags.is_empty() {
                true
            } else {
                include_tags.iter().any(|t| p.tags.contains(t))
            }
        })
        .filter(|p| !exclude_tags.iter().any(|t| p.tags.contains(t)))
        .filter(|p| {
            if include_repos.is_empty() {
                true
            } else {
                include_repos
                    .iter()
                    .any(|r| format!("{:?}", p.repository).to_lowercase() == r.to_lowercase())
            }
        })
        .filter(|p| {
            !exclude_repos
                .iter()
                .any(|r| format!("{:?}", p.repository).to_lowercase() == r.to_lowercase())
        })
        .filter(|p| match show_mode {
            ShowMode::AllInstalled => true,
            ShowMode::ExplicitlyInstalled => p.is_explicit,
            ShowMode::Dependencies => !p.is_explicit,
            ShowMode::Orphans => orphan_names.contains(&p.name),
            ShowMode::AllAvailable => true,
        })
        .cloned()
        .collect()
}
// Sort packages by a given key
pub fn sort_packages(packages: &mut Vec<Package>, sort_key: SortKey) {
    packages.sort_by(|a, b| match sort_key {
        SortKey::Name => a.name.cmp(&b.name),
        SortKey::Size => b
            .size
            .partial_cmp(&a.size)
            .unwrap_or(std::cmp::Ordering::Equal),
        SortKey::InstallDate => b.install_date.cmp(&a.install_date),
        SortKey::UpdateDate => b.build_date.cmp(&a.build_date),
        SortKey::Popularity => {
            let a_pop = a.popularity.unwrap_or(0.0);
            let b_pop = b.popularity.unwrap_or(0.0);
            b_pop.partial_cmp(&a_pop).unwrap_or(std::cmp::Ordering::Equal)
        }
    });
}

pub fn get_all_repos(packages: &[Package]) -> Vec<String> {
    let mut repos: BTreeSet<String> = BTreeSet::new();
    for pkg in packages {
        repos.insert(format!("{:?}", pkg.repository));
    }
    repos.into_iter().collect()
}