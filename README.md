# Effortless Configuration Management with check-config

**Keep your development environment consistent, shareable, and version-controlled**

check-config is a fast, lightweight, declarative configuration management tool that ensures your configuration files
contain exactly what they should. Instead of managing entire config files,
you declare specific parts that must be present - making configurations shareable, maintainable, and verifiable.

## How It Works

Define your configuration requirements in simple TOML files, then let check-config ensure they're applied:

```toml
# Set your preferred editor
[[lines_present]]
file = "~/.bashrc"
lines = "export EDITOR=hx"
```

```toml
# Ensure git signing is configured
[[lines_present]]
file = "~/.gitconfig"
lines = """
[gpg]
        format = ssh
[commit]
        gpgsign = true
"""
```

Run `check-config --fix` to apply changes, or `check-config` to verify everything is in sync.

## Why check-config?

### 🔧 **Shareable Configuration Snippets**

Traditional dotfile repositories force users to adopt entire configuration files. check-config lets you share just the essential parts:
- Share your preferred Python formatting rules without forcing your entire `pyproject.toml`
- Distribute security settings without overwriting personal aliases
- Collaborate on team standards while preserving individual preferences

### ✅ **Enforce Team Standards**

Ensure consistent development environments across your team:

```shell
# In CI: Verify configurations are up-to-date
check-config

# For developers: Apply required configurations  
check-config --fix
```

Perfect for ensuring tools like Ruff, Black, and ESLint use consistent settings across all developers and CI pipelines.

### 📦 **Composable Configuration**

Combine multiple configuration files to build your complete setup:
- Base configurations for your team
- Personal tweaks and preferences  
- Project-specific requirements
- Environment-specific overrides

## Beyond Simple Lines

check-config supports multiple checker types for different configuration needs:
- **Lines present/absent**: Shell configs, text files
- **Key-value pairs**: TOML, JSON, YAML files
- **File existence**: Ensure critical files exist
- **And more**: See [docs/checkers.md](docs/checkers.md) for all features

## Get Started

Make a `check-config.toml` according your needs:

```toml
# Set your preferred editor
[[lines_present]]
file = "~/.bashrc"
lines = "export EDITOR=hx"
```

And use it:

```shell
# Check if configurations match requirements
check-config

# Apply missing configurations
check-config --fix
```

## Documentation

📖 **[Full Documentation](https://check-config.readthedocs.io)** - Complete guides, examples, and API reference

---

*Declare what you need. Share what matters. Keep everything in sync.*
