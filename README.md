# TimeTrax

A command-line time tracker that records daily work activities in JSON files.

## Installation

From crates.rs:

```bash
cargo install timetrax
```

Cutting edge from GitHub:

```bashbash
cargo install --git https://github.com/0xCCF4/timetrax
```

Or via nix:

```bash
nix run github:0xCCF4/timetrax --
```

## Shell completions

```bash
timetrax completion --shell bash >> ~/.bashrc
timetrax completion --shell zsh  >> ~/.zshrc
timetrax completion --shell fish > ~/.config/fish/completions/timetrax.fish
```

## Data model
TimeTrax organizes time tracking via the following paradigm:
- **Day**: Represents a calendar day, containing multiple activities.
- **Activity**: Represents a specific task or project, having a start time, end time, and further properties.
- **Project**: A higher-level categorization for activities, activities can be associated with a project.
- **Classification**: A label applied to activities for categorization, e.g., "work", "break", "holiday".
- **Job**: A job holds information about target working hours.

### Time tracking attribution
At any point in time, several activities can be active simultaneously, e.g., "working on project A", 
"meeting with team", and "lunch break". TimeTrax will attribute the time spend to the highest priority
classification. If "working on project A" and "meeting with team" classified as "work" (priority 1) and
"lunch break" classified as "break" (priority 2), the time when all three are active will be attributed to
"break".


## Logging

Set `RUST_LOG=debug` (or `trace`) for verbose output:

```bash
RUST_LOG=debug timetrax status
```
