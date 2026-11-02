# Installed extension audit

Checked on 2026-11-02 against local VS Code package manifests. No extension
JavaScript was activated. This is a native data-contribution check, not a
claim of full extension compatibility.

## Packages with working theme/snippet data

- burkeholland.simple-react-snippets: 0 themes, 132 snippets; code features require a host
- dsznajder.es7-react-js-snippets: 0 themes, 700 snippets; code features require a host
- GitHub.github-vscode-theme: 9 themes, 0 snippets
- johnpapa.winteriscoming: 6 themes, 0 snippets
- keyring.Lua: 0 themes, 187 snippets
- mathiasfrohlich.Kotlin: 0 themes, 24 snippets
- monokai.theme-monokai-pro-vscode: 8 themes, 0 snippets; code features require a host
- ms-vscode.powershell: 1 themes, 62 snippets; code features require a host
- NativeBase.nativebase-v3-vscode-extension: 0 themes, 240 snippets
- tauri-apps.tauri-vscode: 0 themes, 6 snippets; code features require a host
- teabyii.ayu: 6 themes, 0 snippets
- truman.autocomplate-shell: 0 themes, 164 snippets
- vinliao.fast-arrow: 0 themes, 20 snippets

The 13 packages expose 30 themes and 1,535 language-specific snippet entries.
Basic snippet expansion supports nested defaults, initial mirrored values,
first choices and file/selection variables; regex transforms and linked
placeholder editing/tab navigation are not supported. Mixed code/data
extensions retain unavailable code features.

## Other installed packages

These packages have no native contribution supported by this build. Their
code, grammars, icon themes, extension packs and other APIs need additional
implementation. In particular, language servers, AI assistants, remote tools,
webviews and debugger extensions are not activated.

- sumneko.lua
- GitHub.copilot-chat
- ms-python.vscode-python-envs
- ms-vscode-remote.remote-containers
- vscjava.vscode-java-upgrade
- vscjava.vscode-java-debug
- rust-lang.rust-analyzer
- 42Crunch.vscode-openapi
- AakashP.adb-qr
- Angular.ng-template
- Anthropic.claude-code
- Arjun.swagger-viewer
- arturock.gitstash
- atlassian.atlascode
- bbvmlabs.wireless-debug-extension
- be5invis.vscode-icontheme-nomo-dark
- blanu.vscode-styled-jsx
- bradlc.vscode-tailwindcss
- CodeSandbox-io.codesandbox-projects
- Continue.continue
- corona10.muse-code-unofficial
- dbaeumer.vscode-eslint
- DenizhanDaklr.muse-code-for-copilot
- dgileadi.java-decompiler
- dhruv.maven-dependency-explorer
- docker.docker
- eamodio.gitlens
- expo.vscode-expo-tools
- FallenMax.mithril-emmet
- fisher.xlsx-tools
- flowtype.flow-for-vscode
- formulahendry.docker-explorer
- formulahendry.docker-extension-pack
- fwcd.kotlin
- Gaussian.gaussian-codex-for-copilot
- google.geminicodeassist
- google.google-antigravity
- GrapeCity.gc-excelviewer
- grikomsn.openai-oauth-copilot-chat
- lllllllqw.jsdoc
- ltmoerdani.opencode-copilot-chat
- lucas-rds.curl-to-axios
- LukeSpine.meta-spark-for-copilot
- mads-hartmann.bash-ide-vscode
- maketes.git-patch-utility
- matsken.git-stash-viewer
- Meezilla.json
- mnxn.jvm-bytecode-viewer
- ms-azuretools.vscode-containers
- ms-azuretools.vscode-docker
- ms-python.debugpy
- ms-python.isort
- ms-python.python
- ms-python.vscode-pylance
- ms-python.vscode-python-envs
- ms-vscode-remote.remote-containers
- ms-vscode-remote.remote-ssh
- ms-vscode-remote.remote-ssh
- ms-vscode-remote.remote-ssh-edit
- ms-vscode-remote.remote-wsl
- ms-vscode.makefile-tools
- ms-vscode.remote-explorer
- ms-vsliveshare.vsliveshare
- msjsdiag.vscode-react-native
- NeoMetric.google-acp-for-copilot
- openai.chatgpt
- openai.codex-audio
- PKief.material-icon-theme
- rangav.vscode-thunder-client
- redhat.fabric8-analytics
- redhat.java
- redhat.vscode-yaml
- ritwickdey.LiveServer
- rust-lang.rust-analyzer
- rvest.vs-code-prettier-eslint
- searKing.preview-vscode
- shayanline.devin-vscode
- sourcegraph.amp
- styled-components.vscode-styled-components
- sumneko.lua
- Supermaven.supermaven
- thebearingedge.vscode-sql-lit
- tomoki1207.pdf
- usernamehw.errorlens
- vscjava.vscode-gradle
- vscjava.vscode-java-dependency
- vscjava.vscode-java-pack
- vscode-icons-team.vscode-icons
- Vue.volar
- wmorellato.babel
- YoniChechik.git-diff-viewer

See [the compatibility plan](extension-compatibility.md).
