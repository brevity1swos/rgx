# Changelog

## Beta

## [3.4.0] 2025-12-30

- Upgrade NPM dependencies
- CI: Use NPM trusted publishers for better security
- Add .npmignore to decrease package size

## [3.3.1] 2025-11-23

- Upgrade NPM dependencies

## [3.3.0] 2025-11-16

- Upgrade NodeJs minimum version to Node 20
- Upgrade NPM dependencies
- Upgrade Github Actions workflows dependencies
- Upgrade MegaLinter to v9

## [3.2.0] 2024-08-22

- Upgrade npm dependencies
- Upgrade MegaLinter to v8

## [3.1.0] 2024-16-03

- Update schema to final 2.1.0 release (but the schema definition remains the same)
  - New value:  `$schema: 'http://json.schemastore.org/sarif-2.1.0.json'`
  - Previous value: `$schema: 'http://json.schemastore.org/sarif-2.1.0-rtm.5.json'`
- Add github-dependents-info workflow

## [3.0.0] 2023-11-26

- BREAKING CHANGE: New minimal version of Node.js: v18
- Upgrade Github Actions workflows
- Upgrade MegaLinter
- Upgrade npm dependencies

## [2.0.3] 2022-10-23

- Run yarn-audit-fix to upgrade dependencies with security issues (minimist, node-fetch)
- CI: Upgrade MegaLinter to v6

## [2.0.2] 2022-01-30

- Update License to MIT

## [2.0.1] 2022-01-12

- Update JSON schema to <http://json.schemastore.org/sarif-2.1.0-rtm.5.json>

## [2.0.0] 2022-01-12

- Mandatory properties `toolDriverName` and `toolDriverVersion` for SarifRunBuilder
- Change default schema version to <https://www.schemastore.org/schemas/json/sarif-2.1.0-rtm.5.json>
- When possible, automatically populate SARIF properties:
  - `artifact.sourceLanguage`
  - `result.locations.location.physicalLocation.artifactLocation.index`
  - `result.ruleIndex`
- Fix bug when initSimple is called without region properties

## [1.0.0] 2022-01-11

- Initial version
