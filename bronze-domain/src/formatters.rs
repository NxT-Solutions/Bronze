//! Named output-profile formatters (story 6.4, QUE-004).

use crate::entities::Lifecycle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostCopyAction {
    Unchanged,
    Copied,
    Active,
    Done,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdvancePolicy {
    Keep,
    NextQueued,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Plain {
        separator: String,
    },
    MarkdownBullets {
        marker: char,
        separator: String,
    },
    MarkdownNumbered {
        start_at: u32,
        separator: String,
    },
    PromptBlock {
        context_heading: String,
        instruction_heading: String,
        item_delimiter: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputProfile {
    pub format: OutputFormat,
    pub post_copy_action: PostCopyAction,
    pub advance_policy: AdvancePolicy,
}

pub fn default_output_profile() -> OutputProfile {
    OutputProfile {
        format: OutputFormat::Plain {
            separator: "\n".into(),
        },
        post_copy_action: PostCopyAction::Copied,
        advance_policy: AdvancePolicy::Keep,
    }
}

pub fn format_items(profile: &OutputProfile, items: &[&str]) -> String {
    match &profile.format {
        OutputFormat::Plain { separator } => items.join(separator),
        OutputFormat::MarkdownBullets { marker, separator } => items
            .iter()
            .map(|item| format!("{marker} {item}"))
            .collect::<Vec<_>>()
            .join(separator),
        OutputFormat::MarkdownNumbered {
            start_at,
            separator,
        } => items
            .iter()
            .enumerate()
            .map(|(i, item)| format!("{}. {item}", start_at.saturating_add(i as u32)))
            .collect::<Vec<_>>()
            .join(separator),
        OutputFormat::PromptBlock {
            context_heading,
            instruction_heading,
            item_delimiter,
        } => {
            let body = items
                .iter()
                .map(|item| escape_delimiter(item, item_delimiter))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "{context_heading}\n{item_delimiter}\n{body}\n{item_delimiter}\n{instruction_heading}"
            )
        }
    }
}

pub fn lifecycle_after_copy(action: PostCopyAction) -> Option<Lifecycle> {
    match action {
        PostCopyAction::Unchanged => None,
        PostCopyAction::Copied => Some(Lifecycle::Copied),
        PostCopyAction::Active => Some(Lifecycle::Active),
        PostCopyAction::Done => Some(Lifecycle::Done),
    }
}

fn escape_delimiter(item: &str, delimiter: &str) -> String {
    if delimiter.is_empty() {
        return item.to_string();
    }
    item.replace(delimiter, &format!("\\{delimiter}"))
}

#[cfg(test)]
mod formatters_tests {
    use super::*;

    #[test]
    fn formatters_default_is_copied_keep_and_prompt_escapes() {
        let default = default_output_profile();
        assert_eq!(default.post_copy_action, PostCopyAction::Copied);
        assert_eq!(default.advance_policy, AdvancePolicy::Keep);
        assert_eq!(format_items(&default, &["a", "b"]), "a\nb");
        let prompt = OutputProfile {
            format: OutputFormat::PromptBlock {
                context_heading: "Context".into(),
                instruction_heading: "Instructions".into(),
                item_delimiter: "<<<".into(),
            },
            post_copy_action: PostCopyAction::Copied,
            advance_policy: AdvancePolicy::Keep,
        };
        let out = format_items(&prompt, &["hello <<< world", "${not-a-template}"]);
        assert!(out.contains("hello \\<<< world"));
        assert!(out.contains("${not-a-template}"));
        assert!(!out.contains("<html"));
        assert_eq!(
            lifecycle_after_copy(PostCopyAction::Copied),
            Some(Lifecycle::Copied)
        );
    }
}
