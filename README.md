# CaBr2

[![License: GPL3+](https://img.shields.io/badge/License-GPL3+-blue.svg?style=flat-square)](https://www.gnu.org/licenses/gpl-3.0.en.html)
[![gh-actions](https://img.shields.io/github/workflow/status/Calciumdibromid/CaBr2/frontend?style=flat-square)](https://github.com/Calciumdibromid/CaBr2/actions)
[![Crowdin](https://badges.crowdin.net/cabr2/localized.svg)](https://crowdin.com/project/cabr2)

Generate "experiment wise safety sheets" in compliance to European law.

## Description

Calciumdibromid (short: CaBr2) is a tool to generate safety data sheets for experiments.

It is written in [Angular](https://angular.io/) and can be either used as a standalone
desktop application or as a static webpage with WASM bindings and an API server to
generate PDFs.

## Structure

This project can be built in two ways:

- web front end with webserver and WASM bindings
- [Tauri](https://tauri.studio/) app

From this the folder structure was derived:

| path                  | description                                                                      |
|-----------------------|----------------------------------------------------------------------------------|
| `/`                   | Git repo root with obvious files                                                 |
| `webserver/`          | CaBr2 as REST API implementation                                                 |
| `crates/`             | core CaBr2 implementation that is shared                                         |
| `frontend/`           | Angular application that can be built for Tauri or as standalone web application |
| `frontend/src/`       | Angular source code                                                              |
| `frontend/src-tauri/` | Tauri glue code for CaBr2 logic                                                  |
| `frontend/src-wasm/`  | WASM glue code for CaBr2 logic                                                   |

To learn more about a specific part of this project, go to the corresponding README:

[Angular Application](frontend)  
[WASM library](frontend/src-wasm)  
[Webserver](webserver)  

## Translate

Translation is done via [Crowdin](https://crowdin.com/project/cabr2).

To improve the translation of a language or add a new one visit [https://crowdin.com/project/cabr2](https://crowdin.com/project/cabr2).
