function assert(predicate: boolean, msg?: string | undefined): void | never {
  if (!predicate) throw new Error(msg || "Assertion failed");
}

type PossibleFuture<T> = T | Promise<T>;
export type ResultOrFuture<T, F> = PossibleFuture<Result<T, F>>;
export type FutureResult<T, F> = Promise<Result<T, F>>;

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

  isOk(): boolean {
    return !this.isErr();
  }

  isErr(): boolean {
    return this.err !== undefined;
  }

  unwrap(): T {
    return this.err !== undefined ? this.expect("Result is not Ok") : this.ok!;
  }

  expect(msg: string): T {
    assert(this.err === undefined, msg);

    return this.ok!;
  }

  unwrapOrElse(def: (err: E) => T) {
    return this.err !== undefined ? def(this.err!) : this.ok;
  }

  unwrapOr(def: T) {
    return this.err !== undefined ? def : this.ok;
  }

  /**
   * Calls op if the result is Ok, otherwise returns the Err value of self.
   *
   * This function can be used for control flow based on Result values.
   *
   * @param op callback to call when the result is Ok
   * @returns new Result if original was Ok, or original Err
   */
  andThen<U>(op: (ok: T) => Result<U, E>): Result<U, E>;
  andThen<U>(op: (ok: T) => FutureResult<U, E>): FutureResult<U, E>;
  andThen<U>(
    op: (ok: T) => ResultOrFuture<U, E>,
  ): ResultOrFuture<U, E> {
    if (this.err !== undefined) return this as unknown as Result<U, E>;
    return op(this.ok!) as ResultOrFuture<U, E>;
  }

  orElse<F>(alt: (err: E) => Result<T, F>) {
    return this.err !== undefined ? alt(this.err) : this;
  }

  /**
   * Maps a Result<T, E> to Result<U, E> by applying a function to a contained Ok value, leaving an Err value untouched.
   *
   * This function can be used to compose the results of two functions.
   *
   * @param mapper callback function that returns a new Ok value
   * @returns a new Result with the new Ok value or the original Err
   */
  map<U>(mapper: (ok: T) => Promise<U>): Promise<Result<U, E>>;
  map<U>(mapper: (ok: T) => U): Result<U, E>;
  map<U>(mapper: (ok: T) => U): Result<U, E> | Promise<Result<U, E>> {
    if (this.err !== undefined) return this as unknown as Result<U, E>;
    const result = mapper(this.ok!);
    return (result instanceof Promise)
      ? result.then(Result.new_ok) as Promise<Result<U, E>>
      : Result.new_ok(result);
  }

  mapErr<F>(mapper: (err: E) => F): Result<T, F> {
    return this.err !== undefined
      ? Result.new_err(mapper(this.err))
      : this as unknown as Result<T, F>;
  }

  mapOr<U>(def: U, mapper: (ok: T) => Promise<U>): Promise<U>;
  mapOr<U>(def: U, mapper: (ok: T) => U): U;
  mapOr<U>(def: U, mapper: (ok: T) => U): U {
    return this.err !== undefined ? def : mapper(this.ok!);
  }

  /** Maps a Result<T, E> to U by applying fallback function default to a contained Err value,
   * or function f to a contained Ok value.
   *
   * This function can be used to unpack a successful result while handling an error.
   *
   * @returns If either {def} or {mapper} return a Promise, then a Promise to U,
   *          otherwise U
   */
  mapOrElse<U>(
    def: (err: E) => Promise<U>,
    mapper: (ok: T) => Promise<U>,
  ): Promise<U>;
  mapOrElse<U>(
    def: (err: E) => Promise<U>,
    mapper: (ok: T) => U,
  ): Promise<U>;
  mapOrElse<U>(
    def: (err: E) => U,
    mapper: (ok: T) => Promise<U>,
  ): Promise<U>;
  mapOrElse<U>(
    def: (err: E) => U,
    mapper: (ok: T) => U,
  ): U;
  mapOrElse<U>(
    def: (err: E) => U,
    mapper: (ok: T) => U,
  ): U {
    return this.err !== undefined ? def(this.err!) : mapper(this.ok!);
  }

  eq(other: Result<T, E>): boolean {
    if (other.isOk() == this.isOk()) {
      if (this.isOk()) {
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
