export type Meta = { [key: string]: unknown };
export type TokenInfo = { created: number; token: string; events: string[] };
export type TokenUpdateRequestBody = { token: string; meta?: Meta };
export type TokenUpdateResult = { token: string; meta: Meta };
export type TokenUpdateResponseBody = { Ok?: TokenUpdateResult; Err?: string };
