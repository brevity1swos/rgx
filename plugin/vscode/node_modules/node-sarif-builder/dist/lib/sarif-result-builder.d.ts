import { ArtifactLocation, Region, Result } from 'sarif';
import { SarifResultOptions } from '../types/node-sarif-builder';
export declare class SarifResultBuilder {
    result: Result;
    constructor(options?: SarifResultOptions);
    initSimple(options: {
        level: Result.level;
        messageText: string;
        ruleId: string;
        fileUri?: string;
        startLine?: number;
        startColumn?: number;
        endLine?: number;
        endColumn?: number;
    }): this;
    setLevel(level: Result.level): void;
    setMessageText(message: string): void;
    setRuleId(ruleId: string): void;
    setLocationRegion(region: Region): void;
    setLocationArtifactUri(artifactLocation: ArtifactLocation): void;
    private manageInitLocation;
    private manageInitPhysicalLocation;
}
