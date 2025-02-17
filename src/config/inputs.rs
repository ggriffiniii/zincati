use crate::config::fragments;
use failure::{Fallible, ResultExt};
use log::trace;
use ordered_float::NotNan;
use serde::Serialize;

/// Runtime configuration holding environmental inputs.
#[derive(Debug, Serialize)]
pub(crate) struct ConfigInput {
    pub(crate) cincinnati: CincinnatiInput,
    pub(crate) updates: UpdateInput,
    pub(crate) identity: IdentityInput,
}

impl ConfigInput {
    /// Read config fragments and merge them into a single config.
    pub(crate) fn read_configs(
        dirs: Vec<String>,
        common_path: &str,
        extensions: Vec<String>,
    ) -> Fallible<Self> {
        use std::io::Read;

        let scanner = liboverdrop::FragmentScanner::new(dirs, common_path, true, extensions);

        let mut fragments = Vec::new();
        for (_, fpath) in scanner.scan() {
            trace!("reading config fragment '{}'", fpath.display());

            let fp = std::fs::File::open(&fpath)
                .context(format!("failed to open file '{}'", fpath.display()))?;
            let mut bufrd = std::io::BufReader::new(fp);
            let mut content = vec![];
            bufrd
                .read_to_end(&mut content)
                .context(format!("failed to read content of '{}'", fpath.display()))?;
            let frag: fragments::ConfigFragment =
                toml::from_slice(&content).context("failed to parse TOML")?;

            fragments.push(frag);
        }

        let cfg = Self::merge_fragments(fragments);
        Ok(cfg)
    }

    /// Merge multiple fragments into a single configuration.
    fn merge_fragments(fragments: Vec<fragments::ConfigFragment>) -> Self {
        let mut cincinnatis = vec![];
        let mut updates = vec![];
        let mut identities = vec![];

        for snip in fragments {
            if let Some(c) = snip.cincinnati {
                cincinnatis.push(c);
            }
            if let Some(f) = snip.updates {
                updates.push(f);
            }
            if let Some(i) = snip.identity {
                identities.push(i);
            }
        }

        Self {
            cincinnati: CincinnatiInput::from_fragments(cincinnatis),
            updates: UpdateInput::from_fragments(updates),
            identity: IdentityInput::from_fragments(identities),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CincinnatiInput {
    /// Base URL (template) for the Cincinnati service.
    pub(crate) base_url: String,
}

impl CincinnatiInput {
    fn from_fragments(fragments: Vec<fragments::CincinnatiFragment>) -> Self {
        let mut cfg = Self {
            base_url: String::new(),
        };

        for snip in fragments {
            if let Some(u) = snip.base_url {
                cfg.base_url = u;
            }
        }

        cfg
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct IdentityInput {
    pub(crate) group: String,
    pub(crate) node_uuid: String,
    pub(crate) rollout_wariness: Option<NotNan<f64>>,
}

impl IdentityInput {
    fn from_fragments(fragments: Vec<fragments::IdentityFragment>) -> Self {
        let mut cfg = Self {
            group: String::new(),
            node_uuid: String::new(),
            rollout_wariness: None,
        };

        for snip in fragments {
            if let Some(g) = snip.group {
                cfg.group = g;
            }
            if let Some(nu) = snip.node_uuid {
                cfg.node_uuid = nu;
            }
            if let Some(rw) = snip.rollout_wariness {
                cfg.rollout_wariness = Some(rw);
            }
        }

        cfg
    }
}

/// Config for update logic.
#[derive(Debug, Serialize)]
pub(crate) struct UpdateInput {
    /// Whether to enable automatic downgrades.
    pub(crate) allow_downgrade: bool,
    /// Whether to enable auto-updates logic.
    pub(crate) enabled: bool,
    /// Update strategy.
    pub(crate) strategy: String,
    /// `fleet_lock` strategy config.
    pub(crate) fleet_lock: FleetLockInput,
}

/// Config for "fleet_lock" strategy.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct FleetLockInput {
    /// Base URL (template) for the FleetLock service.
    pub(crate) base_url: String,
}

impl UpdateInput {
    fn from_fragments(fragments: Vec<fragments::UpdateFragment>) -> Self {
        let mut allow_downgrade = false;
        let mut enabled = true;
        let mut strategy = String::new();
        let mut fleet_lock = FleetLockInput {
            base_url: String::new(),
        };

        for snip in fragments {
            if let Some(a) = snip.allow_downgrade {
                allow_downgrade = a;
            }
            if let Some(e) = snip.enabled {
                enabled = e;
            }
            if let Some(s) = snip.strategy {
                strategy = s;
            }
            if let Some(fl) = snip.fleet_lock {
                if let Some(b) = fl.base_url {
                    fleet_lock.base_url = b;
                }
            }
        }

        Self {
            allow_downgrade,
            enabled,
            strategy,
            fleet_lock,
        }
    }
}
