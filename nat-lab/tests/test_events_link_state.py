import asyncio
import pytest
from contextlib import AsyncExitStack
from helpers import SetupParameters, setup_mesh_nodes
from telio import AdapterType, LinkState
from typing import List, Tuple
from utils.bindings import (
    default_features,
    FeatureLinkDetection,
    FeatureWireguard,
    FeaturePersistentKeepalive,
)
from utils.connection_util import ConnectionTag
from utils.ping import ping


def long_persistent_keepalive_periods() -> FeatureWireguard:
    return FeatureWireguard(
        persistent_keepalive=FeaturePersistentKeepalive(
            proxying=3600, direct=3600, vpn=3600, stun=3600
        )
    )


def _generate_setup_parameter_pair(
    cfg: List[Tuple[ConnectionTag, AdapterType]],
    enhaced_detection: bool,
) -> List[SetupParameters]:
    if enhaced_detection:
        count = 1
    else:
        count = 0

    features = default_features(enable_link_detection=True)
    features.link_detection = FeatureLinkDetection(
        rtt_seconds=1, no_of_pings=count, use_for_downgrade=False
    )
    features.wireguard = long_persistent_keepalive_periods()

    return [
        SetupParameters(
            connection_tag=tag,
            adapter_type=adapter,
            features=features,
        )
        for tag, adapter in cfg
    ]


FEATURE_ENABLED_PARAMS = [
    # This scenario has been removed because it was causing flakyness due to LLT-5014.
    # Add it back when the issue is fixed.
    # pytest.param(
    #     _generate_setup_paramete_pair([
    #         (ConnectionTag.DOCKER_CONE_CLIENT_1, AdapterType.LinuxNativeWg),
    #         (ConnectionTag.DOCKER_CONE_CLIENT_2, AdapterType.LinuxNativeWg),
    #     ])
    # ),
    pytest.param(
        _generate_setup_parameter_pair(
            [
                (ConnectionTag.DOCKER_CONE_CLIENT_1, AdapterType.LinuxNativeWg),
                (ConnectionTag.DOCKER_CONE_CLIENT_2, AdapterType.BoringTun),
            ],
            False,
        )
    ),
    pytest.param(
        _generate_setup_parameter_pair(
            [
                (ConnectionTag.DOCKER_CONE_CLIENT_1, AdapterType.LinuxNativeWg),
                (ConnectionTag.DOCKER_CONE_CLIENT_2, AdapterType.BoringTun),
            ],
            True,
        )
    ),
    pytest.param(
        _generate_setup_parameter_pair(
            [
                (ConnectionTag.DOCKER_CONE_CLIENT_1, AdapterType.BoringTun),
                (ConnectionTag.DOCKER_CONE_CLIENT_2, AdapterType.BoringTun),
            ],
            False,
        )
    ),
    pytest.param(
        _generate_setup_parameter_pair(
            [
                (ConnectionTag.DOCKER_CONE_CLIENT_1, AdapterType.BoringTun),
                (ConnectionTag.DOCKER_CONE_CLIENT_2, AdapterType.BoringTun),
            ],
            True,
        )
    ),
    pytest.param(
        _generate_setup_parameter_pair(
            [
                (ConnectionTag.WINDOWS_VM_1, AdapterType.WindowsNativeWg),
                (ConnectionTag.DOCKER_CONE_CLIENT_1, AdapterType.BoringTun),
            ],
            False,
        ),
        marks=pytest.mark.windows,
    ),
    pytest.param(
        _generate_setup_parameter_pair(
            [
                (ConnectionTag.WINDOWS_VM_1, AdapterType.WindowsNativeWg),
                (ConnectionTag.DOCKER_CONE_CLIENT_1, AdapterType.BoringTun),
            ],
            True,
        ),
        marks=pytest.mark.windows,
    ),
]

FEATURE_DISABLED_PARAMS = [
    pytest.param([
        SetupParameters(
            connection_tag=ConnectionTag.DOCKER_CONE_CLIENT_1,
            adapter_type=AdapterType.BoringTun,
        ),
        SetupParameters(
            connection_tag=ConnectionTag.DOCKER_CONE_CLIENT_2,
            adapter_type=AdapterType.BoringTun,
        ),
    ])
]


@pytest.mark.asyncio
@pytest.mark.parametrize("setup_params", FEATURE_ENABLED_PARAMS)
async def test_event_link_state_peers_idle_all_time(
    setup_params: List[SetupParameters],
) -> None:
    async with AsyncExitStack() as exit_stack:
        env = await setup_mesh_nodes(exit_stack, setup_params)
        alpha, beta = env.nodes
        client_alpha, client_beta = env.clients

        # Expect no link event while peers are idle
        await asyncio.sleep(20)
        alpha_events = client_beta.get_link_state_events(alpha.public_key)
        beta_events = client_alpha.get_link_state_events(beta.public_key)

        # 1 down when node is Connecting, 1 up when still Connecting and 1 up when node is Connected
        assert alpha_events == [LinkState.Down, LinkState.Up, LinkState.Up]
        assert beta_events == [LinkState.Down, LinkState.Up, LinkState.Up]


@pytest.mark.asyncio
@pytest.mark.parametrize("setup_params", FEATURE_ENABLED_PARAMS)
async def test_event_link_state_peers_exchanging_data_for_a_long_time(
    setup_params: List[SetupParameters],
) -> None:
    async with AsyncExitStack() as exit_stack:
        env = await setup_mesh_nodes(exit_stack, setup_params)
        alpha, beta = env.nodes
        client_alpha, client_beta = env.clients
        connection_alpha, connection_beta = [
            conn.connection for conn in env.connections
        ]

        for _ in range(0, 40):
            await asyncio.sleep(1)
            await ping(connection_alpha, beta.ip_addresses[0])
            await ping(connection_beta, alpha.ip_addresses[0])

        # Expect no nolink event while peers are active
        alpha_events = client_beta.get_link_state_events(alpha.public_key)
        beta_events = client_alpha.get_link_state_events(beta.public_key)

        # 1 down when node is Connecting, 1 up when still Connecting and 1 up when node is Connected
        assert alpha_events == [LinkState.Down, LinkState.Up, LinkState.Up]
        assert beta_events == [LinkState.Down, LinkState.Up, LinkState.Up]


@pytest.mark.asyncio
@pytest.mark.parametrize("setup_params", FEATURE_ENABLED_PARAMS)
async def test_event_link_state_peers_exchanging_data_then_idling_then_resume(
    setup_params: List[SetupParameters],
) -> None:
    async with AsyncExitStack() as exit_stack:
        env = await setup_mesh_nodes(exit_stack, setup_params)
        alpha, beta = env.nodes
        client_alpha, client_beta = env.clients
        connection_alpha, connection_beta = [
            conn.connection for conn in env.connections
        ]

        await ping(connection_alpha, beta.ip_addresses[0])
        await ping(connection_beta, alpha.ip_addresses[0])

        # Expect no link event while peers are idle
        await asyncio.sleep(20)

        await ping(connection_alpha, beta.ip_addresses[0])
        await ping(connection_beta, alpha.ip_addresses[0])

        alpha_events = client_beta.get_link_state_events(alpha.public_key)
        beta_events = client_alpha.get_link_state_events(beta.public_key)

        # 1 down when node is Connecting, 1 up when still Connecting and 1 up when node is Connected
        assert alpha_events == [LinkState.Down, LinkState.Up, LinkState.Up]
        assert beta_events == [LinkState.Down, LinkState.Up, LinkState.Up]


@pytest.mark.asyncio
@pytest.mark.parametrize("setup_params", FEATURE_ENABLED_PARAMS)
async def test_event_link_state_peer_goes_offline(
    setup_params: List[SetupParameters],
) -> None:
    async with AsyncExitStack() as exit_stack:
        env = await setup_mesh_nodes(exit_stack, setup_params)
        alpha, beta = env.nodes
        client_alpha, client_beta = env.clients
        connection_alpha, connection_beta = [
            conn.connection for conn in env.connections
        ]

        await ping(connection_alpha, beta.ip_addresses[0])
        await ping(connection_beta, alpha.ip_addresses[0])

        await client_beta.stop_device()

        await asyncio.sleep(1)

        with pytest.raises(asyncio.TimeoutError):
            await ping(connection_alpha, beta.ip_addresses[0], 5)

        await asyncio.sleep(25)
        alpha_events = client_beta.get_link_state_events(alpha.public_key)
        beta_events = client_alpha.get_link_state_events(beta.public_key)

        # 1 down when node is Connecting, 1 up when still Connecting and 1 up when node is Connected
        assert alpha_events == [LinkState.Down, LinkState.Up, LinkState.Up]
        # beta will have 2 down events: 1 when is Connecting and 1 detected and 2 up when Connecting and Connected
        assert beta_events == [
            LinkState.Down,
            LinkState.Up,
            LinkState.Up,
            LinkState.Down,
        ]


@pytest.mark.asyncio
@pytest.mark.parametrize("setup_params", FEATURE_DISABLED_PARAMS)
async def test_event_link_state_feature_disabled(
    setup_params: List[SetupParameters],
) -> None:
    async with AsyncExitStack() as exit_stack:
        env = await setup_mesh_nodes(exit_stack, setup_params)
        alpha, beta = env.nodes
        client_alpha, client_beta = env.clients
        connection_alpha, connection_beta = [
            conn.connection for conn in env.connections
        ]

        await ping(connection_alpha, beta.ip_addresses[0])
        await ping(connection_beta, alpha.ip_addresses[0])

        await asyncio.sleep(30)

        alpha_events = client_beta.get_link_state_events(alpha.public_key)
        beta_events = client_alpha.get_link_state_events(beta.public_key)

        assert len(alpha_events) > 1
        assert len(beta_events) > 1
        assert all(e is None for e in alpha_events)
        assert all(e is None for e in beta_events)
