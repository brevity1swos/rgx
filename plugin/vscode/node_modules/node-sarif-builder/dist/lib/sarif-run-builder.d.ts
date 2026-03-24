import { Run } from 'sarif';
import { SarifRuleBuilder } from '..';
import { SarifRunOptions } from '../types/node-sarif-builder';
import { SarifResultBuilder } from './sarif-result-builder';
export declare class SarifRunBuilder {
    run: Run;
    constructor(options?: SarifRunOptions);
    initSimple(options: {
        toolDriverName: string;
        toolDriverVersion: string;
        url?: string;
    }): this;
    addRule(sarifRuleBuilder: SarifRuleBuilder): void;
    addResult(sarifResultBuilder: SarifResultBuilder): void;
    setToolDriverName(name: string): void;
    setToolDriverVersion(version: string): void;
    setToolDriverUri(url: string): void;
}
