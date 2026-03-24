"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SarifResultBuilder = void 0;
const utils_1 = require("./utils");
class SarifResultBuilder {
    // Initialize SARIF Result builder
    constructor(options = {}) {
        // Default result value
        this.result = {
            level: 'error',
            message: {},
            ruleId: 'SARIF_BUILDER_INVALID: Please send the rule Id ruleId property, or call setRuleId(ruleId)',
        };
        (0, utils_1.setOptionValues)(options, this.result);
    }
    initSimple(options) {
        this.setLevel(options.level);
        this.setMessageText(options.messageText);
        this.setRuleId(options.ruleId);
        if (options.fileUri) {
            this.setLocationArtifactUri({ uri: options.fileUri });
        }
        if (options.startLine !== null && options.startLine !== undefined) {
            // Initialize Region with default values with necessary
            const region = {
                startLine: options.startLine,
                startColumn: options.startColumn || 1,
                endLine: options.endLine || options.startLine,
                endColumn: options.endColumn || 1,
            };
            // Check for invalid region values
            if (options.startLine === 0 ||
                options.startColumn === 0 ||
                options.endLine === 0 ||
                options.endColumn === 0) {
                throw new Error('Region limit can not be 0 (minimum line 1 or column 1) in ' +
                    JSON.stringify(options));
            }
            this.setLocationRegion(region);
        }
        return this;
    }
    setLevel(level) {
        this.result.level = level;
    }
    setMessageText(message) {
        this.result.message.text = message;
    }
    setRuleId(ruleId) {
        this.result.ruleId = ruleId;
    }
    setLocationRegion(region) {
        this.manageInitPhysicalLocation();
        this.result.locations[0].physicalLocation.region = region;
    }
    setLocationArtifactUri(artifactLocation) {
        this.manageInitPhysicalLocation();
        this.result.locations[0].physicalLocation.artifactLocation =
            artifactLocation;
    }
    manageInitLocation() {
        var _a, _b;
        if ((_b = (_a = this.result) === null || _a === void 0 ? void 0 : _a.locations) === null || _b === void 0 ? void 0 : _b.length) {
            return;
        }
        this.result.locations = [{}];
    }
    manageInitPhysicalLocation() {
        var _a;
        this.manageInitLocation();
        if ((_a = this.result) === null || _a === void 0 ? void 0 : _a.locations[0].physicalLocation) {
            return;
        }
        this.result.locations[0].physicalLocation = {};
    }
}
exports.SarifResultBuilder = SarifResultBuilder;
