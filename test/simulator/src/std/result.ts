import { None, Option, Some } from "./option.ts";

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
    return this.ok !== undefined;
  }

  is_err(): boolean {
    return this.err !== undefined;
  }

  unwrap(): T {
    return this.ok === undefined ? this.expect("Result is not Ok") : this.ok;
  }

  expect(msg: string): T {
    assert(this.ok != undefined, msg);

    return this.ok!;
  }

  unwrap_or_else(def: (err: E) => T) {
    return this.ok !== undefined ? this.ok : def(this.err!);
  }

  unwrap_or(def: T) {
    return this.ok !== undefined ? this.ok : def;
  }

  and_then<U>(next: (ok: T) => Result<U, E>): Result<U, E> {
    return this.ok !== undefined
      ? next(this.ok)
      : this as unknown as Result<U, E>;
  }

  async async_and_then<U>(
    next: (ok: T) => Promise<Result<U, E>>,
  ): Promise<Result<U, E>> {
    return this.ok !== undefined
      ? await next(this.ok)
      : this as unknown as Result<U, E>;
  }

  or_else<F>(alt: (err: E) => Result<T, F>) {
    return this.err ? alt(this.err) : this;
  }

  map<U>(mapper: (ok: T) => U): Result<U, E> {
    return this.ok !== undefined
      ? Result.new_ok(mapper(this.ok))
      : this as unknown as Result<U, E>;
  }

  async async_map<U>(mapper: (ok: T) => Promise<U>): Promise<Result<U, E>> {
    return this.ok
      ? Result.new_ok(await mapper(this.ok))
      : this as unknown as Result<U, E>;
  }

  map_err<F>(mapper: (err: E) => F): Result<T, F> {
    return this.err
      ? Result.new_err(mapper(this.err))
      : this as unknown as Result<T, F>;
  }

  map_or<U>(def: U, mapper: (ok: T) => U): U {
    return this.ok !== undefined ? mapper(this.ok) : def;
  }

  async async_map_or<U>(def: U, mapper: (ok: T) => Promise<U>): Promise<U> {
    return this.ok !== undefined ? await mapper(this.ok) : def;
  }

  map_or_else<U>(def: (err: E) => U, mapper: (ok: T) => U): U {
    return this.ok !== undefined ? mapper(this.ok) : def(this.err!);
  }

  async async_map_or_else<U>(
    def: (err: E) => U,
    mapper: (ok: T) => Promise<U>,
  ): Promise<U> {
    return this.ok !== undefined ? await mapper(this.ok) : def(this.err!);
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

function map_or_else() {
  const k = 21;

  const [x, y] = ([Ok("foo"), Err("foobar")] as Result<string, string>[]).map(
    (res) => res.map_or_else((_e) => k * 2, (v) => v.length),
  );
  console.assert(x == 3, "result of Ok(foo)");
  console.assert(y == 42, "result of Err(foobar)");
}

function map_or() {
  const k = 21;

  const [x, y] = ([Ok("foo"), Err("foobar")] as Result<string, string>[]).map(
    (res) => res.map_or(k * 3, (v) => v.length),
  );
  console.assert(x == 3, "result of Ok(foo)");
  console.assert(y == 63, "result of Err(foobar)");
}

const MAX_U32 = Math.pow(2, 32);
function checked_mul_u32(x: number, y: number): Option<number> {
  const product = x * y;
  return product < MAX_U32 ? Some(product) : None as Option<number>;
}

function and_then() {
  function sq_then_to_string(x: number): Result<string, string> {
    return checked_mul_u32(x, x).map((sq) => `${sq}`).ok_or("overflowed");
  }
  assert(Ok(2).and_then(sq_then_to_string).eq(Ok("4")));
  assert(Ok(1_000_000).and_then(sq_then_to_string).eq(Err("overflowed")));
  assert(
    Err<number, string>("not a number").and_then(sq_then_to_string).eq(
      Err("not a number"),
    ),
  );
}

function main() {
  map_or();
  map_or_else();
  and_then();
}

main();
