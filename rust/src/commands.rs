//! Searchable native commands, shared by pointer and keyboard activation.
use crate::runtime::{MotionPreference, Route, Runtime};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Page(Route),
    Theme,
    Palette(bool),
    Hue(f32),
    Motion(MotionPreference),
    Guide,
    Toast,
}
#[derive(Clone, Debug)]
pub struct Command {
    pub id: String,
    pub label: String,
    pub detail: &'static str,
    pub icon: &'static str,
    pub action: Action,
}

pub fn all() -> Vec<Command> {
    let mut commands: Vec<_> = Route::ALL
        .into_iter()
        .map(|route| Command {
            id: route.id().into(),
            label: route.title().into(),
            detail: route.group(),
            icon: route.icon(),
            action: Action::Page(route),
        })
        .collect();
    for (id, label, detail, icon, action) in [
        ("theme", "Toggle theme", "Appearance", "sun", Action::Theme),
        (
            "color",
            "Color palette",
            "Appearance",
            "droplet",
            Action::Palette(false),
        ),
        (
            "mono",
            "Mono palette",
            "Appearance",
            "moon",
            Action::Palette(true),
        ),
        (
            "motion-full",
            "Full motion",
            "Motion",
            "play",
            Action::Motion(MotionPreference::Full),
        ),
        (
            "motion-slow",
            "Slow-motion mode",
            "Motion",
            "wave",
            Action::Motion(MotionPreference::Slow),
        ),
        (
            "motion-reduced",
            "Reduced motion",
            "Motion",
            "pause",
            Action::Motion(MotionPreference::Reduced),
        ),
        (
            "motion-system",
            "System motion preference",
            "Motion",
            "sliders",
            Action::Motion(MotionPreference::System),
        ),
        ("guide", "Open guide", "Help", "book-open", Action::Guide),
        ("toast", "Show a toast", "Feedback", "bell", Action::Toast),
    ] {
        commands.push(Command {
            id: id.into(),
            label: label.into(),
            detail,
            icon,
            action,
        });
    }
    for (name, hue) in [
        ("Forest", 168.0),
        ("Teal", 195.0),
        ("Ochre", 55.0),
        ("Clay", 18.0),
        ("Blue", 235.0),
        ("Violet", 290.0),
        ("Rose", 330.0),
        ("Red", 0.0),
    ] {
        commands.push(Command {
            id: format!("accent-{name}"),
            label: format!("{name} accent"),
            detail: "Accent color",
            icon: "droplet",
            action: Action::Hue(hue),
        });
    }
    commands
}

/// Consecutive and word-start matches rank ahead of scattered subsequences.
pub fn score(query: &str, text: &str) -> Option<i32> {
    let query = query.trim().to_lowercase();
    let text = text.to_lowercase();
    if query.is_empty() {
        return Some(0);
    }
    if text == query {
        return Some(10_000);
    }
    if text.starts_with(&query) {
        return Some(5_000 - text.len() as i32);
    }
    if let Some(index) = text.find(&query) {
        return Some(3_000 - index as i32);
    }
    let mut needle = query.chars().peekable();
    let mut score = 0;
    let mut previous = false;
    let mut boundary = true;
    for c in text.chars() {
        let matched = needle.peek() == Some(&c);
        if matched {
            needle.next();
            score += 10 + if previous { 16 } else { 0 } + if boundary { 12 } else { 0 };
        } else {
            score -= 1;
        }
        previous = matched;
        boundary = c.is_whitespace() || c == '-' || c == '/';
    }
    needle.peek().is_none().then_some(score)
}
pub fn search(query: &str) -> Vec<Command> {
    let mut matches: Vec<_> = all()
        .into_iter()
        .filter_map(|c| score(query, &c.label).map(|s| (s, c)))
        .collect();
    matches.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    matches.into_iter().map(|(_, c)| c).collect()
}
pub fn run(action: Action, rt: &mut Runtime) {
    match action {
        Action::Page(route) => rt.navigate(route),
        Action::Theme => rt.set_theme(!rt.desired_theme.unwrap_or(rt.prefs.dark)),
        Action::Palette(mono) => rt.set_palette(mono),
        Action::Hue(hue) => rt.set_hue(hue),
        Action::Motion(motion) => rt.set_motion(motion),
        Action::Guide => rt.request_guide(),
        Action::Toast => rt.toast("Saved", "Changes are available locally.", "success"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search_ranks_exact_and_consecutive_matches() {
        assert_eq!(search("forms")[0].id, "forms");
        assert_eq!(search("workspace")[0].id, "workspace");
        assert!(!search("rduc mot").is_empty());
        assert!(search("zzzzzz").is_empty());
    }
    #[test]
    fn every_command_has_a_real_icon() {
        for command in all() {
            assert!(
                crate::icons::names().contains(&command.icon),
                "{}",
                command.icon
            );
        }
    }
}
