"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SarifRunBuilder = void 0;
const utils_1 = require("./utils");
// SARIF Run builder
class SarifRunBuilder {
    // Initialize SARIF Run builder
    constructor(options = {}) {
        // Default run value
        this.run = {
            tool: {
                driver: {
                    name: process.env.npm_package_name ||
                        'SARIF_BUILDER_INVALID: Please send the tool name in tool.driver.name property, or call setToolName(name)',
                    rules: [],
                },
            },
            results: [],
        };
        (0, utils_1.setOptionValues)(options, this.run);
    }
    initSimple(options) {
        this.setToolDriverName(options.toolDriverName);
        this.setToolDriverVersion(options.toolDriverVersion);
        if (options.url) {
            this.setToolDriverUri(options.url);
        }
        return this;
    }
    addRule(sarifRuleBuilder) {
        this.run.tool.driver.rules.push(sarifRuleBuilder.rule);
    }
    addResult(sarifResultBuilder) {
        this.run.results.push(sarifResultBuilder.result);
    }
    setToolDriverName(name) {
        this.run.tool.driver.name = name;
    }
    setToolDriverVersion(version) {
        this.run.tool.driver.version = version;
    }
    setToolDriverUri(url) {
        this.run.tool.driver.informationUri = url;
    }
}
exports.SarifRunBuilder = SarifRunBuilder;
