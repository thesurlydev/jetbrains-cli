# JetBrains CLI

The JetBrains family of IDEs (IntelliJ IDEA, PyCharm, etc.) typically have a built-in web server.

This tool can be used to:

1. List the installed IDE instances.
2. Find the port that the web server is listening on for each IDE instance.
3. Get the IDE configuration.


## Features

## JetBrains IDE Logs


The log file name is `idea.log` and is located under the install directory of each IDE instance.

### Examples

Examples for IntelliJ IDEA 2022.2:

#### Windows:

* Syntax: `%LOCALAPPDATA%\JetBrains\<product><version>\log`
* Logs (idea.log.path): `%LOCALAPPDATA%\JetBrains\IntelliJIdea2022.2\log`

#### macOS:

* Syntax: `~/Library/Logs/JetBrains/<product><version>`
* Logs (idea.log.path): `~/Library/Logs/JetBrains/IntelliJIdea2022.2`

#### Linux:

* Syntax: `~/.cache/JetBrains/<product><version>/log`
* Logs (idea.log.path): `~/.cache/JetBrains/IntelliJIdea2022.2/log`


<product> would be one of the following:

* IntelliJIdea (IntelliJ IDEA Ultimate Edition)
* IdeaIC (IntelliJ IDEA Community Edition)
* RubyMine
* WebIde (PhpStorm versions before 2016.1 and WebStorm before 7.0 use this common directory)
* PhpStorm (PhpStorm starting from 2016.1 version)
* WebStorm (WebStorm starting from 7.0 version)
* PyCharm (PyCharm Professional)
* PyCharmCE (PyCharm Community)
* AppCode
* CLion
* DataGrip
* Rider
