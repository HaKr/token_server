export class ProgressError {}

export class ProgressFetchFailed extends ProgressError {
  constructor(public reason: ProgressError) {
    super();
  }
}
export class ProgressCreateFailed extends ProgressError {}
export class ProgressRemoveFailed extends ProgressError {}
export class ProgressGotInvalidToken extends ProgressError {}
export class ProgressUpdateWithoutToken extends ProgressError {}
export class ProgressRemoveWithoutToken extends ProgressError {}
export class ProgressSkipThisOne extends ProgressError {}

export class ProgressStopped extends ProgressError {
  constructor(public reason: ProgressError) {
    super();
  }
}
