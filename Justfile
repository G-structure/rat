set shell := ["bash", "-cu"]

# Run the simulator example directly
sim-agent:
    cargo run -q --example sim_agent -- --scenario happy_path_edit --speed fast

# Run RAT against the simulator (no network/credits)
# Usage: just sim
sim:
    cargo run -q -- \
      --agent-cmd cargo \
      --agent-arg run --agent-arg --quiet \
      --agent-arg --example --agent-arg sim_agent \
      --agent-arg -- \
      --agent-arg --scenario --agent-arg happy_path_edit \
      --agent-arg --speed --agent-arg fast \
      --agent sim

