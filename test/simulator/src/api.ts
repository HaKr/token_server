export type Options = { [key: string]: string | number | boolean };
export type Meta = Partial<Options>;
export type TokenInfo = { created: number; token: string; events: string[] };
export type TokenUpdateRequestBody = { token: string; meta?: Meta };
export type TokenUpdateResult = { token: string; meta: Meta };
export type TokenUpdateResponseBody = { Ok?: TokenUpdateResult; Err?: string };
export function isMeta(v: unknown): v is Meta {
  return typeof v == "object" && v != null;
}
export function formatMeta(meta: Meta) {
  return (meta === null
    ? "NIL"
    : `${meta.lastName ? meta.lastName : meta.year ? meta.year : ""}`) +
    `${
      (meta !== null && (meta.lastName || meta.year)) &&
        meta.updatedAt
        ? `, ${meta.updatedAt}`
        : ""
    }`;
}
export function maxWidth(str: string, width: number) {
  if (str.length <= width) return str;
  const half = (width - 5) / 2;
  return `${str.slice(0, half)} ... ${str.slice(-half)}`;
}
