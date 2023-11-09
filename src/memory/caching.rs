#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum CachingMechanism {
    Forgetful,
    SummarizeAtLimit { limit: usize, save_to_lt: bool },
}

impl Default for CachingMechanism {
    fn default() -> Self {
        Self::default_summary_at_limit()
    }
}

impl CachingMechanism {
    pub fn limit(&self) -> usize {
        match self {
            Self::Forgetful => 2, // Only allows for user in agent out
            Self::SummarizeAtLimit { limit, .. } => *limit as usize,
        }
    }
    pub fn long_term_enabled(&self) -> bool {
        match self {
            Self::Forgetful => false,
            Self::SummarizeAtLimit { save_to_lt, .. } => *save_to_lt,
        }
    }
    pub fn default_summary_at_limit() -> Self {
        let limit = 50;
        let save_to_lt = false;
        CachingMechanism::SummarizeAtLimit { limit, save_to_lt }
    }
}
