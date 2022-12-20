import { Meta, TokenUpdateResponseBody, TokenUpdateResult } from "./api.ts";
import { Err, Ok, Result } from "./std/result.ts";
import { Logging } from "./logging.ts";
import { Failure } from "./error.ts";

export type ClientResult<T> = Result<T, ClientError>;
export type FutureClientResult<T> = Promise<ClientResult<T>>;

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

  public async ping(): FutureClientResult<true> {
    return (await this.fetch(
      TokenClient.ENDPOINT_PING,
      {
        method: "GET",
      },
    )).map_or_else(
      (err) => err instanceof UrlNotFound ? Ok(true) : Err(err),
      (response) => Err(new UnexpectedPingResult(response)),
    );
  }

  public async create_token(meta: Meta): FutureClientResult<string> {
    return (await this.fetch_text(
      TokenClient.ENDPOINT_TOKEN,
      {
        method: "POST",
        headers: CONTENT_JSON,
        body: JSON.stringify({ meta }),
      },
    ));
  }

  public async shutdown(): FutureClientResult<string> {
    return (await this.fetch_text(
      TokenClient.ENDPOINT_SHUTDOWN,
      {
        method: "GET",
      },
    ));
  }

  public async update_token(
    token: string,
    meta?: Meta,
    forceMediaError = false,
  ): FutureClientResult<TokenUpdateResult> {
    return (await this.fetch_json<TokenUpdateResponseBody>(
      TokenClient.ENDPOINT_TOKEN,
      {
        method: "PUT",
        headers: forceMediaError ? CONTENT_XML : CONTENT_JSON,
        body: JSON.stringify({ token, meta }),
      },
    ))
      .map_or_else<ClientResult<TokenUpdateResult>>(Err, (response) => {
        if (response.Ok) {
          return Ok(response.Ok);
        } else {
          if (response.Err! == "InvalidToken") {
            return Err(new InvalidToken());
          } else {
            return Err(
              new UnexpectedUpdateResponse(response.Err),
            );
          }
        }
      });
  }

  public async delete_token(
    token: string,
  ): FutureClientResult<string> {
    return (await this.fetch_text(
      TokenClient.ENDPOINT_TOKEN,
      {
        method: "DELETE",
        headers: CONTENT_JSON,
        body: JSON.stringify({ token }),
      },
    ));
  }

  private async fetch_json<T extends Meta = Meta>(
    url: string,
    options: RequestInit,
  ): FutureClientResult<T> {
    const event = TokenClient.LOGGER.trace(
      `${options.method} ${url} ${options.body}`,
    );
    const result: FutureClientResult<T> = (await this.fetch(url, options))
      .async_and_then(async (response) => {
        if (response.headers.get("content-type") == "application/json") {
          return Ok<T, ClientError>(await response.json());
        } else {return Err(
            new UnsupportedContentTypeResponse(
              response.headers.get("content-type"),
            ),
          );}
      });

    TokenClient.LOGGER.trace(...event, ` ->`, await result);

    return result;
  }
  private async fetch_text(
    url: string,
    options: RequestInit,
  ): FutureClientResult<string> {
    const event = TokenClient.LOGGER.trace(
      `${options.method} ${url} ${options.body}`,
    );

    const result = await (await this.fetch(url, options)).async_and_then(async (
      response,
    ) => Ok<string, ClientError>(await response.text()));

    TokenClient.LOGGER.trace(...event, ` ->`, result);
    return result;
  }

  protected fetch(
    url: string,
    options: RequestInit,
  ): FutureClientResult<Response> {
    return fetch(url, options)
      .then(async (response): FutureClientResult<Response> => {
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
      });
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