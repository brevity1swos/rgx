"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SarifRuleBuilder = void 0;
const utils_1 = require("./utils");
/*
  Rules describing any error that the linter can return
*/
class SarifRuleBuilder {
    // Initialize SARIF Run builder
    constructor(options = {}) {
        this.rule = {
            id: 'SARIF_BUILDER_INVALID: Please send the rule identifier in id property, or call setRuleId(ruleId)',
            shortDescription: {
                text: 'SARIF_BUILDER_INVALID: Please send the rule text in shortDescription.text property, or call setShortDescriptionText(text)',
            },
        };
        (0, utils_1.setOptionValues)(options, this.rule);
    }
    initSimple(options) {
        this.setRuleId(options.ruleId);
        if (options.shortDescriptionText) {
            this.setShortDescriptionText(options.shortDescriptionText);
        }
        if (options.fullDescriptionText) {
            this.setFullDescriptionText(options.fullDescriptionText);
        }
        if (options.helpUri) {
            this.setHelpUri(options.helpUri);
        }
        return this;
    }
    setRuleId(ruleId) {
        this.rule.id = ruleId;
    }
    setShortDescriptionText(shortDescriptionText) {
        this.rule.shortDescription.text = shortDescriptionText;
    }
    setFullDescriptionText(fullDescriptionText) {
        this.rule.fullDescription = this.rule.fullDescription || { text: '' };
        this.rule.fullDescription.text = fullDescriptionText;
    }
    setHelpUri(url) {
        this.rule.helpUri = url;
    }
}
exports.SarifRuleBuilder = SarifRuleBuilder;
