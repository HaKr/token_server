import { Meta } from "./api.ts";
import {
  ProgressCreateFailed,
  ProgressError,
  ProgressFetchFailed,
  ProgressRemoveFailed,
  ProgressRemoveWithoutToken,
  ProgressSkipThisOne,
  ProgressStopped,
  ProgressUpdateWithoutToken,
} from "./error.ts";
import { metadata_collection } from "./mock/metadata_collection.js";
import {
  create_token,
  InvalidToken,
  remove_token,
  request_dump,
  shutdown_server,
  update_token,
} from "./requests.ts";
import { RequestProgress } from "./request_progress.ts";
import { Option } from "./std/option.ts";
import { Err, Ok, Result } from "./std/result.ts";

type ProgressResult = Result<RequestProgress, ProgressError>;
type FutureResult = Promise<ProgressResult>;

function main() {
  run()
    .then((result) =>
      result.map_or_else(
        () => console.log("Finished", instance),
        (res) =>
          console.error(
            instance,
            " ABORTED:",
            instance == ERROR_INSTANCE
              // deno-lint-ignore no-explicit-any
              ? (res.err! as any).reason
              // deno-lint-ignore no-explicit-any
              : (res.err! as any).reason.reason.message,
          ),
      )
    );
}

async function run() {
  if (doSleep && instance == ERROR_INSTANCE) {
    setTimeout(() => {
      shutdown_server(instance);
    }, 4793);
  }

  const workload: ProgressResult[] = metadata_collection.map(
    (meta: Meta): ProgressResult => {
      return meta.invalid && instance != ERROR_INSTANCE
        ? Err(new ProgressSkipThisOne())
        : Ok(
          new RequestProgress(
            instance,
            meta.invalid ? null as unknown as Meta : meta,
          ),
        );
    },
  ).filter((req) => req.is_ok());

  let state = await Promise.all(
    workload.map(async (progress) => await create(progress)),
  );
  let abort = aborted(state);

  if (abort.is_none() && dump) reqdump(instance);

  if (abort.is_none()) {
    state = await Promise.all(state.map(async (progress) => {
      if (progress.is_ok()) await sleep(2);
      return await update(progress, { updatedAt: `${Date.now()}` });
    }));
    abort = aborted(state);
  }

  if (abort.is_none() && dump) reqdump(instance);

  if (abort.is_none()) {
    state = await Promise.all(
      state.map(async (progress, index) => {
        if (progress.is_ok()) await sleep(4);
        if (instance == ERROR_INSTANCE) {
          if (index == 1) await remove(progress);
        }
        return await update(progress);
      }),
    );
  }

  abort = aborted(state);

  return abort;
}

async function create(progress: ProgressResult): FutureResult {
  const step = "create token";

  return await progress.async_and_then(async (progress) => {
    return (await create_token(progress.instanceName, progress.meta))
      .map_or_else<ProgressResult>((err) => {
        progress.log_failure(step, err);

        return (err as unknown) instanceof TypeError
          ? Err(new ProgressStopped(new ProgressFetchFailed(err)))
          : Err(new ProgressCreateFailed());
      }, (token) => {
        progress.token = token;
        progress.log_success(step);

        return Ok(progress);
      });
  });
}

function reqdump(instance: string) {
  request_dump(instance);
}

async function update(progress: ProgressResult, meta?: Meta): FutureResult {
  const step = meta === undefined ? "refresh token" : "update metadata";

  return await progress.async_and_then(async (progress) => {
    return await progress.optional_token.ok_or_else(() =>
      new ProgressUpdateWithoutToken()
    ).async_map_or_else<ProgressResult>(
      (_err) => {
        return Ok(progress);
      },
      async (token) =>
        (await update_token(progress.instanceName, token, meta))
          .map_or_else((err) => {
            if (err instanceof InvalidToken) {
              progress.log_failure(step, "invalid token");
              progress.clear_token();

              return Ok(progress);
            } else {
              return Err(new ProgressStopped(err));
            }
          }, (updateResult) => {
            progress.meta = updateResult.meta;
            progress.token = updateResult.token;
            progress.log_success(step);

            return Ok(progress);
          }),
    );
  });
}

async function remove(progress: ProgressResult): FutureResult {
  const step = "remove token";

  return await progress.async_and_then(async (progress) => {
    const token_result = progress.optional_token.ok_or_else(() =>
      new ProgressRemoveWithoutToken()
    );

    return await token_result.async_map_or_else<ProgressResult>(
      (err) => {
        if (err instanceof ProgressRemoveWithoutToken) {
          progress.log_failure(step, "no token");
          return Ok(progress);
        } else {
          return Err(err);
        }
      },
      async (token) =>
        (await remove_token(progress.instanceName, token))
          .map_or_else<ProgressResult>((err) => {
            progress.log_failure(step, err);

            return Err(new ProgressStopped(new ProgressRemoveFailed()));
          }, (_) => {
            progress.clear_token();
            progress.log_success(step);

            return Ok(progress);
          }),
    );
  });
}

function aborted(state: ProgressResult[]): Option<ProgressResult> {
  return Option.from(
    state.find((res) =>
      res.map_or_else((err) => err instanceof ProgressStopped, (_ok) => false)
    ),
  );
}

const instanceName = Deno.args[0];
const randomSleep = Deno.args[1];

const DEFAULT_INSTANCE = "sim";
const ERROR_INSTANCE = "D";

const { instance, dump } =
  typeof instanceName == "string" && instanceName.length > 0 &&
    instanceName != DEFAULT_INSTANCE
    ? { instance: instanceName, dump: false }
    : { instance: DEFAULT_INSTANCE, dump: true };

const { sleep, doSleep } =
  typeof randomSleep == "string" && randomSleep.toLowerCase() == "yes"
    ? {
      sleep: (secs: number) =>
        new Promise((resolve) =>
          setTimeout(resolve, Math.random() * secs * 1_000)
        ),
      doSleep: true,
    }
    : { sleep: (_: number) => Promise.resolve(), doSleep: false };

if (instance == DEFAULT_INSTANCE) {
  console.debug(`Instance: ${instance}; dump: ${dump}; sleep? ${doSleep}`);
}

main();
