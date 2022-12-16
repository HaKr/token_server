import { Meta } from "./api.ts";
import { None, Option, Some } from "./std/option.ts";

export class RequestProgress {
  created: number = Date.now();
  events: string[] = [];
  public optional_token = None as Option<string>;

  constructor(private instance: string, protected meta_: Meta) {}

  get instanceName() {
    return this.instance;
  }
  set token(val: string) {
    this.optional_token = Some(val);
  }

  set meta(v: Meta) {
    this.meta_ = v;
  }

  get meta() {
    return this.meta_;
  }

  clear_token() {
    this.optional_token = None as Option<string>;
  }

  log_success(what: string) {
    console.log(`${this.formatLabel()} ${what} succeeded`);
  }

  log_failure(what: string, why: string) {
    console.log(
      `${this.formatLabel()} ${what} failed${why.length > 0 ? `: ${why}` : ""}`,
    );
  }

  private info() {
    return this.meta_ === null
      ? "NIL"
      : `${
        this.meta_.lastName
          ? this.meta_.lastName
          : this.meta_.year
          ? this.meta_.year
          : ""
      }` +
        `${
          (this.meta_.lastName || this.meta_.year) && this.meta_.updatedAt
            ? ", "
            : ""
        }` +
        `${this.meta_.updatedAt ? `updatedAt: ${this.meta_.updatedAt}` : ""}`;
  }

  private formatLifetime(lifetime: number, pad: number): string {
    return `${lifetime}`.padStart(pad, "0");
  }

  private formatLabel(): string {
    return `[${this.formatLifetime(Date.now() - this.created, 5)}ms ` +
      `${
        this.optional_token.map_or(" none", (_) => "token").unwrap_or(
          "impossible",
        )
      } ` +
      `${this.instance}: {${this.info().padEnd(38)}}]`;
  }
}
