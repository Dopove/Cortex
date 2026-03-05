# Cortex 2.5.9: Quickstart Guide

Get up and running with Cortex 2.5.9 in less than 5 minutes.

## 1. Installation

Cortex can be installed globally with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/Dopove/Cortex/main/install.sh | bash
```

Alternatively, you can move the `cortex` binary manually to your PATH.

## 2. Initialize the Environment

Cortex needs to verify your Python environment and hardware capabilities before its first run.

```bash
./cortex init
```

## 3. Create Your First Bundle

Cortex bundles contain your agent logic, dependencies, and model manifests.

```bash
# Build your agent directory
./cortex build path/to/your/agent-dir my-agent.cortex
```

## 4. Run Your Agent

Execute your bundle. Cortex will automatically:

1. Create an isolated virtual environment.
2. Install any `requirements.txt` found in the bundle.
3. Start the agent(s) defined in the manifest.

```bash
./cortex run my-first-agent.cortex
```

## 5. Security: Encrypting Your Work (Optional)

To share bundles securely, add AES-256-GCM encryption.

```bash
# Set your password
export CORTEX_BUNDLE_PASSWORD="my-secure-password"

# Encrypt the bundle
./cortex encrypt my-first-agent.cortex

# Now, any run/info command will require the correct password in the environment
./cortex info my-first-agent.cortex
```

## 🚀 Next Steps

- Explore the [Usage Reference](./USAGE_REFERENCE.md) for all CLI subcommands.
- Review [System Requirements](./REQUIREMENTS.md) for large-scale model deployments.
