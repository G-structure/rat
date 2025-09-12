set shell := ["bash", "-cu"]

# Default scenario
SCENARIO := "happy-path-edit"
SPEED := "fast"

# Run the simulator example directly with default scenario
sim-agent:
    cargo run -q --example sim_agent -- --scenario {{SCENARIO}} --speed {{SPEED}}

# Run RAT against the simulator (no network/credits) with default scenario
# Usage: just sim
sim:
    cargo run -q -- \
      --agent-cmd cargo \
      --agent-arg run --agent-arg --quiet \
      --agent-arg --example --agent-arg sim_agent \
      --agent-arg -- \
      --agent-arg --scenario --agent-arg {{SCENARIO}} \
      --agent-arg --speed --agent-arg {{SPEED}} \
      --agent sim

# Run specific scenarios
sim-happy-path:
    cargo run -q -- \
      --agent-cmd cargo \
      --agent-arg run --agent-arg --quiet \
      --agent-arg --example --agent-arg sim_agent \
      --agent-arg -- \
      --agent-arg --scenario --agent-arg happy-path-edit \
      --agent-arg --speed --agent-arg {{SPEED}} \
      --agent sim

sim-failure-path:
    cargo run -q -- \
      --agent-cmd cargo \
      --agent-arg run --agent-arg --quiet \
      --agent-arg --example --agent-arg sim_agent \
      --agent-arg -- \
      --agent-arg --scenario --agent-arg failure-path \
      --agent-arg --speed --agent-arg {{SPEED}} \
      --agent sim

sim-images-thoughts:
    cargo run -q -- \
      --agent-cmd cargo \
      --agent-arg run --agent-arg --quiet \
      --agent-arg --example --agent-arg sim_agent \
      --agent-arg -- \
      --agent-arg --scenario --agent-arg images-and-thoughts \
      --agent-arg --speed --agent-arg {{SPEED}} \
      --agent sim

sim-commands-update:
    cargo run -q -- \
      --agent-cmd cargo \
      --agent-arg run --agent-arg --quiet \
      --agent-arg --example --agent-arg sim_agent \
      --agent-arg -- \
      --agent-arg --scenario --agent-arg commands-update \
      --agent-arg --speed --agent-arg {{SPEED}} \
      --agent sim

# Run simulator directly with specific scenarios
sim-agent-happy-path:
    cargo run -q --example sim_agent -- --scenario happy-path-edit --speed {{SPEED}}

sim-agent-failure-path:
    cargo run -q --example sim_agent -- --scenario failure-path --speed {{SPEED}}

sim-agent-images-thoughts:
    cargo run -q --example sim_agent -- --scenario images-and-thoughts --speed {{SPEED}}

sim-agent-commands-update:
    cargo run -q --example sim_agent -- --scenario commands-update --speed {{SPEED}}

# Run with custom scenario and speed
# Usage: just sim-custom SCENARIO=failure-path SPEED=normal
sim-custom scenario=SCENARIO speed=SPEED:
    cargo run -q -- \
      --agent-cmd cargo \
      --agent-arg run --agent-arg --quiet \
      --agent-arg --example --agent-arg sim_agent \
      --agent-arg -- \
      --agent-arg --scenario --agent-arg {{scenario}} \
      --agent-arg --speed --agent-arg {{speed}} \
      --agent sim

# Run simulator directly with custom parameters
sim-agent-custom scenario=SCENARIO speed=SPEED:
    cargo run -q --example sim_agent -- --scenario {{scenario}} --speed {{speed}}

test:
    RUSTFLAGS="-Awarnings" cargo test --quiet
