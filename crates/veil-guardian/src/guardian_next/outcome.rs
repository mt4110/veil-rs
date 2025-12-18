#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchOutcome {
    CacheHitFresh,
    CacheHitStale,
    CacheHitStaleFallback,
    NetworkFetched,
    NetworkNotModified,
    OfflineUsedFreshCache,
    OfflineFallbackUsedStale,
    FailedNoUsableCache,
}

/// Human readable label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeLabel(pub String);

impl OutcomeLabel {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn outcome_label(outcome: FetchOutcome) -> OutcomeLabel {
    let s = match outcome {
        FetchOutcome::CacheHitFresh => "Hit (Fresh)",
        FetchOutcome::OfflineUsedFreshCache => "Hit (Fresh)",
        FetchOutcome::CacheHitStale => "Hit (Stale)",
        FetchOutcome::CacheHitStaleFallback => "Hit (Stale) [Fallback]",
        FetchOutcome::OfflineFallbackUsedStale => "Hit (Stale) [Offline Fallback]",
        FetchOutcome::NetworkFetched => "Fetched",
        FetchOutcome::NetworkNotModified => "Hit (Fresh) [304]",
        FetchOutcome::FailedNoUsableCache => "Miss (No Cache)",
    };
    OutcomeLabel(s.to_string())
}
