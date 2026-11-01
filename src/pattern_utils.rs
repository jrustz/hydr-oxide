use std::{collections::HashMap, fmt::Display};

use rand::RngExt;
use reqwest::header::{HeaderMap, HeaderValue};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pattern {
    User,
    Pass,
    RandIP,
}

impl Pattern {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "^USER^",
            Self::Pass => "^PASS^",
            Self::RandIP => "^RAND_IP^",
        }
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct TemplateEngine {
    patterns: HashMap<Pattern, String>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        patterns.insert(Pattern::User, String::new());
        patterns.insert(Pattern::Pass, String::new());
        patterns.insert(Pattern::RandIP, String::new());
        Self { patterns }
    }

    pub fn set_value(&mut self, pattern: Pattern, value: &str) {
        self.patterns.insert(pattern, value.to_string());
    }

    pub fn render(&self, template: &str) -> String {
        let mut result = Self::normolize_template(template);
        for (pattern, value) in self.patterns.iter().filter(|(_, v)| !v.is_empty()) {
            result = result.replace(pattern.as_str(), value);
        }
        result
    }

    pub fn render_headers(&self, mut headers: HeaderMap) -> HeaderMap {
        for (_, value) in headers.iter_mut() {
            let value_str = value.to_str().unwrap();

            let rendered_str = self.render(value_str);

            *value = HeaderValue::from_str(&rendered_str).unwrap();
        }

        headers
    }
    pub fn generate_random_ip(&self) -> String {
        let mut rng = rand::rng();
        format!(
            "{}.{}.{}.{}",
            rng.random_range(0..=255),
            rng.random_range(0..=255),
            rng.random_range(0..=255),
            rng.random_range(0..=255)
        )
    }

    pub fn normolize_template(template: &str) -> String {
        template.trim().to_string()
    }
}
