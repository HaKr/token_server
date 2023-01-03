import { Meta, TokenUpdateResponseBody, TokenUpdateResult } from "./api.ts";
import { Err, Ok, Result, resultFrom, ResultPromise } from "./deps.ts";
import { Logging } from "./logging.ts";
import { Failure } from "./error.ts";

export type ClientResult<T> = Result<T, ClientError>;
export type FutureClientResult<T> = ResultPromise<T, ClientError>;

const CONTENT_JSON = {
  "Content-Type": "application/json",
};

const CONTENT_XML = {
  "Content-Type": "text/xml",
};

export class TokenClient {
  static LOGGER = Logging.for(TokenClient.name);
  static SERVER = "http://127.0.0.1:3666";
  static ENDPOINT_TOKEN = `${TokenClient.SERVER}/token`;
  static ENDPOINT_DUMP = `${TokenClient.SERVER}/dump`;
  static ENDPOINT_PING = `${TokenClient.SERVER}/ping`;
  static ENDPOINT_SHUTDOWN = `${TokenClient.SERVER}/shutdown`;

  public ping(): FutureClientResult<true> {
    return resultFrom(
      this.fetch(
        TokenClient.ENDPOINT_PING,
        {
          method: "GET",
        },
      ).mapOrElse(
        (err) =>
          err instanceof UrlNotFound
            ? Ok<true, ClientError>(true)
            : Err<true, ClientError>(err),
        (response) =>
          Err<true, ClientError>(new UnexpectedPingResult(response)),
      ),
    );
  }

  public createToken(meta: Meta): FutureClientResult<string> {
    return this.fetchText(
      TokenClient.ENDPOINT_TOKEN,
      {
        method: "POST",
        headers: CONTENT_JSON,
        body: JSON.stringify({ meta }),
      },
    );
  }

  public shutdown(): FutureClientResult<string> {
    return this.fetchText(
      TokenClient.ENDPOINT_SHUTDOWN,
      {
        method: "GET",
      },
    );
  }

  public updateToken(
    token: string,
    meta?: Meta,
    forceMediaError = false,
  ): FutureClientResult<TokenUpdateResult> {
    return resultFrom(
      this.fetchJson<TokenUpdateResponseBody>(
        TokenClient.ENDPOINT_TOKEN,
        {
          method: "PUT",
          headers: forceMediaError ? CONTENT_XML : CONTENT_JSON,
          body: JSON.stringify({ token, meta }),
        },
      ).mapOrElse<ClientResult<TokenUpdateResult>>(Err, (response) => {
        if (response.Ok) {
          return Ok<TokenUpdateResult, ClientError>(response.Ok);
        } else {
          if (response.Err! == "InvalidToken") {
            return Err<TokenUpdateResult, ClientError>(new InvalidToken());
          } else {
            return Err<TokenUpdateResult, ClientError>(
              new UnexpectedUpdateResponse(response.Err),
            );
          }
        }
      }),
    );
  }

  public deleteToken(
    token: string,
  ): FutureClientResult<string> {
    return this.fetchText(
      TokenClient.ENDPOINT_TOKEN,
      {
        method: "DELETE",
        headers: CONTENT_JSON,
        body: JSON.stringify({ token }),
      },
    );
  }

  private fetchJson<T extends Meta = Meta>(
    url: string,
    options: RequestInit,
  ): FutureClientResult<T> {
    const event = TokenClient.LOGGER.trace(
      `${options.method} ${url} ${options.body}`,
    );
    const result: FutureClientResult<T> = this.fetch(url, options)
      .andThen(async (response) => {
        if (response.headers.get("content-type") == "application/json") {
          return Ok<T, ClientError>(await response.json());
        } else {return Err<T, ClientError>(
            new UnsupportedContentTypeResponse(
              response.headers.get("content-type"),
            ),
          );}
      });

    result.then((result) => TokenClient.LOGGER.trace(...event, ` ->`, result));

    return result;
  }
  private fetchText(
    url: string,
    options: RequestInit,
  ): FutureClientResult<string> {
    const event = TokenClient.LOGGER.trace(
      `${options.method} ${url} ${options.body}`,
    );

    const result = this.fetch(url, options)
      .andThen(async (response) =>
        Ok<string, ClientError>(await response.text())
      );

    TokenClient.LOGGER.trace(...event, ` ->`, result);
    return result;
  }

  protected fetch(
    url: string,
    options: RequestInit,
  ): FutureClientResult<Response> {
    return resultFrom(
      fetch(url, options)
        .then(async (response): Promise<ClientResult<Response>> => {
          if (response.ok) {
            return Ok(response);
          } else {
            switch (response.status) {
              case 400:
                return Err(new BadRequest(await response.text()));
              case 404:
                return Err(new UrlNotFound(url));
              case 405:
                return Err(new MethodNotAllowed(options.method!));
              case 415:
                return Err(new UnsupportedMediaType(await response.text()));
              case 422:
                return Err(new UnprocessableEntity(await response.text()));
              default:
                return Err(
                  new UnrecognizedResponse(
                    response.status,
                    response.statusText,
                    await response.text(),
                  ),
                );
            }
          }
        })
        .catch((err): ClientResult<Response> => {
          return err instanceof TypeError
            ? Err(new NoConnection())
            : Err(new UnrecognizedFailure(err));
        }),
    );
  }
}

export abstract class ClientError extends Failure {}

export class UrlNotFound extends ClientError {}
export class UnsupportedContentTypeResponse extends ClientError {}
export class NoConnection extends ClientError {}
export class MethodNotAllowed extends ClientError {}
export class UnrecognizedResponse extends ClientError {
  constructor(
    private statusCode: number,
    private statusText: string,
    private info: string,
  ) {
    super();
  }

  toString(): string {
    return super.toString() +
      `${this.statusCode} ${this.statusText}  ${this.info}`;
  }
}
export class UnrecognizedFailure extends ClientError {
  // deno-lint-ignore no-explicit-any
  constructor(err: any) {
    super(err.message ? err.message : JSON.stringify(err));
  }
}
export class UnexpectedPingResult extends ClientError {}
export class BadRequest extends ClientError {}
export class UnsupportedMediaType extends ClientError {}
export class UnprocessableEntity extends ClientError {}
export class InvalidToken extends ClientError {}
export class UnexpectedUpdateResponse extends ClientError {}
