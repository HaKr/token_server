function assert(predicate: boolean, msg?: string | undefined): void | never {
  if (!predicate) throw new Error(msg || "Assertion failed");
}

export class Result<T, E> {
  ok: T | undefined = undefined;
  err: E | undefined = undefined;

  private constructor(ok: T | undefined, err: E | undefined) {
    if (ok !== undefined) this.ok = ok;
    else this.err = err;
  }

  static new_ok<U, F>(ok: U): Result<U, F> {
    assert(
      !(ok instanceof Promise),
      "You'd better await promises before returning them as Result",
    );
    return new Result(ok, undefined as unknown as F);
  }

  static new_err<U, F>(err: F): Result<U, F> {
    return new Result(undefined as unknown as U, err);
  }

  is_ok(): boolean {
    return !this.is_err();
  }

  is_err(): boolean {
    return this.err !== undefined;
  }

  unwrap(): T {
    return this.err !== undefined ? this.expect("Result is not Ok") : this.ok!;
  }

  expect(msg: string): T {
    assert(this.err === undefined, msg);

    return this.ok!;
  }

  unwrap_or_else(def: (err: E) => T) {
    return this.err !== undefined ? def(this.err!) : this.ok;
  }

  unwrap_or(def: T) {
    return this.err !== undefined ? def : this.ok;
  }

  and_then<U>(next: (ok: T) => Result<U, E>): Result<U, E> {
    return this.err !== undefined
      ? this as unknown as Result<U, E>
      : next(this.ok!);
  }

  async async_and_then<U>(
    next: (ok: T) => Promise<Result<U, E>>,
  ): Promise<Result<U, E>> {
    return this.err !== undefined
      ? this as unknown as Result<U, E>
      : await next(this.ok!);
  }

  or_else<F>(alt: (err: E) => Result<T, F>) {
    return this.err !== undefined ? alt(this.err) : this;
  }

  map<U>(mapper: (ok: T) => U): Result<U, E> {
    return this.err !== undefined
      ? this as unknown as Result<U, E>
      : Result.new_ok(mapper(this.ok!));
  }

  async async_map<U>(mapper: (ok: T) => Promise<U>): Promise<Result<U, E>> {
    return this.err !== undefined
      ? this as unknown as Result<U, E>
      : Result.new_ok(await mapper(this.ok!));
  }

  map_err<F>(mapper: (err: E) => F): Result<T, F> {
    return this.err !== undefined
      ? Result.new_err(mapper(this.err))
      : this as unknown as Result<T, F>;
  }

  map_or<U>(def: U, mapper: (ok: T) => U): U {
    return this.err !== undefined ? def : mapper(this.ok!);
  }

  async async_map_or<U>(def: U, mapper: (ok: T) => Promise<U>): Promise<U> {
    return this.err !== undefined ? def : await mapper(this.ok!);
  }

  map_or_else<U>(def: (err: E) => U, mapper: (ok: T) => U): U {
    return this.err !== undefined ? def(this.err) : mapper(this.ok!);
  }

  async async_map_or_else<U>(
    def: (err: E) => U,
    mapper: (ok: T) => Promise<U>,
  ): Promise<U> {
    return this.err !== undefined ? def(this.err!) : await mapper(this.ok!);
  }

  eq(other: Result<T, E>): boolean {
    if (other.is_ok() == this.is_ok()) {
      if (this.is_ok()) {
        return this.ok === other.ok;
      } else {
        return this.err === other.err;
      }
    } else {
      return false;
    }
  }
}

export function Ok<T, E>(x: T): Result<T, E> {
  return Result.new_ok(x);
}

export function Err<T, E>(x: E): Result<T, E> {
  return Result.new_err(x);
}
