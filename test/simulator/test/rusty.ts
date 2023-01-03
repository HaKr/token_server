import { assertEquals } from "https://deno.land/std@0.168.0/testing/asserts.ts";

import { Err, None, Ok, Option, Result, Some } from "../src/deps.ts";

Deno.test("map_or_else", () => {
  const k = 21;

  const [x, y] = ([Ok("foo"), Err("foobar")] as Result<string, string>[]).map(
    (res) => res.mapOrElse((_e) => k * 2, (v) => v.length),
  );
  assertEquals(x, 3);
  assertEquals(y, 42);
});

Deno.test("map_or", () => {
  const k = 21;

  const [x, y] = ([Ok("foo"), Err("foobar")] as Result<string, string>[]).map(
    (res) => res.mapOrElse(() => k * 3, (v) => v.length),
  );
  assertEquals(x, 3);
  assertEquals(y, 63);
});

const MAX_U32 = Math.pow(2, 32);
function checked_mul_u32(x: number, y: number): Option<number> {
  const product = x * y;
  return product < MAX_U32 ? Some(product) : None();
}

Deno.test("and_then", () => {
  function sq_then_to_string(x: number): Result<string, string> {
    return checked_mul_u32(x, x).map((sq) => `${sq}`).okOr("overflowed");
  }
  assert_eq(Ok(2).andThen(sq_then_to_string), Ok("4"));
  assert_eq(Ok(1_000_000).andThen(sq_then_to_string), Err("overflowed"));
  assert_eq(
    Err<number, string>("not a number").andThen(sq_then_to_string),
    Err("not a number"),
  );
});

Deno.test("map_async", async () => {
  const ok = await Ok(42)
    .map((_n) => Promise.resolve(`fourty-two`));

  assert_eq(Ok("fourty-two"), ok);

  const err = await Err(42)
    .map((_n) => Promise.resolve(`fourty-two`));
  assert_eq(Err(42), err);
});

Deno.test("okOrElse", async () => {
  assert_eq(Some(42).okOrElse(() => "nope"), Ok(42));
  assert_eq(None().okOrElse(() => "nope"), Err("nope"));

  const ok = await Some(42)
    .okOrElse(() => Promise.resolve(`fourty-two`));

  assert_eq(Ok(42), ok);

  const err = await None<number>()
    .okOrElse(() => Promise.resolve(`fourty-two`));
  assert_eq(Err("fourty-two"), err);
});

function assert_eq<T>(expected: T): (result: T) => void | never;
function assert_eq<T>(expected: T, result: T): void | never;
function assert_eq<T>(
  expected: T,
  result?: T,
): void | never | ((result: T) => void | never) {
  if (result === undefined) return (result: T) => assert_eq(expected, result);

  assertEquals(result, expected);
}
