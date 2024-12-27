use std::{fmt::Display, str::FromStr};

use leptos::prelude::*;
use leptos_use::use_preferred_dark;

use super::cookie::use_cookie;

/// Represents available themes for the application
#[derive(Debug, Clone, Default)]
pub enum Theme {
    #[default]
    Retro,
    Dark,
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Retro => write!(f, "retro"),
            Theme::Dark => write!(f, "dark"),
        }
    }
}

impl FromStr for Theme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "retro" => Ok(Theme::Retro),
            _ => Ok(Theme::Dark),
        }
    }
}

impl Theme {
    /// Switches between available themes - if current is dark, switches to light and vice versa
    pub fn switch(&self) -> Theme {
        match self {
            Theme::Retro => Theme::Dark,
            Theme::Dark => Theme::Retro,
        }
    }
}

#[derive(Clone)]
pub struct ThemeSwitcher {
    cookie: WriteSignal<Option<Theme>>,
    pub current: Signal<Theme>,
}

impl ThemeSwitcher {
    pub fn new() -> Self {
        // Try to load theme from cookies first
        let cookie = use_cookie::<Theme>("theme");

        let theme = Signal::derive(move || {
            // Check user's system preference for dark mode
            // Default to Retro theme if system preference is not available
            let preference = use_preferred_dark()
                .get_untracked()
                .then(|| Theme::Dark)
                .unwrap_or(Theme::Retro);

            // Use cookie value if exists, otherwise fall back to system preference
            cookie.0.get().unwrap_or(preference)
        });

        Self {
            cookie: cookie.1,
            current: theme,
        }
    }

    /// Switches the current theme and saves the new preference to cookie
    pub fn switch(&self) {
        self.cookie.set(Some(self.current.get_untracked().switch()));
    }
}
