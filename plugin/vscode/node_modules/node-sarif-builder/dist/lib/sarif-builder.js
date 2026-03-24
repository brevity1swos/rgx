"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SarifBuilder = void 0;
const path = require("path");
const fs = require("fs-extra");
const languages_1 = require("./languages");
const utils_1 = require("./utils");
// SARIF Builder
class SarifBuilder {
    // Initialize SARIF Log builder
    constructor(options = {}) {
        // Default run value
        this.log = {
            $schema: 'http://json.schemastore.org/sarif-2.1.0.json',
            version: '2.1.0',
            runs: [],
        };
        (0, utils_1.setOptionValues)(options, this.log);
    }
    addRun(sarifRunBuilder) {
        this.log.runs.push(sarifRunBuilder.run);
    }
    generateSarifFileSync(file) {
        const sarifJsonString = this.buildSarifJsonString();
        fs.writeFileSync(file, sarifJsonString, 'utf8');
    }
    async generateSarifFile(file) {
        const sarifJsonString = this.buildSarifJsonString();
        await fs.writeFile(file, sarifJsonString, 'utf8');
    }
    buildSarifOutput() {
        // Complete runs
        this.log.runs = this.log.runs.map((run) => this.completeRunFields(run));
        return this.log;
    }
    // Build final sarif json, complete when possible
    buildSarifJsonString(options = { indent: false }) {
        this.buildSarifOutput();
        const sarifJson = options.indent
            ? JSON.stringify(this.log, null, 2)
            : JSON.stringify(this.log);
        if (sarifJson.includes('SARIF_BUILDER_INVALID')) {
            throw new Error('Your SARIF log is invalid, please solve SARIF_BUILDER_INVALID messages');
        }
        return sarifJson;
    }
    completeRunFields(run) {
        var _a, _b, _c, _d;
        // Collect all missing artifacts from results
        run.artifacts = run.artifacts || [];
        for (const result of run.results) {
            for (const location of result.locations || []) {
                if (((_b = (_a = location === null || location === void 0 ? void 0 : location.physicalLocation) === null || _a === void 0 ? void 0 : _a.artifactLocation) === null || _b === void 0 ? void 0 : _b.uri) &&
                    run.artifacts.filter((artifact) => {
                        var _a;
                        return ((_a = artifact === null || artifact === void 0 ? void 0 : artifact.location) === null || _a === void 0 ? void 0 : _a.uri) ===
                            location.physicalLocation.artifactLocation.uri;
                    }).length === 0) {
                    // Add result to driver artifact only if not existing
                    const ext = path
                        .extname(location.physicalLocation.artifactLocation.uri)
                        .replace('.', '');
                    const language = languages_1.EXTENSIONS_LANGUAGES[ext] || 'unknown';
                    run.artifacts.push({
                        sourceLanguage: language,
                        location: { uri: location.physicalLocation.artifactLocation.uri },
                    });
                }
            }
        }
        // Build artifacts indexes
        const artifactIndexes = run.artifacts.map((artifact) => {
            var _a;
            return (_a = artifact === null || artifact === void 0 ? void 0 : artifact.location) === null || _a === void 0 ? void 0 : _a.uri;
        });
        // Build rules indexes
        const rulesIndexes = (((_d = (_c = run === null || run === void 0 ? void 0 : run.tool) === null || _c === void 0 ? void 0 : _c.driver) === null || _d === void 0 ? void 0 : _d.rules) || []).map((rule) => {
            return rule.id;
        });
        // Update index in results with computed values
        run.results = run.results.map((result) => {
            // Set rule index in results
            if (rulesIndexes.indexOf(result.ruleId) > -1) {
                result.ruleIndex = rulesIndexes.indexOf(result.ruleId);
            }
            // Set artifact index in results
            if (result.locations) {
                result.locations = result.locations.map((location) => {
                    var _a, _b;
                    const uri = (_b = (_a = location === null || location === void 0 ? void 0 : location.physicalLocation) === null || _a === void 0 ? void 0 : _a.artifactLocation) === null || _b === void 0 ? void 0 : _b.uri;
                    if (uri && artifactIndexes.indexOf(uri) > -1) {
                        location.physicalLocation.artifactLocation.index =
                            artifactIndexes.indexOf(uri);
                    }
                    return location;
                });
            }
            return result;
        });
        return run;
    }
}
exports.SarifBuilder = SarifBuilder;
