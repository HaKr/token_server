import { Meta } from "./api.ts";
import { MissingToken, SimulationAborted } from "./error.ts";
import { SimulationFailed } from "./error.ts";
import { Logging } from "./logging.ts";
import { metadata_collection } from "./mock/metadata_collection.js";
import { Scheduler } from "./scheduler.ts";
import { Session } from "./session.ts";
import { Err, Ok, Result, ResultPromise } from "./deps.ts";
import { Task } from "./tasks.ts";
import { ClientError, NoConnection, TokenClient } from "./token_client.ts";

type SimulationResult = Result<boolean, SimulationFailed>;
type SimulationTaskResult = ResultPromise<unknown, SimulationFailed>;
export type TaskExecutor = (session: Session) => SimulationTaskResult;

export class Simulator {
  static LOGGER = Logging.for(Simulator.name);
  static taskMap: { [key: string]: TaskExecutor } = {
    [Task.Create]: Simulator.prototype.create,
    [Task.Update]: Simulator.prototype.update,
    [Task.UpdateWithError]: Simulator.prototype.updateWithXml,
    [Task.Refresh]: Simulator.prototype.refresh,
    [Task.Remove]: Simulator.prototype.remove,
  };

  static clientErrorToSimulationResult(
    err: ClientError,
  ): SimulationFailed {
    if (err instanceof NoConnection) {
      return new SimulationAborted("server unavailable");
    }
    return new SimulationFailed(err);
  }

  client = new TokenClient();
  created = Date.now();
  scheduler;

  constructor(
    public name: string,
    include_errors: boolean,
    random_wait: number,
  ) {
    let index = 0;
    this.scheduler = new Scheduler(random_wait > 0);
    for (
      const assignment of metadata_collection.filter((candidate) =>
        include_errors || candidate !== null
      )
    ) {
      assert(
        assignment == null || typeof assignment == "object",
        "Illegal input format",
      );
      const delays = generateDelays(random_wait);
      let when = 0;
      const session = new Session(assignment as Meta);
      for (
        const task of [Task.Create, Task.Update].concat(
          include_errors && index == 1 ? Task.Remove : [],
        ).concat(
          include_errors && index == 3 ? Task.UpdateWithError : [],
        )
          .concat(Task.Refresh)
      ) {
        when += delays.next().value!;
        this.scheduler.schedule(
          when,
          task,
          session, // deliberately surpassing check for correct input data
        );
      }
      index++;
    }
  }

  async run(): Promise<SimulationResult> {
    const iter = this.scheduler.iter();

    let result: Result<boolean, SimulationFailed>;
    do {
      result = (await iter
        .next()
        .mapOrElse(
          () => Ok<boolean, SimulationFailed>(false),
          (todo) => {
            const info = `${todo.session}  ${todo.task}`;
            return Simulator.taskMap[todo.task].call(this, todo.session)
              .mapOrElse(
                (err) => {
                  if (err instanceof SimulationAborted) {
                    return Err<boolean, SimulationFailed>(err);
                  } else {
                    this.logFailure(info, err.toString());
                  }
                  return Ok<boolean, SimulationFailed>(true);
                },
                () => {
                  this.logSuccess(info);
                  return Ok<boolean, SimulationFailed>(true);
                },
              );
          },
        )).unwrapOrElse(() => Ok(false));
    } while (result.unwrapOr(false));

    return result.map((looping) => !looping);
  }

  shutdownServer() {
    return this.client.shutdown();
  }

  protected create(session: Session): SimulationTaskResult {
    return this.client.createToken(session.meta)
      .map((token) => session.create(token))
      .mapErr(Simulator.clientErrorToSimulationResult);
  }

  protected update(session: Session): SimulationTaskResult {
    return this.updateTask(session, false);
  }

  protected updateWithXml(session: Session): SimulationTaskResult {
    return this.updateTask(session, true);
  }

  protected updateTask(
    session: Session,
    forceMediaError: boolean,
  ): SimulationTaskResult {
    return session
      .tokenOrElse(() => new MissingToken())
      .mapOrElse(
        Err,
        (token) =>
          this.client.updateToken(
            token,
            { updatedAt: Date.now() },
            forceMediaError,
          )
            .map((update_result) =>
              session.update(update_result.token, update_result.meta)
            ).mapErr(Simulator.clientErrorToSimulationResult),
      );
  }

  protected refresh(session: Session): SimulationTaskResult {
    return session
      .tokenOrElse(() => new MissingToken())
      .mapOrElse(
        Err,
        (token) =>
          this.client.updateToken(token)
            .map((update_result) =>
              session.update(update_result.token, update_result.meta)
            ).mapErr(Simulator.clientErrorToSimulationResult),
      );
  }

  protected remove(session: Session): SimulationTaskResult {
    return session
      .tokenOrElse(() => new MissingToken())
      .mapOrElse(
        Err,
        (token) =>
          this.client.deleteToken(token)
            .map(() => session.clearToken())
            .mapErr(Simulator.clientErrorToSimulationResult),
      );
  }

  private logSuccess(what: string) {
    Simulator.LOGGER.info(`${this.formatLabel()} ${what} succeeded`);
  }

  private logFailure(what: string, why: string) {
    Simulator.LOGGER.error(
      `${this.formatLabel()} ${what} failed${
        why.length > 0 ? `: ${maxWidth(why, 94)}` : ""
      }`,
    );
  }

  private formatLifetime(lifetime: number, pad: number): string {
    return `${lifetime}`.padStart(pad, "0");
  }

  private formatLabel(): string {
    return `[${
      this.formatLifetime(Date.now() - this.created, 5)
    }ms ${this.name}]`;
  }
}

function maxWidth(str: string, width: number) {
  if (str.length <= width) return str;
  const half = (width - 5) / 2;
  return `${str.slice(0, half)} ... ${str.slice(-half)}`;
}

function* generateDelays(maxSeconds: number) {
  yield 0;
  while (true) {
    yield maxSeconds == 0 ? 1 : Math.round(Math.random() * maxSeconds * 1000);
  }
}

function assert(predicate: boolean, msg?: string | undefined): void | never {
  if (!predicate) throw new Error(msg || "Assertion failed");
}
