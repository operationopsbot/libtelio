//! Object descriptions of various
//! telio configurable features via API

use std::{collections::HashSet, fmt};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{de::IntoDeserializer, Deserialize, Deserializer, Serialize};
use strum_macros::EnumCount;
use telio_utils::telio_log_warn;

/// Default keepalive period used for proxying peers,
/// STUN servers and VPN servers
pub const DEFAULT_PERSISTENT_KEEPALIVE_PERIOD: u32 = 25;

/// Default keepalive period for direct peers
pub const DEFAULT_DIRECT_PERSISTENT_KEEPALIVE_PERIOD: u32 = 5;

/// Configurable persistent keepalive periods for different types of peers
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct FeaturePersistentKeepalive {
    /// Persistent keepalive period given for VPN peers (in seconds) [default 15s]
    #[serde(default = "FeaturePersistentKeepalive::get_default_keepalive_period")]
    pub vpn: Option<u32>,

    /// Persistent keepalive period for direct peers (in seconds) [default 5s]
    #[serde(default = "FeaturePersistentKeepalive::get_default_direct_keepalive_period")]
    pub direct: u32,

    /// Persistent keepalive period for proxying peers (in seconds) [default 25s]
    #[serde(default = "FeaturePersistentKeepalive::get_default_keepalive_period")]
    pub proxying: Option<u32>,

    /// Persistent keepalive period for stun peers (in seconds) [default 25s]
    #[serde(default = "FeaturePersistentKeepalive::get_default_keepalive_period")]
    pub stun: Option<u32>,
}

impl FeaturePersistentKeepalive {
    fn get_default_keepalive_period() -> Option<u32> {
        Some(DEFAULT_PERSISTENT_KEEPALIVE_PERIOD)
    }

    fn get_default_direct_keepalive_period() -> u32 {
        DEFAULT_DIRECT_PERSISTENT_KEEPALIVE_PERIOD
    }
}

impl Default for FeaturePersistentKeepalive {
    fn default() -> Self {
        Self {
            vpn: Some(DEFAULT_PERSISTENT_KEEPALIVE_PERIOD),
            direct: DEFAULT_DIRECT_PERSISTENT_KEEPALIVE_PERIOD,
            proxying: Some(DEFAULT_PERSISTENT_KEEPALIVE_PERIOD),
            stun: Some(DEFAULT_PERSISTENT_KEEPALIVE_PERIOD),
        }
    }
}

/// Configurable features for Wireguard peers
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct FeatureWireguard {
    /// Configurable persistent keepalive periods for wireguard peers
    #[serde(default)]
    pub persistent_keepalive: FeaturePersistentKeepalive,
}

impl FeatureWireguard {
    fn default_on_null<'de, D>(deserializer: D) -> Result<FeatureWireguard, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::deserialize(deserializer)?;
        Ok(opt.unwrap_or_default())
    }
}

#[serde_with::serde_as]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
/// QoS configuration options
pub struct FeatureQoS {
    /// How often to collect rtt data in seconds. Default value is 300.
    pub rtt_interval: Option<u32>,
    /// Number of tries for each node. Default value is 3.
    pub rtt_tries: Option<u32>,
    /// Types of rtt analytics. Default is Ping.
    pub rtt_types: Option<Vec<String>>,
    /// Number of buckets used for rtt and throughput. Default value is 5.
    pub buckets: Option<u32>,
}

/// Configurable features for Nurse module
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct FeatureNurse {
    /// The unique identifier of the device, used for meshnet ID
    pub fingerprint: String,
    /// Heartbeat interval in seconds. Default value is 3600.
    pub heartbeat_interval: Option<u32>,
    /// Initial heartbeat interval in seconds. Default value is None.
    pub initial_heartbeat_interval: Option<u32>,
    /// QoS configuration for Nurse
    pub qos: Option<FeatureQoS>,
    /// Enable/disable collecting nat type
    pub enable_nat_type_collection: Option<bool>,
}

/// Configurable features for Lana module
#[derive(Clone, Default, PartialEq, Eq, Deserialize)]
pub struct FeatureLana {
    /// Path of the file where events will be stored. If such file does not exist, it will be created, otherwise reused
    pub event_path: String,
    /// Whether the events should be sent to produciton or not
    pub prod: bool,
}

impl fmt::Debug for FeatureLana {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FeatureLana")
            .field("prod", &self.prod)
            .finish()
    }
}

/// Configurable features for exit Dns
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct FeatureExitDns {
    /// Controls if it is allowed to reconfigure DNS peer when exit node is
    /// (dis)connected.
    pub auto_switch_dns_ips: Option<bool>,
}

/// Mesh connection path type
#[derive(Clone, Copy, Debug, Default, EnumCount, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PathType {
    /// Nodes connected via a middle-man relay
    #[default]
    Relay,
    /// Nodes connected directly via WG
    Direct,
}

/// Enable wanted paths for telio
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct FeaturePaths {
    /// Enable paths in increasing priority: 0 is worse then 1 is worse then 2 ...
    /// [PathType::Relay] always assumed as -1
    pub priority: Vec<PathType>,
    /// Force only one specific path to be used.
    pub force: Option<PathType>,
}

/// Available Endpoint Providers for meshnet direct connections
#[derive(
    Clone,
    Copy,
    Debug,
    EnumCount,
    IntoPrimitive,
    TryFromPrimitive,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
#[repr(u32)]
#[serde(rename_all = "kebab-case")]
pub enum EndpointProvider {
    /// Use local interface ips as possible endpoints
    Local = 1,
    /// Use stun and wg-stun results as possible endpoints
    Stun = 2,
    /// Use IGD and upnp to generate endpoints
    Upnp = 3,
}

/// Endpoint polling interval
pub const DEFAULT_ENDPOINT_POLL_INTERVAL_SECS: u64 = 25;

/// Enable meshent direct connection
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct FeatureDirect {
    /// Endpoint providers [default all]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_providers")]
    pub providers: Option<HashSet<EndpointProvider>>,
    /// Polling interval for endpoints [default 10s]
    pub endpoint_interval_secs: Option<u64>,
    /// Configuration options for skipping unresponsive peers
    #[serde(default = "FeatureDirect::default_skip_unresponsive_peers")]
    pub skip_unresponsive_peers: Option<FeatureSkipUnresponsivePeers>,
}

impl Default for FeatureDirect {
    fn default() -> Self {
        Self {
            skip_unresponsive_peers: Self::default_skip_unresponsive_peers(),
            providers: Default::default(),
            endpoint_interval_secs: Default::default(),
        }
    }
}

impl FeatureDirect {
    fn default_skip_unresponsive_peers() -> Option<FeatureSkipUnresponsivePeers> {
        Some(Default::default())
    }
}

/// Avoid sending periodic messages to peers with no traffic reported by wireguard
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct FeatureSkipUnresponsivePeers {
    /// Time after which peers is considered unresponsive if it didn't receive any packets
    #[serde(default = "FeatureSkipUnresponsivePeers::default_no_rx_threshold_secs")]
    pub no_rx_threshold_secs: u64,
}

impl FeatureSkipUnresponsivePeers {
    const fn default_no_rx_threshold_secs() -> u64 {
        180
    }
}

impl Default for FeatureSkipUnresponsivePeers {
    fn default() -> Self {
        Self {
            no_rx_threshold_secs: Self::default_no_rx_threshold_secs(),
        }
    }
}

/// Configure derp behaviour
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct FeatureDerp {
    /// Tcp keepalive set on derp server's side [default 15s]
    pub tcp_keepalive: Option<u32>,
    /// Derp will send empty messages after this many seconds of not sending/receiving any data [default 60s]
    pub derp_keepalive: Option<u32>,
    /// Enable polling of remote peer states to reduce derp traffic
    pub enable_polling: Option<bool>,
    /// Use Mozilla's root certificates instead of OS ones [default false]
    #[serde(default)]
    pub use_built_in_root_certificates: bool,
}

/// Whether to validate keys
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct FeatureValidateKeys(pub bool);

impl Default for FeatureValidateKeys {
    fn default() -> Self {
        Self(true)
    }
}

/// Turns on connection resets upon VPN server change
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct FeatureBoringtunResetConns(pub bool);

/// Turns on post quantum VPN tunnel
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct FeaturePostQuantumVPN {
    /// Initial handshake retry interval in seconds
    #[serde(default = "FeaturePostQuantumVPN::default_handshake_retry_interval_s")]
    pub handshake_retry_interval_s: u32,

    /// Rekey interval in seconds
    #[serde(default = "FeaturePostQuantumVPN::default_rekey_interval_s")]
    pub rekey_interval_s: u32,
}

impl FeaturePostQuantumVPN {
    const fn default_handshake_retry_interval_s() -> u32 {
        8
    }

    const fn default_rekey_interval_s() -> u32 {
        90
    }
}

fn deserialize_providers<'de, D>(de: D) -> Result<Option<HashSet<EndpointProvider>>, D::Error>
where
    D: Deserializer<'de>,
{
    let eps: Vec<&str> = match Deserialize::deserialize(de) {
        Ok(vec) => vec,
        Err(_) => return Ok(None),
    };

    let eps: HashSet<_> = eps
        .into_iter()
        .filter_map(|provider| {
            EndpointProvider::deserialize(<&str as IntoDeserializer>::into_deserializer(provider))
                .map_err(|e| {
                    telio_log_warn!("Failed to parse EndpointProvider: {}", e);
                })
                .ok()
        })
        .collect();

    Ok(Some(eps))
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
/// Encompasses all of the possible features that can be enabled
pub struct Features {
    /// Additional wireguard configuration
    #[serde(default, deserialize_with = "FeatureWireguard::default_on_null")]
    pub wireguard: FeatureWireguard,
    /// Nurse features that can be configured for QoS
    pub nurse: Option<FeatureNurse>,
    /// Event logging configurable features
    pub lana: Option<FeatureLana>,
    /// Deprecated by direct since 4.0.0
    pub paths: Option<FeaturePaths>,
    /// Configure options for exit dns
    pub exit_dns: Option<FeatureExitDns>,
    /// Configure options for direct WG connections
    pub direct: Option<FeatureDirect>,
    /// Test environment (natlab) requires binding feature disabled
    /// TODO: Remove it once mac integration tests support the binding mechanism
    #[cfg(any(target_os = "macos", feature = "pretend_to_be_macos"))]
    pub is_test_env: Option<bool>,
    /// Derp server specific configuration
    pub derp: Option<FeatureDerp>,
    /// Flag to specify if keys should be validated
    #[serde(default)]
    pub validate_keys: FeatureValidateKeys,
    /// IPv6 support
    #[serde(default)]
    pub ipv6: bool,
    /// Nicknames support
    #[serde(default)]
    pub nicknames: bool,
    /// Flag to turn on connection reset upon VPN server change for boringtun adapter
    #[serde(default)]
    pub boringtun_reset_connections: FeatureBoringtunResetConns,
    /// If and for how long to flush events when stopping telio. Setting to Some(0) means waiting until all events have been flushed, regardless of how long it takes
    pub flush_events_on_stop_timeout_seconds: Option<u64>,
    /// Flag to turn on post quantum VPN tunnel
    #[serde(default)]
    pub post_quantum_vpn: Option<FeaturePostQuantumVPN>,
}

impl FeaturePaths {
    /// Returns a vector of 'PathType' sorted according to the priority
    pub fn paths(&self) -> Vec<PathType> {
        if let Some(path) = self.force {
            vec![path]
        } else {
            // Collect PathType array, without any duplicates, and keeping prio for priorities
            [PathType::Relay]
                .iter()
                .chain(self.priority.iter())
                .fold(Vec::new(), |mut v, p| {
                    if !v.contains(p) {
                        v.push(*p);
                    }
                    v
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;
    use serde_json::from_str;

    use super::*;

    const CORRECT_FEATURES_JSON_WITHOUT_IS_TEST_ENV: &str = r#"
        {
            "wireguard":
            {
                "persistent_keepalive": {
                    "vpn": null,
                    "stun": 50
                }
            },
            "nurse":
            {
                "fingerprint": "fingerprint_test"
            },
            "lana":
            {
                "event_path": "path/to/some/event/data",
                "prod": true
            },
            "paths":
            {
                "priority": ["relay", "direct"],
                "force": "relay"
            },
            "direct": {},
            "exit_dns": {}
        }"#;

    const CORRECT_FEATURES_JSON_WITH_ALL_OPTIONAL: &str = r#"
        {
            "wireguard":
            {
                "persistent_keepalive": {
                    "vpn": null,
                    "stun": 50
                }
            },
            "nurse":
            {
                "fingerprint": "fingerprint_test"
            },
            "lana":
            {
                "event_path": "path/to/some/event/data",
                "prod": true
            },
            "paths":
            {
                "priority": ["relay", "direct"],
                "force": "relay"
            },
            "direct": 
            {
                "providers": 42,
                "endpoint_interval_secs": 10,
                "skip_unresponsive_peers":
                {
                    "enabled": true,
                    "no_rx_threshold_secs": 50
                }
            },
            "exit_dns":
            {
                "auto_switch_dns_ips": true
            },
            "is_test_env": true,
            "derp":
            {
                "tcp_keepalive": 1,
                "derp_keepalive": 2,
                "enable_polling": true
            },
            "validate_keys": false,
            "ipv6": true,
            "nicknames": true,
            "boringtun_reset_connections": true,
            "post_quantum_vpn":
            {
                "handshake_retry_interval_s": 16,
                "rekey_interval_s": 120
            }
        }"#;

    static EXPECTED_FEATURES: Lazy<Features> = Lazy::new(|| Features {
        wireguard: FeatureWireguard {
            persistent_keepalive: FeaturePersistentKeepalive {
                vpn: None,
                direct: 5,
                proxying: Some(25),
                stun: Some(50),
            },
        },
        nurse: Some(FeatureNurse {
            fingerprint: "fingerprint_test".to_string(),
            heartbeat_interval: None,
            initial_heartbeat_interval: None,
            qos: None,
            enable_nat_type_collection: None,
        }),
        lana: Some(FeatureLana {
            event_path: "path/to/some/event/data".to_string(),
            prod: true,
        }),
        paths: Some(FeaturePaths {
            priority: vec![PathType::Relay, PathType::Direct],
            force: Some(PathType::Relay),
        }),
        direct: Some(FeatureDirect {
            providers: None,
            endpoint_interval_secs: Some(10),
            skip_unresponsive_peers: Some(FeatureSkipUnresponsivePeers {
                no_rx_threshold_secs: 50,
            }),
        }),
        exit_dns: Some(FeatureExitDns {
            auto_switch_dns_ips: Some(true),
        }),
        #[cfg(any(target_os = "macos", feature = "pretend_to_be_macos"))]
        is_test_env: Some(true),
        derp: Some(FeatureDerp {
            tcp_keepalive: Some(1),
            derp_keepalive: Some(2),
            enable_polling: Some(true),
            use_built_in_root_certificates: false,
        }),
        validate_keys: FeatureValidateKeys(false),
        ipv6: true,
        nicknames: true,
        boringtun_reset_connections: FeatureBoringtunResetConns(true),
        flush_events_on_stop_timeout_seconds: None,
        post_quantum_vpn: Some(FeaturePostQuantumVPN {
            handshake_retry_interval_s: 16,
            rekey_interval_s: 120,
        }),
    });

    static EXPECTED_FEATURES_WITHOUT_TEST_ENV: Lazy<Features> = Lazy::new(|| Features {
        wireguard: FeatureWireguard {
            persistent_keepalive: FeaturePersistentKeepalive {
                vpn: None,
                direct: 5,
                proxying: Some(25),
                stun: Some(50),
            },
        },
        nurse: Some(FeatureNurse {
            fingerprint: "fingerprint_test".to_string(),
            heartbeat_interval: None,
            initial_heartbeat_interval: None,
            qos: None,
            enable_nat_type_collection: None,
        }),
        lana: Some(FeatureLana {
            event_path: "path/to/some/event/data".to_string(),
            prod: true,
        }),
        paths: Some(FeaturePaths {
            priority: vec![PathType::Relay, PathType::Direct],
            force: Some(PathType::Relay),
        }),
        direct: Some(FeatureDirect {
            providers: None,
            endpoint_interval_secs: None,
            skip_unresponsive_peers: Some(Default::default()),
        }),
        exit_dns: Some(FeatureExitDns {
            auto_switch_dns_ips: None,
        }),
        #[cfg(any(target_os = "macos", feature = "pretend_to_be_macos"))]
        is_test_env: None,
        derp: None,
        validate_keys: Default::default(),
        ipv6: false,
        nicknames: false,
        boringtun_reset_connections: FeatureBoringtunResetConns(false),
        flush_events_on_stop_timeout_seconds: None,
        post_quantum_vpn: None,
    });

    #[test]
    fn test_json_direct_feature_set() {
        let full_json = r#"
        {
            "providers": ["local", "stun"],
            "endpoint_interval_secs": 30,
            "skip_unresponsive_peers": {
                "no_rx_threshold_secs": 42
            }
        }"#;

        let partial_json = r#"
        {
            "providers": ["local"]
        }"#;

        let full_features = FeatureDirect {
            providers: Some(
                vec![EndpointProvider::Local, EndpointProvider::Stun]
                    .into_iter()
                    .collect(),
            ),
            endpoint_interval_secs: Some(30),
            skip_unresponsive_peers: Some(FeatureSkipUnresponsivePeers {
                no_rx_threshold_secs: 42,
            }),
        };

        let partial_features = FeatureDirect {
            providers: Some(vec![EndpointProvider::Local].into_iter().collect()),
            endpoint_interval_secs: None,
            skip_unresponsive_peers: Some(Default::default()),
        };

        assert_eq!(from_str::<FeatureDirect>(full_json).unwrap(), full_features);
        assert_eq!(
            from_str::<FeatureDirect>(partial_json).unwrap(),
            partial_features
        );
    }

    #[test]
    fn test_json_direct_accepts_arbitrary_providers() {
        let arbitrery_providers_json = r#"
        {
            "providers": ["local", "stun", "other", "none"]
        }"#;

        let parsed_feature = FeatureDirect {
            providers: Some(
                vec![EndpointProvider::Local, EndpointProvider::Stun]
                    .into_iter()
                    .collect(),
            ),
            ..Default::default()
        };

        assert_eq!(
            from_str::<FeatureDirect>(arbitrery_providers_json).unwrap(),
            parsed_feature
        );
    }

    #[test]
    fn test_json_direct_allow_empty() {
        // Custom deserialize needs default to make Option None

        let empty_json = r#"{}"#;

        let parsed_feature = FeatureDirect::default();

        assert_eq!(
            from_str::<FeatureDirect>(empty_json).unwrap(),
            parsed_feature
        );
    }

    #[test]
    fn test_json_to_qos_feature_set() {
        let full_json = r#"
        {
            "rtt_interval": 3600,
            "rtt_tries": 5,
            "rtt_types": ["Ping"],
            "buckets": 5
        }"#;

        let partial_json = r#"
        {
            "rtt_interval": 3600
        }"#;

        let full_features = FeatureQoS {
            rtt_interval: Some(3600),
            rtt_tries: Some(5),
            rtt_types: Some(vec![String::from("Ping")]),
            buckets: Some(5),
        };

        let partial_features = FeatureQoS {
            rtt_interval: Some(3600),
            rtt_tries: None,
            rtt_types: None,
            buckets: None,
        };

        assert_eq!(from_str::<FeatureQoS>(full_json).unwrap(), full_features);
        assert_eq!(
            from_str::<FeatureQoS>(partial_json).unwrap(),
            partial_features
        );
    }

    #[test]
    #[cfg(not(any(target_os = "macos", feature = "pretend_to_be_macos")))]
    fn test_json_to_nurse_feature_set() {
        let full_json = r#"
        {
            "nurse": {
                "fingerprint": "fingerprint_test",
                "heartbeat_interval": 3600,
                "qos": {
                    "rtt_interval": 3600,
                    "rtt_tries": 5,
                    "rtt_types": ["Ping"],
                    "buckets": 5
                }
            }
        }"#;

        let empty_qos_json = r#"
        {
            "nurse": {
                "fingerprint": "fingerprint_test",
                "qos": {}
            }
        }"#;

        let no_qos_json = r#"
        {
            "nurse": {
                "fingerprint": "fingerprint_test"
            }
        }"#;

        let full_features = Features {
            wireguard: Default::default(),
            nurse: Some(FeatureNurse {
                fingerprint: String::from("fingerprint_test"),
                heartbeat_interval: Some(3600),
                initial_heartbeat_interval: None,
                qos: Some(FeatureQoS {
                    rtt_interval: Some(3600),
                    rtt_tries: Some(5),
                    rtt_types: Some(vec![String::from("Ping")]),
                    buckets: Some(5),
                }),
                enable_nat_type_collection: None,
            }),
            lana: None,
            paths: None,
            direct: None,
            exit_dns: None,
            derp: None,
            validate_keys: Default::default(),
            ipv6: false,
            nicknames: false,
            boringtun_reset_connections: Default::default(),
            flush_events_on_stop_timeout_seconds: None,
            post_quantum_vpn: Default::default(),
        };

        let empty_qos_features = Features {
            wireguard: Default::default(),
            nurse: Some(FeatureNurse {
                fingerprint: String::from("fingerprint_test"),
                heartbeat_interval: None,
                initial_heartbeat_interval: None,
                qos: Some(FeatureQoS {
                    rtt_interval: None,
                    rtt_tries: None,
                    rtt_types: None,
                    buckets: None,
                }),
                enable_nat_type_collection: None,
            }),
            lana: None,
            paths: None,
            direct: None,
            exit_dns: None,
            derp: None,
            validate_keys: Default::default(),
            ipv6: false,
            nicknames: false,
            boringtun_reset_connections: Default::default(),
            flush_events_on_stop_timeout_seconds: None,
            post_quantum_vpn: Default::default(),
        };

        let no_qos_features = Features {
            wireguard: Default::default(),
            nurse: Some(FeatureNurse {
                fingerprint: String::from("fingerprint_test"),
                heartbeat_interval: None,
                initial_heartbeat_interval: None,
                qos: None,
                enable_nat_type_collection: None,
            }),
            lana: None,
            paths: None,
            direct: None,
            exit_dns: None,
            derp: None,
            validate_keys: Default::default(),
            ipv6: false,
            nicknames: false,
            boringtun_reset_connections: Default::default(),
            flush_events_on_stop_timeout_seconds: None,
            post_quantum_vpn: Default::default(),
        };

        assert_eq!(from_str::<Features>(full_json).unwrap(), full_features);
        assert_eq!(
            from_str::<Features>(empty_qos_json).unwrap(),
            empty_qos_features
        );
        assert_eq!(from_str::<Features>(no_qos_json).unwrap(), no_qos_features);
    }

    #[test]
    #[cfg(not(any(target_os = "macos", feature = "pretend_to_be_macos")))]
    fn test_json_to_exit_dns_feature_set() {
        let full_json = r#"
        {
            "exit_dns": {
                "auto_switch_dns_ips": true
            }
        }"#;

        let empty_json = r#"
        {
            "exit_dns": {}
        }"#;

        let full_features = Features {
            wireguard: Default::default(),
            nurse: None,
            lana: None,
            paths: None,
            direct: None,
            exit_dns: Some(FeatureExitDns {
                auto_switch_dns_ips: Some(true),
            }),
            derp: None,
            validate_keys: Default::default(),
            ipv6: false,
            nicknames: false,
            boringtun_reset_connections: Default::default(),
            flush_events_on_stop_timeout_seconds: None,
            post_quantum_vpn: Default::default(),
        };

        let empty_features = Features {
            wireguard: Default::default(),
            nurse: None,
            lana: None,
            paths: None,
            direct: None,
            exit_dns: Some(FeatureExitDns {
                auto_switch_dns_ips: None,
            }),
            derp: None,
            validate_keys: Default::default(),
            ipv6: false,
            nicknames: false,
            boringtun_reset_connections: Default::default(),
            flush_events_on_stop_timeout_seconds: None,
            post_quantum_vpn: Default::default(),
        };

        assert_eq!(from_str::<Features>(full_json).unwrap(), full_features);
        assert_eq!(from_str::<Features>(empty_json).unwrap(), empty_features);
    }

    #[test]
    fn test_json_allow_null_for_wireguard() {
        let empty_json = r#"
        {
            "wireguard": null
        }"#;

        let empty_features = Features {
            wireguard: Default::default(),
            nurse: None,
            lana: None,
            paths: None,
            direct: None,
            #[cfg(any(target_os = "macos", feature = "pretend_to_be_macos"))]
            is_test_env: None,
            exit_dns: None,
            derp: None,
            validate_keys: Default::default(),
            ipv6: false,
            nicknames: false,
            boringtun_reset_connections: Default::default(),
            flush_events_on_stop_timeout_seconds: None,
            post_quantum_vpn: Default::default(),
        };

        assert_eq!(from_str::<Features>(empty_json).unwrap(), empty_features);
    }

    #[test]
    fn test_json_fail_on_invalid_wireguard() {
        let empty_json = r#"
        {
            "wireguard": 123
        }"#;

        assert!(from_str::<Features>(empty_json).is_err());
    }

    #[test]
    fn test_json_without_is_test_env_to_feature_set() {
        let deserialization: Result<Features, _> =
            from_str(CORRECT_FEATURES_JSON_WITHOUT_IS_TEST_ENV);

        assert_eq!(
            deserialization.unwrap(),
            *EXPECTED_FEATURES_WITHOUT_TEST_ENV
        );
    }

    #[test]
    fn test_json_with_is_test_env_to_feature_set() {
        let deserialization: Result<Features, _> =
            from_str(CORRECT_FEATURES_JSON_WITH_ALL_OPTIONAL);

        assert_eq!(deserialization.unwrap(), *EXPECTED_FEATURES);
    }

    #[test]
    #[cfg(not(any(target_os = "macos", feature = "pretend_to_be_macos")))]
    fn test_default_features() {
        let expected_defaults = Features {
            wireguard: Default::default(),
            nurse: None,
            lana: None,
            paths: None,
            exit_dns: None,
            direct: None,
            derp: None,
            validate_keys: Default::default(),
            ipv6: false,
            nicknames: false,
            boringtun_reset_connections: Default::default(),
            flush_events_on_stop_timeout_seconds: None,
            post_quantum_vpn: Default::default(),
        };

        assert_eq!(Features::default(), expected_defaults);
    }

    #[test]
    fn get_paths_from_feature_paths() {
        assert_eq!(
            FeaturePaths {
                priority: vec![PathType::Direct],
                force: None
            }
            .paths(),
            vec![PathType::Relay, PathType::Direct]
        );
        assert_eq!(
            FeaturePaths {
                priority: vec![PathType::Direct, PathType::Relay, PathType::Direct],
                force: None
            }
            .paths(),
            vec![PathType::Relay, PathType::Direct]
        );
        assert_eq!(
            FeaturePaths {
                priority: vec![PathType::Direct, PathType::Relay, PathType::Direct],
                force: Some(PathType::Direct)
            }
            .paths(),
            vec![PathType::Direct]
        );
    }
}
