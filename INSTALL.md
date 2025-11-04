# Cortex Installation Guide

This guide provides step-by-step instructions for setting up the development environment for Cortex v0.4.0 (Beta Release), which includes installing its prerequisite, Mojo, and building the Cortex binary from source.

## Prerequisites

Before you begin, ensure you have the following installed on your system:
- **`curl`**, **`tar`**: For downloading and extracting files.
- **`git`**: For cloning the source code repository.
- **Python 3.11+**: For running AI agents.

## Step 1: Install Mojo

Mojo is the programming language Cortex is built with and is a required prerequisite.

### 1.1. Download the Mojo SDK

The recommended way to install Mojo is by using the `modular` command-line tool, which manages Mojo installations.

```bash
curl -s https://get.modular.com | sh -
```

This command will download and run the Modular installer script. Follow the on-screen instructions to complete the installation.

### 1.2. Install Mojo

Once the Modular tool is installed, use it to install the latest version of the Mojo SDK:

```bash
modular install mojo
```

The installer will add Mojo to your system's PATH. You may need to restart your terminal or source your shell's profile file (e.g., `~/.bashrc`, `~/.zshrc`) for the changes to take effect.

### 1.3. Verify Mojo Installation

To confirm that Mojo has been installed correctly, run the following command:

```bash
mojo --version
```

You should see the installed Mojo version printed to the console.

---

## Step 2: Build Cortex from Source

Once Mojo is installed, you can build the Cortex binary from the source code.

### 2.1. Clone the Cortex Repository

First, clone the official Cortex repository from GitHub to your local machine:

```bash
git clone https://github.com/your-username/cortex.git
cd cortex
```
*Note: Replace `your-username` with the actual GitHub username or organization where the repository is hosted.*

### 2.2. Build the Cortex Binary

Use the Mojo compiler to build the `cortex` executable from the main CLI file. This command will create an optimized binary named `cortex` in the project's root directory.

```bash
mojo build src/cortex_cli.mojo -o cortex
```

### 2.3. Verify Cortex Installation

Check that the binary was built successfully by running the `version` command:

```bash
./cortex --version
```

This should display the current Cortex version, for example: `Cortex v0.4.0 - Multi-Agent Bundler`.

---

## Step 3: (Optional) Move to a Global Path

For easier access, you can move the compiled `cortex` binary to a directory in your system's `PATH`, such as `/usr/local/bin`:

```bash
sudo mv cortex /usr/local/bin/cortex
```

After moving it, you can run Cortex from any directory by simply typing `cortex`.

```bash
cortex --version
```

You are now ready to start bundling and running AI agents with Cortex!
