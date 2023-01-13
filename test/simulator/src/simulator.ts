type SimulationTaskResult = ResultPromise<unknown, SimulationFailed>;
type TaskExecutor = (session: Session) => SimulationTaskResult;

export class Simulator {
  static LOGGER = Logging.for(Simulator.name);
  static taskMap = new Map<TaskName, TaskExecutor>([
    [TaskName.Create, Simulator.prototype.create],
    [TaskName.Update, Simulator.prototype.update],
    [TaskName.UpdateWithError, Simulator.prototype.updateWithXml],
    [TaskName.Refresh, Simulator.prototype.refresh],
    [TaskName.Remove, Simulator.prototype.remove],
  ]);

  static clientErrorToSimulationResult(
    err: ClientError,
  ): SimulationFailed {
    if (err instanceof NoConnection) {
      return new SimulationAborted("server unavailable");
    }
    return new SimulationUnknownError(err);
  }

  static create(
    def: { name: string; includeErrors: boolean; randomWait: number },
  ) {
    return new Simulator(def.name, def.includeErrors, def.randomWait);
  }

  private client = new TokenClient();
  private created = Date.now();
  private scheduler;

  constructor(
    public name: string,
    include_errors: boolean,
    random_wait: number,
  ) {
    let index = 0;
    this.scheduler = new Scheduler(random_wait > 0);
    for (
      const assignment of metadata_collection.filter((candidate) =>
        include_errors || isMeta(candidate)
      )
    ) {
      const delays = generateRandomDelays(random_wait);
      let when = 0;
      const session = new Session(assignment as Meta); // deliberately surpassing check for correct input data
      for (
        const task of [TaskName.Create, TaskName.Update].concat(
          include_errors && index == 1 ? TaskName.Remove : [],
        ).concat(
          include_errors && index == 3 ? TaskName.UpdateWithError : [],
        )
          .concat(TaskName.Refresh)
      ) {
        when += delays.next().value!;
        this.scheduler.schedule(
          when,
          task,
          session,
        );
      }
      index++;
    }
  }

  run(): ResultPromise<boolean, SimulationFailed> {
    return Ok((async () => {
      const iter = this.scheduler.iter();

      let result: Result<boolean, SimulationFailed>;
      do {
        result = await iter
          .next()
          .mapResult(
            () => OkPromise<boolean, SimulationFailed>(false),
            (todo) => {
              const info = `${todo.session}  ${todo.task}`;
              return Some(Simulator.taskMap.get(todo.task))
                .okOrElse<SimulationFailed>(() =>
                  new SimulationTaskUnknown(todo.task)
                )
                .map((executor) => executor.call(this, todo.session))
                .mapResult(
                  (err) => {
                    if (err instanceof SimulationAborted) {
                      return Err<boolean, SimulationFailed>(err);
                    } else {
                      this.logFailure(info, err.toString());
                      return Ok(true);
                    }
                  },
                  () => {
                    this.logSuccess(info);
                    return Ok(true);
                  },
                );
            },
          );
      } while (result.unwrapOr(false));

      return result;
    })());
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
      .andThen(
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
      .andThen(
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
      .andThen(
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

function* generateRandomDelays(maxSeconds: number) {
  yield 0;
  while (true) {
    yield maxSeconds == 0 ? 1 : Math.round(Math.random() * maxSeconds * 1000);
  }
}

import {
  ClientError,
  Err,
  isMeta,
  Logging,
  maxWidth,
  Meta,
  metadata_collection,
  MissingToken,
  NoConnection,
  Ok,
  OkPromise,
  Result,
  ResultPromise,
  Scheduler,
  Session,
  SimulationAborted,
  SimulationFailed,
  SimulationTaskUnknown,
  SimulationUnknownError,
  Some,
  TaskName,
  TokenClient,
} from "./deps.ts";
