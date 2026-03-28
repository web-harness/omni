pub use bashkit::{Bash, BashTool, ExecutionLimits, GitConfig};

pub struct BashkitBuilder {
    username: Option<String>,
    hostname: Option<String>,
    python_enabled: bool,
    git_enabled: bool,
}

impl BashkitBuilder {
    pub fn new() -> Self {
        Self {
            username: None,
            hostname: None,
            python_enabled: false,
            git_enabled: false,
        }
    }

    pub fn username(mut self, username: &str) -> Self {
        self.username = Some(username.to_string());
        self
    }

    pub fn hostname(mut self, hostname: &str) -> Self {
        self.hostname = Some(hostname.to_string());
        self
    }

    pub fn with_python(mut self, enabled: bool) -> Self {
        self.python_enabled = enabled;
        self
    }

    pub fn with_git(mut self, enabled: bool) -> Self {
        self.git_enabled = enabled;
        self
    }

    pub fn build(self) -> Bash {
        let mut builder = Bash::builder();

        if let Some(username) = self.username {
            builder = builder.username(&username);
        }

        if let Some(hostname) = self.hostname {
            builder = builder.hostname(&hostname);
        }

        if self.git_enabled {
            builder = builder.git(GitConfig::new());
        }

        builder.build()
    }
}

impl Default for BashkitBuilder {
    fn default() -> Self {
        Self::new()
    }
}
