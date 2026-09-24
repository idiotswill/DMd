DMd Windows desktop

Run the DMd setup executable to install for the current Windows user. The installer
contains the WebView2 installer for machines that need it, including offline setup.
After installation, open DMd from the Start menu.

The supplementary portable folder contains DMd.exe and its required content and
license resources. Keep that folder intact. It requires an installed WebView2 runtime.
The installer is the supported route for a machine without WebView2.

This early desktop infrastructure build does not yet connect the table controls.
It is not a completed Gate 3 playable build. No developer server is required to run it.

build-info.json identifies the exact source commit and target. SHA256SUMS.txt records
the packaged file hashes. portable/licenses/dependencies contains license declarations
and available upstream notice texts, including build tooling. SRD attribution and
license notices remain alongside the rules content.
