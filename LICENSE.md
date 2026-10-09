# Billflux licensing status — draft

**This document is not the final Billflux license.** The copyright holder has not yet selected a license for the Billflux-specific source code, templates, artwork, or compiled application. Publication of this repository alone does not grant permission to copy, modify, redistribute, or sell those parts. Review and replace or amend this document before a public software release.

Third-party components retain their own licenses. This inventory identifies components currently used by the project; it does not relicense them or establish that the portable distribution meets every license condition.

| Component | Use in Billflux | License information |
| --- | --- | --- |
| Rust crates, including Tauri and rusqlite | Listed in Cargo.toml and Cargo.lock; compiled into the application | Individual crate licenses apply. Tauri is MIT OR Apache-2.0. A complete version-specific notice inventory is still needed for a release. |
| Fira Sans and Fira Sans Condensed | Bundled fonts | SIL Open Font License 1.1. Full texts in the portable build: bin/OFL-FiraSans.txt and bin/OFL-FiraSansCondensed.txt. Sources: assets/fonts/Fira_Sans/OFL.txt and assets/fonts/Fira_Sans_Condensed/OFL.txt. |
| Mustang CLI 2.26.0 | ZUGFeRD processing and validation in bin/ | Apache License 2.0. See upstream [LICENSE](https://github.com/ZUGFeRD/mustangproject/blob/master/LICENSE) and [NOTICE](https://github.com/ZUGFeRD/mustangproject/blob/master/NOTICE). Review the JAR's bundled dependency notices before release. |
| Ghostscript | PDF/A conversion in bin/gs/ | GNU Affero General Public License version 3. The complete AGPL text is included in bin/gs/AGPL-3.0.txt. See [Artifex licensing information](https://ghostscript.com/faq/). Compliance for distributing Billflux together with Ghostscript remains under review. |
| Eclipse Temurin 11 | Java runtime in bin/jre11/ | GPL version 2 with the Classpath Exception and additional notices. Keep the runtime's NOTICE and legal/ directory. See the [Adoptium FAQ](https://adoptium.net/docs/faq). |
| Chrome Headless Shell | HTML-to-PDF rendering in bin/chrome-headless-shell/ | Chromium/BSD-style and component licenses. Keep LICENSE.headless_shell and review its component notices. |
