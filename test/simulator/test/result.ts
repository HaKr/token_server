import { None, Option, Some } from "../src/std/option.ts";
import { Err, Ok, Result } from "../src/std/result.ts";

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

function assert(predicate: boolean, msg?: string | undefined): void | never {
  if (!predicate) throw new Error(msg || "Assertion failed");
}

function main() {
  map_or();
  map_or_else();
  and_then();
}

main();
