import { Meta, TokenUpdateResponseBody, TokenUpdateResult } from "./api.ts";
import { Err, Ok, Result } from "./std/result.ts";

const SERVER = "http://127.0.0.1:3666";
const ENDPOINT_TOKEN = `${SERVER}/token`;
const ENDPOINT_DUMP = `${SERVER}/dump`;
// const ENDPOINT_NONEXISTING = `${SERVER}/doesnotexist`;

const headers = {
  "Content-Type": "application/json",
};

export async function create_token(
  instance: string,
  meta: Meta | undefined,
): Promise<Result<string, string>> {
  if (meta) meta.instance = instance;

  return (await fetch_result(`${ENDPOINT_TOKEN}?instance=${instance}`, {
    method: "POST",
    headers,
    body: JSON.stringify({ meta }),
  }))
    .async_and_then(async (response) => {
      if (response.ok) {
        return Ok<string, string>(await response.text());
      } else {
        return errorFromResponse(response);
      }
    });
}

export async function request_dump(
  instance: string,
): Promise<Result<true, string>> {
  return (await fetch_result(`${ENDPOINT_DUMP}?d=${instance}`, {
    method: "HEAD",
  }))
    .async_and_then(async (response) => {
      if (response.ok) {
        return Ok<true, string>(true);
      } else {
        return await errorFromResponse(response);
      }
    });
}

export async function remove_token(
  instance: string,
  token: string,
): Promise<Result<true, string>> {
  return (await fetch_result(`${ENDPOINT_TOKEN}?instance=${instance}`, {
    method: "DELETE",
    headers,
    body: JSON.stringify({ token }),
  }))
    .async_and_then(async (response) => await resultFrom(response, true));
}

export async function shutdown_server(
  instance: string,
): Promise<Result<true, string>> {
  return (await fetch_result(`${SERVER}/shutdown?instance=${instance}`, {
    method: "GET",
  }))
    .async_and_then((response) => resultFrom(response, true));
}

export class TokenUpdateFailed {
  constructor(public reason: string) {}
}
export class InvalidToken extends TokenUpdateFailed {
  constructor() {
    super("InvalidToken");
  }
}
export class FetchFailed extends TokenUpdateFailed {
}

export async function update_token(
  instance: string,
  token: string,
  meta?: Meta,
): Promise<Result<TokenUpdateResult, TokenUpdateFailed>> {
  return await (await fetch_result(`${ENDPOINT_TOKEN}?instance=${instance}`, {
    method: "PUT",
    headers,
    body: JSON.stringify({ token, meta }),
  })).map_err((err) => new FetchFailed(err) as unknown as TokenUpdateFailed)
    .async_and_then(async (response) => {
      if (
        response.ok &&
        response.headers.get("content-type") == "application/json"
      ) {
        const info = await response.json() as TokenUpdateResponseBody;

        if (info.Ok) {
          return Ok(info.Ok) as Result<TokenUpdateResult, TokenUpdateFailed>;
        } else {
          return Err(new InvalidToken()) as Result<
            TokenUpdateResult,
            TokenUpdateFailed
          >;
        }
      } else {
        return Err(new FetchFailed(await formatResponse(response))) as Result<
          TokenUpdateResult,
          TokenUpdateFailed
        >;
      }
    });
}

async function resultFrom<T>(
  response: Response,
  ok: T,
): Promise<Result<T, string>> {
  if (response.ok) {
    return Ok(ok);
  } else {
    return await errorFromResponse(response);
  }
}

async function errorFromResponse<T>(
  response: Response,
): Promise<Result<T, string>> {
  return Err(await formatResponse(response));
}

async function formatResponse(response: Response): Promise<string> {
  const result = `[${response.status} ${response.statusText}] ${await response
    .text()}`;

  return result.length > 96
    ? `${result.slice(0, 44)} ... ${result.slice(result.length - 46)}`
    : result;
}

/*
export async function create_invalid(instance, meta) {
    const response = await fetch_result(`${ENDPOINT_TOKEN}?instance=${instance}`, {
        method: "POST",
        headers: {
            "Content-Type": "text/plain"
        },
        body: JSON.stringify({ meta })
    });

    if (response.ok) {
        let token = await response.text();
        if (token.startsWith("ERROR")) {
            return Err(token.substring(7));
        } else {
            return Ok({ created: Date.now(), token });
        }
    } else {
        return Err(`[${response.status} ${response.statusText}] ${await response.text()}`)
    }
}

export async function nonexisting(instance) {
    return (await fetch_result(`${ENDPOINT_NONEXISTING}?d=${instance}`, {
        method: "GET"
    }))
        .and_then(response => {
            if (!response.ok) {
                return Err(`${response.status} ${response.statusText}`);
            }
        })

}
*/
function fetch_result(
  url: string,
  options: RequestInit,
): Promise<Result<Response, string>> {
  return fetch(url, options)
    .then(Ok<Response, string>)
    .catch(Err<Response, string>);
}
