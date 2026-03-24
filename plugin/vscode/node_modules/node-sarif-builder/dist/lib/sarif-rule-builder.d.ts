import { ReportingDescriptor } from 'sarif';
import { SarifRuleOptions } from '../types/node-sarif-builder';
export declare class SarifRuleBuilder {
    rule: ReportingDescriptor;
    constructor(options?: SarifRuleOptions);
    initSimple(options: {
        ruleId: string;
        shortDescriptionText: string;
        fullDescriptionText?: string;
        helpUri?: string;
    }): this;
    setRuleId(ruleId: string): void;
    setShortDescriptionText(shortDescriptionText: string): void;
    setFullDescriptionText(fullDescriptionText: string): void;
    setHelpUri(url: string): void;
}
