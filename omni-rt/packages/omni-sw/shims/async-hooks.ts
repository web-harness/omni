export class AsyncLocalStorage<TStore = unknown> {
  getStore(): TStore | undefined {
    return undefined;
  }

  run<TArgs extends unknown[], TResult>(
    _store: TStore,
    callback: (...args: TArgs) => TResult,
    ...args: TArgs
  ): TResult {
    return callback(...args);
  }
}

export class AsyncResource {
  static bind<TArgs extends unknown[], TResult>(callback: (...args: TArgs) => TResult): (...args: TArgs) => TResult {
    return callback;
  }
}
