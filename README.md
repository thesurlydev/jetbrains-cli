# JetBrains CLI

A command-line tool for querying information about installed JetBrains IDEs. It helps you:

* List installed IDE instances
* Check which IDEs are currently running
* Get configuration details including ports and VM options

## Usage

### List Command

List all installed JetBrains IDEs:

```bash
# Text output (default)
jb list

# JSON output
jb list --output json

# Include IDEs without log files
jb list --verbose
```

Example output:
```
Installed IDEs:
  RustRover2024.3
    Install directory: /Users/shane/Applications/RustRover.app
    Running: Yes

  WebStorm2024.3
    Install directory: /Users/shane/Applications/WebStorm.app
    Running: No
```

### Config Command

Get detailed configuration for a specific IDE:

```bash
# Text output (default)
jb config --name IntelliJIdea2024.3

# JSON output
jb config --name IntelliJIdea2024.3 --output json
```

Example output:
```
Configuration for IntelliJIdea2024.3:
  Install directory: /Applications/IntelliJ IDEA.app
  Config directory: /Users/shane/Library/Application Support/JetBrains/IntelliJIdea2024.3
  Logs directory: /Users/shane/Library/Logs/JetBrains/IntelliJIdea2024.3
  VM Options:
    -Xmx2048m
    -Dide.managed.by.toolbox=/Applications/JetBrains Toolbox.app/Contents/MacOS/jetbrains-toolbox
  Port: 63343
  Running: Yes
```

## Features

### IDE Detection

The tool automatically detects installed JetBrains IDEs by scanning the standard installation directories. It supports all major JetBrains IDEs including:

- IntelliJ IDEA (Ultimate and Community)
- WebStorm
- RustRover
- PyCharm (Professional and Community)
- CLion
- PhpStorm
- Rider
- DataGrip

### Running Status

For each IDE, the tool can determine if it's currently running by checking if its web server port is in use.

### Configuration Details

The tool provides access to important configuration details:
- Installation directory
- Configuration directory
- Logs directory
- VM options
- Web server port
- Running status
x