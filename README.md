# simbridge

Sim racing telemetry dashboards and audio effects for Linux. A SimHub replacement that runs natively.

## How it works

simbridge has two parts:

- **simbridge.exe** — a small Windows binary that runs inside your game's Proton prefix. It reads the game's shared memory and sends telemetry over UDP.
- **simbridge-server** — a native Linux server that receives the telemetry, serves web dashboards, and drives audio effects (bass shakers, etc).

The server starts on login as a systemd user service. Open the dashboards on your phone or a second screen during a race.

## Supported games

| Game | Method | Notes |
|------|--------|-------|
| Assetto Corsa Competizione | Bridge | Tested, working |
| Assetto Corsa | Bridge | Built, untested |
| Assetto Corsa Evo | Bridge | Built, untested |
| Assetto Corsa Rally | Bridge | Built, untested |
| rFactor 2 | Bridge | Built, untested |
| BeamNG.drive | Bridge | Built, untested |
| Dirt Rally 2.0 | Native UDP | No bridge needed |

## Dashboards

- **Circuit** — DDU-style dashboard (speed, RPM, gear, temps, tyre wear, lap times)
- **Rally** — rally-focused layout (stage info, split times, surface)
- **Lite** — minimal view for older devices

All dashboards work on any device with a browser. Open your phone's browser and point it at your PC.

## Install

### From a release (recommended)

Download the latest release tarball from [Releases](../../releases), extract it, and run the installer:

```bash
tar xzf simbridge-v*-linux-x86_64.tar.gz
cd simbridge-v*-linux-x86_64
./install.sh
```

### From source

Requires Rust, the `x86_64-pc-windows-gnu` target, and `mingw-w64`:

```bash
# Install prerequisites (Arch/CachyOS)
sudo pacman -S mingw-w64-gcc
rustup target add x86_64-pc-windows-gnu

# Build and install
git clone https://github.com/Carlos-Diaz-07/simbridge.git
cd simbridge
./install.sh
```

## What the installer does

- Builds both binaries (if installing from source)
- Copies `simbridge.exe`, `simbridge-server`, and `simbridge-launch` to `~/.local/bin/`
- Installs and starts a systemd user service (`simbridge-server.service`)
- Adds a desktop entry and icon to your app launcher

## Steam launch options

Add the launch wrapper to your game's Steam launch options. Replace `%command%` — Steam fills that in automatically.

```
simbridge-launch acc %command%
```

Available game adapters: `acc`, `ac`, `acevo`, `acrally`, `rf2`, `beamng`

You can keep your existing launch options (MangoHud, gamemode, etc.) — just prepend the simbridge wrapper:

```
MESA_SHADER_CACHE_MAX_SIZE=12G mangohud simbridge-launch acc %command%
```

### Dirt Rally 2.0 (native UDP)

DR2 has built-in Codemasters UDP telemetry. No bridge needed — just enable it:

Edit `~/.local/share/Steam/steamapps/common/DiRT Rally 2.0/dirtrally2/hardware_settings/hardware_settings_config.xml`:

```xml
<udp enabled="true" extradata="3" ip="127.0.0.1" port="20777" />
```

Then launch DR2 normally. The simbridge server picks up the UDP packets automatically.

## Usage

Once installed, simbridge-server runs in the background. Open the admin panel from your app launcher or go to:

- **Admin panel:** http://localhost:8888
- **Dashboard:** http://YOUR_IP:8888/dash (use the URL shown in the admin panel)

The admin panel shows connection status, lets you switch dashboard modes, and configure audio effects.

## Uninstall

```bash
./uninstall.sh
```

Removes all binaries, the systemd service, the desktop entry, and the icon.

## License

MIT
