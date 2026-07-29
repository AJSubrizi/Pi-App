# Providers and models

Settings → **Providers & Models** manages routing without turning Pi App into a
second provider or package manager.

## Sources of truth

| Data | Owner |
|------|-------|
| Provider endpoint, protocol, key, default route | Active Pi agent profile `config.toml` |
| Available model catalog | `pi --list-models` |
| Provider adapters and optional capabilities | Pi packages installed through `pi install` |

The active profile follows `sessionDataMode`: App-owned in independent mode,
and the shared Pi profile in shared mode.

## Host commands

| Command | Role |
|---------|------|
| `providers_list` | Return configured providers without returning raw keys |
| `providers_upsert` | Add or update a provider |
| `providers_remove` | Remove one provider section |
| `providers_activate` | Switch between the official route and a custom route |
| `providers_set_default` | Update the Pi default model/route |
| `providers_ping` | Test the provider models endpoint |
| `providers_list_models` | Fetch model identifiers from a provider |

## Package boundary

`pi.dev/packages` is a package directory, not an implicit provider registry.
Only explicitly reviewed, pinned packages may be offered as provider adapters.
Installation remains in Settings → Extensions and uses the existing trust
review before invoking Pi CLI.

API keys are never returned to the frontend after saving and must not be copied
into App-owned metadata or logs.
