use std::collections::BTreeMap;

use serde::Serialize;

use crate::database::{Contribution, StoredCommit, TrackedRepository};
use crate::utils::{join_lines, shorten_hash};
use crate::MarkdownStyle;

#[derive(Debug, Clone, Serialize)]
pub struct ContributionEvidence {
    pub contribution: Contribution,
    pub key_commits: Vec<StoredCommit>,
    pub related_commits: Vec<StoredCommit>,
    pub unresolved_key_refs: Vec<String>,
    pub unresolved_related_refs: Vec<String>,
}

pub fn render_markdown(
    repository: &TrackedRepository,
    items: &[ContributionEvidence],
    style: MarkdownStyle,
) -> String {
    let mut output = String::new();
    let heading = match style {
        MarkdownStyle::Resume => "Resume-Ready Contributions",
        MarkdownStyle::Portfolio => "Portfolio Contribution Summary",
    };

    output.push_str(&format!("# {}\n\n", heading));
    output.push_str(&format!(
        "**Repository:** {} (`{}`)\n",
        repository.name, repository.slug
    ));
    if let Some(remote_url) = &repository.remote_url {
        output.push_str(&format!("**Remote:** {}\n", remote_url));
    }
    output.push('\n');

    let mut grouped: BTreeMap<String, Vec<&ContributionEvidence>> = BTreeMap::new();
    for item in items {
        grouped
            .entry(item.contribution.category.clone())
            .or_default()
            .push(item);
    }

    let mut grouped = grouped.into_iter().collect::<Vec<_>>();
    grouped.sort_by(|left, right| category_sort_key(&right.1).cmp(&category_sort_key(&left.1)));

    for (category, mut contributions) in grouped {
        contributions.sort_by(|left, right| {
            right
                .contribution
                .priority
                .cmp(&left.contribution.priority)
                .then_with(|| left.contribution.name.cmp(&right.contribution.name))
        });

        output.push_str(&format!("## {}\n\n", category));

        for item in contributions {
            match style {
                MarkdownStyle::Resume => render_resume_entry(&mut output, item),
                MarkdownStyle::Portfolio => render_portfolio_entry(&mut output, item),
            }
        }
    }

    output
}

fn render_resume_entry(output: &mut String, item: &ContributionEvidence) {
    output.push_str(&format!("### {}\n\n", item.contribution.name));
    output.push_str(&format!("{}\n\n", item.contribution.overview));
    output.push_str(&format!(
        "Status: `{}`{}\n\n",
        item.contribution.status,
        item.contribution
            .confidence
            .as_ref()
            .map(|value| format!(" | Confidence: `{value}`"))
            .unwrap_or_default()
    ));

    if !item.contribution.resume_bullets.is_empty() {
        output.push_str(&join_lines(&item.contribution.resume_bullets));
        output.push_str("\n\n");
    } else {
        output.push_str(&format!("- {}\n\n", item.contribution.description));
    }

    if !item.key_commits.is_empty() {
        let hashes = item
            .key_commits
            .iter()
            .map(|commit| format!("`{}`", shorten_hash(&commit.hash)))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("Evidence: {}\n\n", hashes));
    }

    if !item.contribution.covered_prs.is_empty() {
        let prs = item
            .contribution
            .covered_prs
            .iter()
            .map(|number| format!("#{}", number))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("Covered PRs: {}\n\n", prs));
    }
}

fn render_portfolio_entry(output: &mut String, item: &ContributionEvidence) {
    output.push_str(&format!("### {}\n\n", item.contribution.name));
    output.push_str(&format!("**Priority:** {}\n\n", item.contribution.priority));
    output.push_str(&format!("**Status:** {}\n\n", item.contribution.status));
    if let Some(confidence) = &item.contribution.confidence {
        output.push_str(&format!("**Confidence:** {}\n\n", confidence));
    }
    output.push_str(&format!("{}\n\n", item.contribution.overview));
    output.push_str(&format!("{}\n\n", item.contribution.description));

    if let Some(rationale) = &item.contribution.rationale {
        output.push_str("#### Rationale\n\n");
        output.push_str(&format!("{}\n\n", rationale));
    }

    if !item.contribution.covered_prs.is_empty() {
        let prs = item
            .contribution
            .covered_prs
            .iter()
            .map(|number| format!("#{}", number))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("**Covered PRs:** {}\n\n", prs));
    }

    if !item.contribution.technical_details.is_empty() {
        output.push_str("#### Technical Details\n\n");
        output.push_str(&join_lines(&item.contribution.technical_details));
        output.push_str("\n\n");
    }

    if !item.contribution.resume_bullets.is_empty() {
        output.push_str("#### Resume Bullets\n\n");
        output.push_str(&join_lines(&item.contribution.resume_bullets));
        output.push_str("\n\n");
    }

    render_commit_section(
        output,
        "Key Commits",
        &item.key_commits,
        &item.unresolved_key_refs,
    );
    render_commit_section(
        output,
        "Related Commits",
        &item.related_commits,
        &item.unresolved_related_refs,
    );
}

fn render_commit_section(
    output: &mut String,
    title: &str,
    commits: &[StoredCommit],
    unresolved_refs: &[String],
) {
    if commits.is_empty() && unresolved_refs.is_empty() {
        return;
    }

    output.push_str(&format!("#### {}\n\n", title));

    for commit in commits {
        output.push_str(&format!(
            "- `{}` {} (+{}, -{})\n",
            shorten_hash(&commit.hash),
            commit.summary,
            commit.lines_added,
            commit.lines_deleted,
        ));
    }

    for unresolved in unresolved_refs {
        output.push_str(&format!("- `{}` (not imported yet)\n", unresolved));
    }

    output.push('\n');
}

fn category_sort_key(items: &[&ContributionEvidence]) -> (u8, String) {
    let highest_priority = items
        .iter()
        .map(|item| item.contribution.priority)
        .max()
        .unwrap_or_default();
    let category = items
        .first()
        .map(|item| item.contribution.category.clone())
        .unwrap_or_default();

    (highest_priority, category)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> TrackedRepository {
        TrackedRepository {
            id: 1,
            slug: "contrack".to_string(),
            name: "Contrack".to_string(),
            local_path: "/tmp/contrack".to_string(),
            remote_url: Some("https://github.com/isaacadams/contrack".to_string()),
        }
    }

    fn contribution(name: &str, category: &str, priority: u8) -> Contribution {
        Contribution {
            id: 1,
            repository_id: 1,
            repository_slug: "contrack".to_string(),
            name: name.to_string(),
            overview: format!("Overview for {name}"),
            description: format!("Description for {name}"),
            category: category.to_string(),
            priority,
            status: "draft".to_string(),
            confidence: Some("high".to_string()),
            rationale: Some("Clear feature grouping".to_string()),
            covered_prs: vec![123],
            key_commit_refs: vec!["abc123".to_string()],
            related_commit_refs: Vec::new(),
            technical_details: vec!["Rust CLI".to_string()],
            resume_bullets: vec![format!("Delivered {name}")],
        }
    }

    fn commit(summary: &str) -> StoredCommit {
        StoredCommit {
            hash: "abc123456789".to_string(),
            repository_id: 1,
            repository_slug: "contrack".to_string(),
            author_name: "Isaac".to_string(),
            author_email: Some("isaac@example.com".to_string()),
            committed_at: "now".to_string(),
            summary: summary.to_string(),
            body: None,
            files_changed: vec!["src/main.rs".to_string()],
            lines_added: 10,
            lines_deleted: 2,
        }
    }

    #[test]
    fn renders_categories_by_priority() {
        let feature = ContributionEvidence {
            contribution: contribution("Generator", "Feature", 5),
            key_commits: vec![commit("generator commit")],
            related_commits: Vec::new(),
            unresolved_key_refs: Vec::new(),
            unresolved_related_refs: Vec::new(),
        };
        let bugfix = ContributionEvidence {
            contribution: contribution("Cleanup", "Bug Fix", 2),
            key_commits: vec![commit("cleanup commit")],
            related_commits: Vec::new(),
            unresolved_key_refs: Vec::new(),
            unresolved_related_refs: Vec::new(),
        };

        let rendered = render_markdown(&repository(), &[bugfix, feature], MarkdownStyle::Portfolio);
        let feature_index = rendered.find("## Feature").expect("feature section");
        let bugfix_index = rendered.find("## Bug Fix").expect("bugfix section");
        assert!(feature_index < bugfix_index);
    }
}
