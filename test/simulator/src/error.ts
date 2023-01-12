import { Result } from "./deps.ts";

export abstract class Failure {
  constructor(public reason?: unknown) {}

  toString(): string {
    return this.reason instanceof Failure
      ? this.reason.toString()
      : `${this.constructor.name[0]}${
        this.constructor.name.slice(1).replace(/([A-Z])/g, (_sub, ...args) => {
          const [a] = args as string[];
          return ` ${a.toLowerCase()}`;
        })
      }${this.reason == undefined ? "" : `: ${this.reason}`}`;
  }
}

export abstract class ParseError extends Failure {}

export type SimulationResult = Result<void, SimulationFailed>;
export class SimulationFailed extends Failure {}

export class SimulationAborted extends SimulationFailed {}
export class MissingToken extends SimulationFailed {}
